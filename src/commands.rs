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
pub struct ApplyCommand {
    pub path: String,
    pub input: String,
    pub allow_remote_ops: bool,
}

impl ApplyCommand {
    pub fn from_subcommand(matches: &clap::ArgMatches, allow_remote_ops: bool) -> Result<Self> {
        let cwd = get_cwd().expect("Could not get CWD");
        let path = matches
            .get_one::<String>("path")
            .cloned()
            .unwrap_or_else(|| cwd.clone());
        let input = matches
            .get_one::<String>("file")
            .cloned()
            .unwrap_or_else(|| {
                combine_path(cwd.as_str(), filefile::FilefileNames::default().as_str())
            });
        validate_path(&path)?;
        Ok(Self {
            path,
            input,
            allow_remote_ops,
        })
    }

    pub fn from_file(file: &str, allow_remote_ops: bool) -> Result<Self> {
        let cwd = get_cwd().expect("Could not get CWD");
        validate_path(&cwd)?;
        Ok(Self {
            path: cwd,
            input: file.to_string(),
            allow_remote_ops,
        })
    }
}

fn apply_node(node: &Node, parent: &Path, dry: bool, ops_allowed: bool) -> Result<()> {
    let path = parent.join(&node.basename);
    // If an op is attached, let it produce the artifact — skip default create/write.
    if let Some(op) = &node.op {
        if !ops_allowed {
            anyhow::bail!(
                "remote Filefile contains {:?}; re-run with --allow-remote-ops to permit",
                op
            );
        }
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
                apply_node(child, &path, dry, ops_allowed)?;
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

pub fn is_remote(input: &str) -> bool {
    input.starts_with("http://") || input.starts_with("https://")
}

fn fetch_remote(url: &str) -> Result<String> {
    let body = ureq::get(url)
        .call()
        .map_err(|e| anyhow::anyhow!("fetch {}: {}", url, e))?
        .into_string()?;
    Ok(body)
}

impl Command for ApplyCommand {
    fn execute(&self) -> Result<()> {
        let remote = is_remote(&self.input);
        let raw_yaml = if remote {
            fetch_remote(&self.input)?
        } else {
            let mut s = String::new();
            File::open(&self.input)?.read_to_string(&mut s)?;
            s
        };

        let mut values: serde_yaml::Value =
            serde_yaml::from_str(&raw_yaml).expect("Couldn't convert yaml string to Value");
        let root_nodes = node::convert_value(&mut values);

        let dry = common::get_global_state().dry_run();
        let ops_allowed = !remote || self.allow_remote_ops;
        let root = Path::new(&self.path);
        for node in &root_nodes {
            apply_node(node, root, dry, ops_allowed)?;
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

        eprintln!(
            "Running generate command {:?}, {:?}",
            ctx_path.clone(),
            output.clone()
        );

        let mut root = Node::new(&ctx_path);
        root.node_type = node::NodeType::get(&ctx_path);
        let children = Node::parse_tree(&root).unwrap();

        let value = node::convert_nodes(children);
        let yaml = serde_yaml::to_string(&value).expect("Can't serialize 'Value' into string");

        let dry = common::get_global_state().dry_run();
        if !dry {
            let mut f = fs::File::create(&output)?;
            f.write_all(yaml.as_bytes())?;
        } else {
            eprintln!("DRY: writing filefile");
        }

        if to_stdout {
            println!("{}", yaml);
        }

        Ok(())
    }
}
