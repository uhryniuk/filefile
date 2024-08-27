use anyhow::{anyhow, Error, Result};
use serde_yaml::{from_str, to_string, Mapping, Value};
use std::{
    borrow::BorrowMut,
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    iter::repeat,
    path::{self, Path, PathBuf},
};

use crate::common::{combine_path, get_basename, get_cwd, get_filefile_name, is_directory};

use crate::commands::{Command, Generate};

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub ntype: NodeType,
    pub next: Vec<Node>,
    pub path: String,
}

impl Node {
    pub fn new(name: &str, ntype: NodeType, path: &str) -> Node {
        Node {
            name: name.to_string(),
            ntype,
            path: path.to_string(),
            next: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: Node) {
        self.next.push(child);
    }

    pub fn add_children(&mut self, children: Vec<Node>) {
        for child in children.into_iter() {
            self.next.push(child);
        }
    }

    #[allow(dead_code)]
    pub fn create_paths(&mut self) {
        !todo!("create manifest from node graph")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    FILE,
    DIRECTORY,
    // todo add others
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
                let mut children = node.next.clone();
                children.sort_by(|a, b| b.name.cmp(&a.name));
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

/// Pass the root node and a graph will be created.
pub fn create_graph(node: &mut Node) {
    let children = parse_dir(&node.path);
    for child in children.into_iter() {
        node.add_child(child);
    }
}

/// Traverse a FileGraph and convert it's structure into a Value
/// Will either return
/// - Value<String> on File
/// - Value<Mapping<String, Sequence<String>>> on Directory
///
pub fn create_yaml(node: Node) -> Value {
    let mut list: Vec<Value> = Vec::new();
    match node.ntype {
        NodeType::FILE => Value::from(node.name.to_string()),
        NodeType::DIRECTORY => {
            let mut map: Mapping = Mapping::new();
            for n in node.next.into_iter() {
                list.push(create_yaml(n));
            }
            map.insert(Value::String(node.name), Value::Sequence(list));
            Value::from(map)
        }
    }
}

pub fn write_to_yaml(root: Node, file_name: &str) -> anyhow::Result<()> {
    // // NOTE: remove the "thign" to just create it in root.
    // // Ideally, ill have a '--prefix', '--p' flag to pass
    // // some dir, if the user doesn't want to use ff in the local dir.
    // create_filesystem(b.clone(), String::from(""));
    // // TODO match this error, don't unwrap. in the event we created bad yaml.
    let yaml_str = serde_yaml::to_string(&create_yaml(root.clone()))?;
    fs::write(file_name, &yaml_str)?;

    Ok(())
}

/// Create a directory from a node and it's children.
#[allow(dead_code)]
pub fn create_filesystem(node: Node, ctx: String) {
    let path = combine_path(ctx.as_str(), &node.name);
    match node.ntype {
        NodeType::FILE => {
            // Create the current file, writing nothing to it.
            println!("Writing file to {:?}", path.clone());
            fs::File::create_new(path.clone()).unwrap();
        }
        NodeType::DIRECTORY => {
            // Create the current directory and recurse.
            println!("Writing dir to {:?}", path.clone());
            let _ = fs::create_dir(path.clone());
            for child in node.next.into_iter() {
                create_filesystem(child, path.clone());
            }
        }
    };
}

#[allow(dead_code)]
pub fn print_graph(node: Node, depth: usize) {
    let space = String::from(">".repeat(depth as usize));
    println!("{}{:?}", space, node.name);
    if node.ntype == NodeType::DIRECTORY {
        for child in node.next.into_iter() {
            print_graph(child, depth + 1);
        }
    }
}

/// Not sure what to return, rehaps the actually diff
/// otherwise could just get a bool or something.
#[allow(dead_code)]
pub fn compare_graphs(a: Node, b: Node) {

    // traverse each of the graph
    // sort the nodes in the 'next'
    // compare the nodes in the 'next'
    // compare the current nodes against each other.
    // need to make sure the number of nodes are equal too at each layers
    //  - this can be a short circuit feature
}

#[allow(dead_code)]
pub fn parse_yaml(value: &Value) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();

    match value {
        Value::Null => todo!("not implemented"), // drop the node? create a comment?
        Value::Bool(_b) => todo!("not implemented"), // ignore/something - ghosting? just keeps the
        // node in the file but don't do anything?
        Value::Number(_n) => todo!("not implemented"), // i have no idea, probably the access level
        Value::String(s) => {
            nodes.push(Node::new(s.as_str(), NodeType::FILE, s.as_str()));
        }
        Value::Sequence(seq) => {
            // files within a dir.
            for (_index, item) in seq.iter().enumerate() {
                nodes.extend(parse_yaml(item));
            }
            return nodes;
        }
        Value::Mapping(map) => {
            // conents of the file?, commands or even types?
            for (key, value) in map {
                let key_value = key.as_str().expect("Should get them out of graph");
                let mut local_node = Node::new(key_value, NodeType::DIRECTORY, key_value);
                local_node.add_children(parse_yaml(value));
                nodes.push(local_node);
            }
            return nodes;
        }
        Value::Tagged(t) => {
            // NOTE Tags are literally the '!' ive been using LOL
            println!("{:?}", t);
        }
    };

    return nodes;
}

pub fn parse_dir(path: &str) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();

    // Base case for plain old files.
    if !is_directory(path) {
        return nodes;
    }

    let dir_iter = fs::read_dir(path).expect(format!("Cannot read directory {}", path).as_str());
    for e in dir_iter {
        let entry = e.unwrap();
        let file_name: String = entry.file_name().to_str().unwrap().to_string();
        let abs_path = combine_path(&path, &file_name);

        if entry.path().is_dir() {
            let mut node = Node::new(&file_name, NodeType::DIRECTORY, &abs_path);
            node.add_children(parse_dir(&abs_path));
            nodes.push(node);
        } else if entry.path().is_file() {
            let node = Node::new(&file_name, NodeType::FILE, &abs_path);
            nodes.push(node);
        }
    }

    return nodes;
}
