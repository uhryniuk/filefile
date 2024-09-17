use crate::{
    common::{self, combine_path, validate_path},
    filefile, get_basename, get_cwd, is_directory,
    node::{self, Node, NodeType},
};
use std::{fmt::format, fs::File, io::Read, path::Path};

use anyhow::Result;
use serde_yaml::{from_str, Value};

use crate::node::{create_filesystem, create_graph, create_yaml, parse_yaml, write_to_yaml};

pub trait Command {
    fn execute(&self);
}

struct CommandHelper;

impl CommandHelper {
    #[allow(dead_code)]
    fn read_file(path: &str) -> String {
        let mut file = File::open(&path).expect(format!("Cannot open {}", &path).as_str());
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect(format!("Cannot add content buffer for {}", &path).as_str());
        return contents;
    }
}

#[derive(Debug)]
pub struct ApplyCommand<'a> {
    pub filename: &'a str,
    pub path: &'a str,
    pub matches: &'a clap::ArgMatches,
}

impl<'a> ApplyCommand<'a> {
   fn parse_args(&self) -> Result<(String, String)> {
        let cwd = get_cwd().expect("Could not get CWD");
        let path = self
            .matches
            .get_one::<String>("path")
            .map(String::from)
            .unwrap_or_else(|| cwd.clone());
        
        // TODO this should get a location with the file too
        // currently this just get's a file path
        //
        // Use funciton to get default file path if this is not provided.
        let output = self
            .matches
            .get_one::<String>("file")
            .map(String::from)
            .unwrap_or_else(|| {
                combine_path(cwd.as_str(), filefile::FilefileNames::default().as_str())
            });

        validate_path(&path)?;

        // TODO we only need to validate basename, not filename.
        // ex. this/dir/blah -> only 'this/dir' needs to exist
        // validate_path(&output)?;

        Ok((path, output))
    }
}


impl<'a> Command for ApplyCommand<'a> {
    fn execute(&self) {
        let (path, output) = match self.parse_args() {
            Ok(p) => p,
            Err(err) => {
                eprintln!("Error: Cannot parse args {}", err);
                std::process::exit(1);
            }
        };

        println!(
            "Running root command {:?}, {:?}",
            path.clone(),
            output.clone()
        );

        // Create the path based on the graph.
        let node_type = if is_directory(&path) {
            NodeType::DIRECTORY
        } else {
            NodeType::FILE
        };
        let mut root = Node::new(
            get_basename(path.clone()).as_str(),
            node_type,
            &path.clone(),
        );

        // create references and write to file
        create_graph(&mut root);
        println!("self {:?}", self);

        let data: Value = from_str::<Value>(&common::read_file(self.filename)).unwrap();

            // .expect(format!("Could not read from {}", self.filename.to_string()));

        let mut nodes: Vec<Node> = node::parse_yaml(&data);

        let first = nodes.first().expect("trying to get first node");
        node::print_graph(first.clone(), 0);


        // TODO at this point we have the file system...
        // Now what do we want to do this it?
        // DFS iterate over it, keeping anything besides NOOP nodes.
        // Then we call generate again, to create a new file...

        let ctx = &mut common::get_global_state();
        if !ctx.dry_run() {
            println!("Running apply command");
        }

    }
}

pub struct Generate<'a> {
    pub filename: &'a str,
    pub path: &'a str,
    pub matches: &'a clap::ArgMatches,
}

impl<'a> Generate<'a> {
    fn parse_args(&self) -> Result<(String, String, bool)> {
        let cwd = get_cwd().expect("Could not get CWD");
        let path = self
            .matches
            .get_one::<String>("path")
            .map(String::from)
            .unwrap_or_else(|| cwd.clone());
        
        // TODO this should get a location with the file too
        // currently this just get's a file path
        //
        // Use funciton to get default file path if this is not provided.
        let output = self
            .matches
            .get_one::<String>("file")
            .map(String::from)
            .unwrap_or_else(|| {
                combine_path(cwd.as_str(), filefile::FilefileNames::default().as_str())
            });

        let stdout = self.matches.get_one::<bool>("stdout").unwrap_or(&false);
        println!("{:?}", stdout);

        validate_path(&path)?;

        // TODO we only need to validate basename, not filename.
        // ex. this/dir/blah -> only 'this/dir' needs to exist
        // validate_path(&output)?;

        Ok((path, output, *stdout))
    }
}

impl<'a> Command for Generate<'a> {
    fn execute(&self) {
        let (path, output, to_stdout) = match self.parse_args() {
            Ok(p) => p,
            Err(err) => {
                eprintln!("Error: Cannot parse args {}", err);
                std::process::exit(1);
            }
        };

        println!(
            "Running generate command {:?}, {:?}",
            path.clone(),
            output.clone()
        );

        // Create the path based on the graph.
        let node_type = if is_directory(&path) {
            NodeType::DIRECTORY
        } else {
            NodeType::FILE
        };
        let mut root = Node::new(
            get_basename(path.clone()).as_str(),
            node_type,
            &path.clone(),
        );

        // create references and write to file
        create_graph(&mut root);

        let ctx = &mut common::get_global_state();
        if !ctx.dry_run() {
            println!("Writing to yaml... {}", output.clone());
            let _ = write_to_yaml(root.clone(), &output);
        }

        if to_stdout {
            let yaml_str =
                serde_yaml::to_string(&create_yaml(root.clone())).expect("Cannot convert to yaml");
            println!("{}", yaml_str);
        }
    }
}
