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
        let mut node = Node::new(&path_buf.to_string_lossy().into_owned());
        if path_buf.is_dir() {
            node.node_type = NodeType::DIRECTORY;
        }
        node
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

/// Serialize a `Node` into a mapping-form entry: files become `Value::Null`,
/// directories become a nested `Value::Mapping`. File contents are NOT
/// round-tripped in this MVP.
pub fn convert_node(node: Node) -> Value {
    match node.node_type {
        NodeType::FILE => Value::Null,
        NodeType::DIRECTORY => {
            let mut map = Mapping::new();
            for child in node.children.into_iter() {
                let key = Value::String(child.basename.clone());
                map.insert(key, convert_node(child));
            }
            Value::Mapping(map)
        }
    }
}

/// Serialize a forest of top-level `Node`s into a single `Value::Mapping`.
pub fn convert_nodes(nodes: Vec<Node>) -> Value {
    let mut map = Mapping::new();
    for node in nodes.into_iter() {
        let key = Value::String(node.basename.clone());
        map.insert(key, convert_node(node));
    }
    Value::Mapping(map)
}

/// Convert a YAML `Value` into a forest of `Node`s.
///
/// Filefiles are pure mappings: the top-level `Value` must be a `Mapping`,
/// and each entry's value determines the node kind:
///   - `Value::String(s)` → file with contents `s`
///   - `Value::Null`      → empty file
///   - `Value::Mapping`   → directory with children
///   - `Value::Tagged(t)` → node with the parsed `Operation` attached
pub fn convert_value(value: &mut Value) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    let Value::Mapping(map) = value else {
        eprintln!("Filefile root must be a YAML mapping");
        return nodes;
    };

    for (key, val) in map.iter_mut() {
        let name = key.as_str().expect("Filefile keys must be strings");
        let mut node = Node::new(name);
        match val {
            Value::String(s) => {
                node.contents = Some(s.clone());
                node.node_type = NodeType::FILE;
            }
            Value::Null => {
                node.node_type = NodeType::FILE;
            }
            Value::Mapping(_) => {
                node.node_type = NodeType::DIRECTORY;
                node.add_children(convert_value(val));
            }
            Value::Tagged(t) => {
                match operations::Operation::from_tokens(
                    &t.tag.to_string(),
                    t.value.as_str().unwrap_or(""),
                ) {
                    Ok(op) => node.op = Some(op),
                    Err(err) => {
                        eprintln!("ERROR: {}", err);
                        std::process::exit(1);
                    }
                }
            }
            other => {
                eprintln!("WARN: unsupported value for key {:?}: {:?}", name, other);
            }
        }
        nodes.push(node);
    }
    nodes
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

    #[test]
    fn nested_mapping_becomes_directory_with_children() {
        let nodes = parse("outer:\n  inner: \"x\"\n");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].basename, "outer");
        assert_eq!(nodes[0].node_type, NodeType::DIRECTORY);
        assert_eq!(nodes[0].children.len(), 1);
        assert_eq!(nodes[0].children[0].basename, "inner");
        assert_eq!(nodes[0].children[0].contents.as_deref(), Some("x"));
    }

    #[test]
    fn tagged_value_attaches_operation() {
        let nodes = parse("repo: !git https://example.com/x.git\n");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].basename, "repo");
        assert!(matches!(nodes[0].op, Some(crate::operations::Operation::Git(_))));
    }
}
