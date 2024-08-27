use crate::common::{self, is_directory};
use anyhow::Result;
use std::fs::{copy, remove_dir, remove_file};

#[allow(dead_code)]
pub enum Operation {
    MOVE,
    COPY,
    REMOVE,
    SWAP,
    NOOP,
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
