use crate::{
    common::{combine_path, validate_path},
    filefile, get_cwd,
    node::{self, Node},
};
use std::{fs::File, io::Read, io::Write, path::PathBuf};

use anyhow::{self, Result};

pub trait Command {
    fn execute(&self) -> Result<()>;
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

        let input = self
            .matches
            .get_one::<String>("file")
            .map(String::from)
            .unwrap_or_else(|| {
                combine_path(cwd.as_str(), filefile::FilefileNames::default().as_str())
            });

        validate_path(&path)?;

        Ok((path, input))
    }
}

impl<'a> Command for ApplyCommand<'a> {
    fn execute(&self) -> Result<()> {
        let (path, input) = match self.parse_args() {
            Ok(p) => p,
            Err(err) => {
                eprintln!("Error: Cannot parse args {}", err);
                std::process::exit(1);
            }
        };

        println!(
            "Running apply command {:?}, {:?}",
            path.clone(),
            input.clone()
        );

        // read input file into string buffer
        let mut raw_yaml = String::new();
        let mut f = File::open(&input)?;
        f.read_to_string(&mut raw_yaml)?;

        println!("{}", raw_yaml);

        let mut values: serde_yaml::Value =
            serde_yaml::from_str(&raw_yaml).expect("Couldn't convert yaml string to Value");
        let root_nodes = node::convert_value(&mut values);

        let mut node_queue: Vec<Vec<Node>> = vec![root_nodes.clone()];
        while let Some(mut nodes) = node_queue.pop() {
            for node in nodes.iter_mut() {
                // Update the directory to use provided path...
                node.prefix_path(path.as_str());

                // TODO for debugging only...
                println!("{:?}", node);

                match node.node_type {
                    node::NodeType::FILE => {
                        std::fs::write(node.path_as_str().as_str(), "")?;
                    }
                    node::NodeType::DIRECTORY => {
                        std::fs::create_dir_all(node.path_as_str().as_str())?;
                    }
                }

                // Push the children of the current node onto the queue
                if !node.children.is_empty() {
                    node_queue.push(node.children.clone());
                }
            }
        }

        // 1. Convert to values
        // 2. Convert to nodes.
        // Tagged -> Operation
        // 3. Iterate & Execute

        // // Context is the the root directory to run the apply command in.
        // let mut context = Node::new(&path);
        // let mut nodes: Vec<Node> = node::parse_yaml(&data);
        // context.add_children(nodes.clone());
        //
        // // node::print_graph(context, 0);
        //
        //
        // // TODO at this point we have the file system...
        // // Now what do we want to do this it?
        // // DFS iterate over it, keeping anything besides NOOP nodes.
        // // Then we call generate again, to create a new file...
        //
        // let ctx = &mut common::get_global_state();
        // if !ctx.dry_run() {
        //     println!("Running apply command");
        // }

        Ok(())
    }
}

pub struct GenerateCommand<'a> {
    pub matches: &'a clap::ArgMatches,
}

impl<'a> GenerateCommand<'a> {
    fn parse_args(&self) -> Result<(String, String, bool)> {
        let cwd = get_cwd().expect("Could not get CWD");
        let path = self
            .matches
            .get_one::<String>("path")
            .map(String::from)
            .unwrap_or_else(|| cwd.clone());

        // Use funciton to get default file path if this is not provided.
        let output = self
            .matches
            .get_one::<String>("file")
            .map(String::from)
            .unwrap_or_else(|| {
                combine_path(cwd.as_str(), filefile::FilefileNames::default().as_str())
            });

        let stdout = self.matches.get_one::<bool>("stdout").unwrap_or(&false);


        validate_path(&path)?;

        Ok((path, output, *stdout))
    }
}

impl<'a> Command for GenerateCommand<'a> {
    fn execute(&self) -> Result<()> {
        let (ctx_path, output, to_stdout) = match self.parse_args() {
            Ok(p) => p,
            Err(err) => {
                eprintln!("Error: Cannot parse args {}", err);
                std::process::exit(1);
            }
        };

        println!(
            "Running generate command {:?}, {:?}",
            ctx_path.clone(),
            output.clone()
        );

        // get filesystem tree
        let root = Node::new(&ctx_path);
        let children = Node::parse_tree(&root).unwrap();

        // convert Node -> serde_yaml::Value -> String
        let values = node::convert_nodes(children);
        let yaml = serde_yaml::to_string(&values).expect("Can't serialize 'Value' into string");

        // check if we should write to file
        let ctx = &mut crate::common::get_global_state();
        if !ctx.dry_run() {
            let mut f = std::fs::File::create(&output)?;
            f.write_all(&yaml.as_bytes())?;
        } else {
            eprintln!("DRY: writing filefile");
        }

        if to_stdout {
            println!("{}", yaml);
        }

        Ok(())
    }
}
