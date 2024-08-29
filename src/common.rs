use anyhow::{anyhow, Error, Result};
use serde_yaml::{from_str, to_string, Mapping, Value};
use std::{
    borrow::BorrowMut,
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    iter::repeat,
    path::{self, Path, PathBuf},
    sync::Mutex,
};

use crate::FilefileNamesIterator;

use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Context {
    force: bool,
    dry_run: bool,
    verbose: bool,
}

impl Context {
    fn new() -> Self {
        Context {
            force: false,
            dry_run: false,
            verbose: false,
        }
    }

    pub fn toggle_force(&mut self) {
        self.force = !self.force;
    }
    pub fn toggle_dry_run(&mut self) {
        self.dry_run = !self.dry_run;
    }
    pub fn toggle_verbose(&mut self) {
        self.verbose = !self.verbose;
    }

    #[allow(dead_code)]
    pub fn force(&self) -> bool {
        self.force
    }
    pub fn dry_run(&self) -> bool {
        self.dry_run
    }
    #[allow(dead_code)]
    pub fn verbose(&self) -> bool {
        self.verbose
    }
}

// NOTE Deadlocks are stupidly easy to run into.
// Locks are released once the ref is dropped, try to isolate usage.
static GLOBAL_CONTEXT: OnceLock<Mutex<Context>> = OnceLock::new();

pub fn init_global_state() {
    GLOBAL_CONTEXT
        .set(Mutex::new(Context::new()))
        .expect("Global state already initialized!");
}

pub fn get_global_state() -> std::sync::MutexGuard<'static, Context> {
    GLOBAL_CONTEXT
        .get()
        .expect("Global state not initialized!")
        .lock()
        .unwrap()
}

/// Join 2 strings as if they were paths.
///
/// ex.
/// path1 = "bongo"
/// path2 = "taco"
///
/// println!("{:?}", combine_path(path1, path2));
/// -> "bongo/taco"
pub fn combine_path(path1: &str, path2: &str) -> String {
    let combined_path = Path::new(path1).join(Path::new(path2));
    combined_path.to_str().unwrap().to_string()
}

pub fn get_cwd() -> anyhow::Result<String> {
    let cwd = std::env::current_dir()?
        .to_str()
        .ok_or_else(|| anyhow!("Cannot convert to &str"))?
        .to_string();
    Ok(cwd)
}

pub fn get_basename(path: String) -> String {
    let path_basename = Path::new(&path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    path_basename.to_string()
}

/// Using ArgMatches, attempt to get Filefile name.
/// On failure just looks for a default one in the CWD.
pub fn get_filefile_name(matches: &clap::ArgMatches, arg_name: String) -> String {
    matches
        .get_one::<String>(&arg_name)
        .map(String::from)
        .unwrap_or_else(|| {
            FilefileNamesIterator::new()
                .into_iter()
                .find(|default| Path::new(default).exists())
                .expect("No valid config file found")
                .to_string()
        })
}

pub fn is_directory(path: &str) -> bool {
    let path = Path::new(path);
    match fs::metadata(path) {
        Ok(metadata) => metadata.is_dir(),
        Err(_) => false,
    }
}

pub fn validate_path(path: &str) -> anyhow::Result<String> {
    if !Path::new(path).exists() {
        return Err(anyhow!("Path {:?} does not exist.", path));
    }

    Ok(path.to_string())
}

pub fn get_dirname(path: &str) -> String {
    let path = Path::new(path);
    let dirname = path.parent().unwrap().to_str();
    dirname.expect("Failed to convert path to str.").to_string()
}

