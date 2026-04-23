use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::{
    fs::{self},
    path::PathBuf,
};

use crate::{common::is_directory, operations};

#[derive(Debug, Clone)]
pub struct Node {
    pub path: PathBuf,
    pub basename: String,
    pub node_type: NodeType,
    pub children: Vec<Node>,
    pub op: Option<operations::Operation>,
    pub contents: Option<String>,
}

impl Node {
    pub fn new(name: &str) -> Node {
        // let path: PathBuf = match fs::canonicalize(name) {
        //     Ok(p) => p,
        //     Err(_e) => {
        //         // if path doesn't exist, create future abs path.
        //         let cwd = get_cwd().expect("Couldn't get CWD");
        //
        //         PathBuf::from(cwd).join(PathBuf::from(name))
        //     }
        // };

        // A whole lot easier than fangaling with canonicalize and prefix swapping...
        let path: PathBuf = PathBuf::from(name);

        let basename: String = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Could not get the basename.".to_string());

        Node {
            path,
            basename,
            node_type: NodeType::FILE, // when > 0 children, set to DIRECTORY
            children: Vec::new(),
            op: None,
            contents: None,
        }
    }

    pub fn prefix_path(&mut self, dirname: &str) {
        self.path = PathBuf::from(dirname).join(self.path.clone());
    }

    pub fn path_as_str(&self) -> String {
        self.path.to_str().map(|name| name.to_string()).unwrap()
    }

    pub fn add_child(&mut self, child: Node) {
        self.children.push(child);
        if self.children.len() > 0 {
            self.node_type = NodeType::DIRECTORY;
        }
    }

    pub fn add_children(&mut self, children: Vec<Node>) {
        for child in children.into_iter() {
            self.add_child(child);
        }
    }

    // TEST REQUIRED
    // Parse the children of the node, explicit depth + 1.
    pub fn parse_children(node: &Node) -> Result<Vec<Node>> {
        let mut children: Vec<Node> = Vec::new();

        // short circuit on file type.
        if node.node_type == NodeType::FILE {
            return Ok(children);
        }

        // Convert PathBuf into Node
        for entry in fs::read_dir(&node.path)? {
            children.push(entry?.path().into());
        }

        Ok(children)
    }

    // TEST REQUIRED
    /// Parse entire file tree of the node, explicit depth + n.
    pub fn parse_tree(node: &Node) -> Result<Vec<Node>> {
        let mut local: Vec<Node> = Vec::new();

        // Recursively parses the entire tree
        for child in Node::parse_children(&node)?.iter_mut() {
            child.add_children(Node::parse_tree(&child)?);
            local.push(child.clone());
        }

        Ok(local)
    }
}

impl From<PathBuf> for Node {
    fn from(path_buf: PathBuf) -> Self {
        Node::new(&path_buf.to_string_lossy().into_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    FILE,
    DIRECTORY,
}

impl NodeType {
    pub fn get(name: &str) -> NodeType {
        match is_directory(name) {
            true => NodeType::DIRECTORY,
            false => NodeType::FILE,
        }
    }
}

pub mod iterators {
    use crate::Node;

    pub struct DfsIterator {
        stack: Vec<Node>,
    }

    impl DfsIterator {
        #[allow(dead_code)]
        pub fn new(root: Node) -> Self {
            let mut stack = Vec::new();
            stack.push(root);
            DfsIterator { stack }
        }
    }

    impl Iterator for DfsIterator {
        type Item = Node;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(node) = self.stack.pop() {
                // Push children in reverse alphabetical order to the stack
                let mut children = node.children.clone();
                children.sort_by(|a, b| b.basename.cmp(&a.basename));
                for child in children {
                    self.stack.push(child);
                }
                Some(node)
            } else {
                None
            }
        }
    }
}

//
mod yaml {}
// Pass the root node and a graph will be created.
// pub fn create_graph(node_ctx: &str) -> Vec<Node> {
//     let node = Node::new(node_ctx);
//     let mut children = parse_dir(&node.name);
//     for child in children.iter_mut() {
//         child.add_children(parse_dir(&child.name))
//     }
//     children
// }

// Traverse a FileGraph and convert it's structure into a Value
// Will either return
// - Value<String> on File
// - Value<Mapping<String, Sequence<String>>> on Directory
//
pub fn convert_node(node: Node) -> Value {
    let mut list: Vec<Value> = Vec::new();
    match node.node_type {
        NodeType::FILE => Value::from(node.basename.to_string()),
        NodeType::DIRECTORY => {
            let mut map: Mapping = Mapping::new();
            for n in node.children.iter() {
                list.push(convert_node(n.clone()));
            }
            map.insert(Value::String(node.basename.clone()), Value::Sequence(list));
            Value::from(map)
        }
    }
}
pub fn convert_nodes(mut nodes: Vec<Node>) -> Vec<Value> {
    let mut list: Vec<Value> = Vec::new();
    for node in nodes.iter_mut() {
        list.push(convert_node(node.clone()));
    }
    list
}

///
pub fn convert_value(value: &mut Value) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();

    match value {
        Value::Null => todo!("not implemented"), // drop the node? create a comment?
        // TODO these should throw error, just a warning to stderr that they aren't implemented.
        Value::Bool(_b) => todo!("not implemented"),
        // node in the file but don't do anything?
        Value::Number(_n) => todo!("not implemented"), // i have no idea, probably the access level
        Value::String(s) => {
            nodes.push(Node::new(s.as_str()));
        }
        Value::Sequence(seq) => {
            // files within a dir.
            for (_index, item) in seq.iter_mut().enumerate() {
                nodes.extend(convert_value(item));
            }
            return nodes;
        }
        Value::Mapping(map) => {
            // println!("MAP: {:?}", map);

            // TODO check if the map value is a tag
            // if so, then don't recurse.

            for (key, value) in map.iter_mut() {
                let key_value = key.as_str().expect("Should get them out of graph");
                let mut local_node = Node::new(key_value);
                match value {
                    Value::String(s) => {
                        local_node.contents = Some(s.clone());
                        local_node.node_type = NodeType::FILE;
                    }
                    Value::Null => {
                        local_node.node_type = NodeType::FILE;
                    }
                    Value::Sequence(seq) => {
                        // Update the path before it's turned into a node.
                        // This way it maintains the proper basename and abs path.
                        for index in seq.iter_mut() {
                            if let Value::String(str) = index {
                                // TODO join these paths in a more optimal way... PathBuf...
                                *index = serde_yaml::Value::String(format!(
                                    "{}/{}",
                                    key.as_str().unwrap(),
                                    str
                                ));
                            }
                        }
                        local_node.add_children(convert_value(value));
                    }
                    Value::Tagged(t) => {
                        // TODO this one is only called if this occurs:
                        // When operations are used on maps.
                        // mongo:
                        //  - something: !cp blah blorp
                        //
                        //  We should only take 1 arg, since we infer the location.
                        //  so the above becomes:
                        //  mongo:
                        //      - something: !cp blah
                        //
                        //  Where the contents of blah will be in a new or overwritten file
                        //  something
                        match operations::Operation::from_tokens(
                            &t.tag.to_string(),
                            t.value.as_str().unwrap(),
                        ) {
                            Ok(op) => {
                                local_node.op = Some(op);
                            }
                            Err(err) => {
                                // TODO better error reporting
                                eprintln!("ERROR: {}", err);
                                std::process::exit(1);
                            }
                        }
                    }
                    _ => {}
                }
                nodes.push(local_node);
            }

            return nodes;
        }
        Value::Tagged(t) => {
            // Called when tags are used without a mapping
            // like:
            // mongo:
            //  - !cp blah blorp
            //  Need to enforce 2 arguments here.
            eprintln!("TAG IMPL ERROR: {:?}", t);
        }
    };

    return nodes;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> Vec<Node> {
        let mut v: Value = serde_yaml::from_str(yaml).expect("yaml parse");
        convert_value(&mut v)
    }

    #[test]
    fn string_map_value_becomes_file_with_contents() {
        let nodes = parse("foo: \"bar\"\n");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].basename, "foo");
        assert_eq!(nodes[0].node_type, NodeType::FILE);
        assert_eq!(nodes[0].contents.as_deref(), Some("bar"));
    }

    #[test]
    fn null_map_value_becomes_empty_file() {
        let nodes = parse("foo:\n");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].basename, "foo");
        assert_eq!(nodes[0].node_type, NodeType::FILE);
        assert!(nodes[0].contents.is_none());
    }
}
