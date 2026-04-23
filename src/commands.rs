use crate::{
    common::{self, combine_path, validate_path},
    filefile, get_cwd,
    node::{self, Node},
};
use std::{
    fs::{self, File},
    io::Read,
    io::Write,
    path::{Path, PathBuf},
};

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

fn apply_node(node: &Node, parent: &Path, dry: bool) -> Result<()> {
    let path = parent.join(&node.basename);
    // If an op is attached, let it produce the artifact — skip default create/write.
    if let Some(op) = &node.op {
        return op.execute(&path, dry);
    }
    match node.node_type {
        node::NodeType::DIRECTORY => {
            if dry {
                eprintln!("DRY mkdir {:?}", path);
            } else {
                fs::create_dir_all(&path)?;
            }
            for child in &node.children {
                apply_node(child, &path, dry)?;
            }
        }
        node::NodeType::FILE => {
            let body = node.contents.clone().unwrap_or_default();
            if dry {
                eprintln!("DRY write {:?} ({} bytes)", path, body.len());
            } else {
                fs::write(&path, body)?;
            }
        }
    }
    Ok(())
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

        let mut raw_yaml = String::new();
        let mut f = File::open(&input)?;
        f.read_to_string(&mut raw_yaml)?;

        let mut values: serde_yaml::Value =
            serde_yaml::from_str(&raw_yaml).expect("Couldn't convert yaml string to Value");
        let root_nodes = node::convert_value(&mut values);

        let dry = common::get_global_state().dry_run();
        let root = Path::new(&path);
        for node in &root_nodes {
            apply_node(node, root, dry)?;
        }

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
