use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::{from_str, to_string, value::TaggedValue, Mapping, Value};
use std::{
    borrow::BorrowMut,
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    iter::repeat,
    path::{self, Path, PathBuf},
};

use crate::{
    common::{combine_path, get_basename, get_cwd, get_filefile_name, is_directory},
    operations,
};

#[derive(Debug, Clone)]
pub struct Node {
    pub path: PathBuf,
    pub basename: String,
    pub node_type: NodeType,
    pub children: Vec<Node>,
    pub op: Option<operations::Operation>,
}

impl Node {
    pub fn new(name: &str) -> Node {
        // canonicalize it, does it exist
        // get the base name
        // get node type

        let path: PathBuf = match fs::canonicalize(name) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("ERROR: Couldn't get aboslute for {} because {}", name, e);
                std::process::exit(1);
            }
        };

        let basename: String = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Could not get the basename.".to_string());

        Node {
            path,
            basename,
            node_type: NodeType::get(name),
            children: Vec::new(),
            op: None,
        }
    }

    pub fn path_as_str(&self) -> String {
        self.path.to_str().map(|name| name.to_string()).unwrap()
    }

    pub fn add_child(&mut self, child: Node) {
        self.children.push(child);
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
    // match node.node_type {
    //     NodeType::FILE => Value::from(node.basename.to_string()),
    //     NodeType::DIRECTORY => {
    //         let mut map: Mapping = Mapping::new();
    //         for n in node.children.iter() {
    //             list.push(create_yaml(&n));
    //         }
    //         map.insert(Value::String(node.basename.clone()), Value::Sequence(list));
    //         Value::from(map)
    //     }
    // }
    for node in nodes.iter_mut() {
        list.push(convert_node(node.clone()));
    }

    list
}
//
// pub fn write_to_yaml(nodes: Vec<Node>, file_name: &str) -> anyhow::Result<()> {
//     let mut buffer = String::new();
//     for node in nodes.iter() {
//         let yaml_str = serde_yaml::to_string(&create_yaml(node.clone()))?;
//         buffer.push_str(&yaml_str);
//     }
//
//     fs::write(file_name, buffer)?;
//     Ok(())
// }
//
// /// Create a directory from a node and it's children.
// #[allow(dead_code)]
// pub fn create_filesystem(node: Node, ctx: String) {
//     let path = combine_path(ctx.as_str(), &node.name);
//     match node.node_type {
//         NodeType::FILE => {
//             // Create the current file, writing nothing to it.
//             println!("Writing file to {:?}", path.clone());
//             fs::File::create_new(path.clone()).unwrap();
//         }
//         NodeType::DIRECTORY => {
//             // Create the current directory and recurse.
//             println!("Writing dir to {:?}", path.clone());
//             let _ = fs::create_dir(path.clone());
//             for child in node.next.into_iter() {
//                 create_filesystem(child, path.clone());
//             }
//         }
//     };
// }
//
// #[allow(dead_code)]
// pub fn print_graph(node: Node, depth: usize) {
//     let padding = String::from(">".repeat(depth as usize));
//     println!("{}{:?}, ({:?})", padding, node.name, node.op);
//     if node.node_type == NodeType::DIRECTORY {
//         for child in node.next.into_iter() {
//             print_graph(child, depth + 1);
//         }
//     }
// }
//
// pub fn parse_yaml(value: &Value) -> Vec<Node> {
//     let mut nodes: Vec<Node> = Vec::new();
//
//     match value {
//         Value::Null => todo!("not implemented"), // drop the node? create a comment?
//         Value::Bool(_b) => todo!("not implemented"), // ignore/something - ghosting? just keeps the
//         // node in the file but don't do anything?
//         Value::Number(_n) => todo!("not implemented"), // i have no idea, probably the access level
//         Value::String(s) => {
//             nodes.push(Node::new(s.as_str()));
//         }
//         Value::Sequence(seq) => {
//             // files within a dir.
//             for (_index, item) in seq.iter().enumerate() {
//                 nodes.extend(parse_yaml(item));
//             }
//             return nodes;
//         }
//         Value::Mapping(map) => {
//             // conents of the file?, commands or even types?
//             println!("MAP: {:?}", map);
//
//             // TODO check if the map value is a tag
//             // if so, then don't recurse.
//
//
//
//             for (key, value) in map {
//                 let key_value = key.as_str().expect("Should get them out of graph");
//                 let mut local_node = Node::new(key_value);
//                 match value {
//                     Value::Sequence(_seq) => {
//                         local_node.add_children(parse_yaml(value));
//                     },
//                     Value::Tagged(t) => {
//                         match operations::Operation::from_tokens(&t.tag.to_string(), t.value.as_str().unwrap()) {
//                             Ok(op) => {
//                                 local_node.op = Some(op);
//                             },
//                             Err(err) => {
//                                 // TODO better error reporting
//                                 eprintln!("ERROR: {}", err);
//                                 std::process::exit(1);
//                             }
//                         }
//                     }
//                     _ => {},
//                 }
//                 nodes.push(local_node);
//             }
//
//             // println!("{:?}", nodes);
//             return nodes;
//         }
//         Value::Tagged(t) => {
//             // NOTE Tags are literally the '!' ive been using LOL
//             println!("ERROR TAG FOUND: {:?}", t);
//             eprintln!("TAGS SH")
//         }
//     };
//
//     return nodes;
// }
//
// pub fn parse_dir(path: &str) -> Vec<Node> {
//     let mut nodes: Vec<Node> = Vec::new();
//
//     // Base case for plain old files.
//     if !is_directory(path) {
//         return nodes;
//     }
//
//     let dir_iter = fs::read_dir(path).expect(format!("Cannot read directory {}", path).as_str());
//     for e in dir_iter {
//         let entry = e.unwrap();
//         let file_name: String = entry.file_name().to_str().unwrap().to_string();
//         let abs_path = combine_path(&path, &file_name);
//
//         if entry.path().is_dir() {
//             let mut node = Node::new(&file_name);
//             node.add_children(parse_dir(&abs_path));
//             nodes.push(node);
//         } else if entry.path().is_file() {
//             let node = Node::new(&file_name);
//             nodes.push(node);
//         }
//     }
//
//     return nodes;
// }
