use crate::common::{self, is_directory};
use anyhow::{anyhow, Result};
use std::{collections::VecDeque, fs::{copy, remove_dir, remove_file}, path::PathBuf};


#[allow(dead_code)]
/// Types of operations supported
///
/// Move    = mv,   Moves a file to a new location.
/// COPY    = cp,   Copies a a file & file name (optional).
/// REMOVE  = rm,   Deletes a file.
/// SWAP    = swp,  Swaps 2 file's locations.
/// NOOP    = None, No operation, default operation.

pub enum Operation {
    MOVE(String, String),
    COPY(),
    REMOVE(),
    SWAP(),
    NOOP(),
}

impl Operation {
    /// Parses the T
    pub fn parse_op(raw_token: &str) -> Result<Operation> {
    
        let mut tokens: VecDeque<&str> = raw_token.split_whitespace().collect();
        let op_token = tokens.pop_front();

        match op_token {
            // Some(&"!mv") => 123,
            Some("!cp") => Ok(Operation::COPY()),
            // "!rm" => Ok(Operation::REMOVE),
            // "!swp" => Ok(Operation::SWAP),
            _ => Err(anyhow!("Invalid op_code token stream found: {}", (" "))),
        }
    }

}


#[allow(dead_code)]
fn validate_mv(mut arg_tokens: VecDeque<&str>) -> anyhow::Result<Operation> {
    
    if arg_tokens.len() != 2 {
        return Err(anyhow!("'!mv' requires 2 arguments, {} were provided.", arg_tokens.len()));
    }
    // Validate data src exists.
    let src = common::validate_path(arg_tokens.pop_front().unwrap())?;
    
    // Validate the dirname exists for dest.
    let dirname = &common::get_dirname(arg_tokens.pop_front().unwrap());
    let dest = common::validate_path(dirname)?;
    
    Ok(Operation::MOVE(src, dest))
}

#[allow(dead_code)]
pub fn cp(input: &str, output: &str) -> anyhow::Result<()> {
    let ctx = &mut common::get_global_state();
    if ctx.dry_run() {
        println!("Attempting to copy '{}' to '{}'", input, output);
        return Ok(());
    }

    // put a check in here to not copy the file if
    // the output file has the same name and checksum
    // basically so we can output "not changes to make in cli output"

    let res = copy(input, output);
    println!("{:?}", res);

    return Ok(());
}

#[allow(dead_code)]
pub fn rm(input: &str) -> anyhow::Result<()> {
    let ctx = &mut common::get_global_state();
    if ctx.dry_run() {
        println!("Attempting to remove '{}'", input);
        return Ok(());
    }

    // TODO propogate the exit code and log message.
    let res = if is_directory(input) {
        remove_dir(input)
    } else {
        remove_file(input)
    };

    println!("{:?}", res);
    return Ok(());
}
