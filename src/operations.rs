use crate::common::{self, is_directory};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fs::{copy, remove_dir, remove_file},
};

/// Types of operations supported
///
/// Move    = mv,   Moves a file to a new location.
/// COPY    = cp,   Copies a a file & file name (optional).
/// REMOVE  = rm,   Deletes a file.
/// SWAP    = swp,  Swaps 2 file's locations.
/// NOOP    = None, No operation, default operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    MOVE(String, String),
    COPY(String, String),
    REMOVE(String),
    SWAP(String, String),
    NOOP(),
}

impl Operation {
    // TODO change the names of these functiosn
    // A little misleading, this ok
    pub fn from_raw_token(raw_token: &str) -> Result<Operation> {
        let mut tokens: VecDeque<&str> = raw_token.split_whitespace().collect();
        let op_token = tokens
            .pop_front()
            .expect("Op token should always be at front");
        let token_str: &str = &tokens.into_iter().collect::<String>();
        Operation::from_tokens(op_token, token_str)
    }

    pub fn from_tokens(op_token: &str, op_args: &str) -> Result<Operation> {
        let tokens: VecDeque<&str> = op_args.split_whitespace().collect();
        match op_token {
            "!mv" => parse_move_args(tokens),
            "!cp" => parse_copy_args(tokens),
            "!rm" => parse_remove_args(tokens),
            "!swp" => parse_swap_args(tokens),
            _ => Err(anyhow!(
                "Invalid op_code token stream found: {} {}",
                op_token,
                op_args
            )),
        }
    }

    /// Execute the Operation object based on it's provided context.
    /// TODO this function should return a Result<Manifest>.
    /// If the operation cannot be done, or if there is a runtime failure
    /// aka semantic analysis failure or runtime error
    /// aka compiler error or a runtime error
    ///
    /// This should return a Manifest if successful
    ///     - So then we can log, which op moved what to where...
    ///     - Perhaps there is other info we may care about?
    pub fn execute(op: Operation) {
        println!("{:?}", op);
    }
}

fn parse_copy_args(mut arg_tokens: VecDeque<&str>) -> Result<Operation> {
    let token_count = 2;
    if arg_tokens.len() != token_count {
        return Err(anyhow!(format!(
            "Found {} arguments while '!cp' accepts {}.",
            arg_tokens.len(),
            token_count
        )));
    }

    let src = String::from(
        arg_tokens
            .pop_front()
            .expect("Unable to pop 'src' arg for '!cp'"),
    );
    let dest = String::from(
        arg_tokens
            .pop_front()
            .expect("Unable to pop 'dest' arg for '!cp'"),
    );
    Ok(Operation::COPY(src, dest))
}

fn parse_move_args(mut arg_tokens: VecDeque<&str>) -> Result<Operation> {
    let token_count = 2;
    if arg_tokens.len() != token_count {
        return Err(anyhow!(format!(
            "Found {} arguments while '!cp' accepts {}.",
            arg_tokens.len(),
            token_count
        )));
    }

    let src = String::from(
        arg_tokens
            .pop_front()
            .expect("Unable to pop 'src' arg for '!mv'"),
    );
    let dest = String::from(
        arg_tokens
            .pop_front()
            .expect("Unable to pop 'dest' arg for '!mv'"),
    );
    Ok(Operation::MOVE(src, dest))
}

fn parse_remove_args(mut arg_tokens: VecDeque<&str>) -> Result<Operation> {
    let token_count = 1;
    if arg_tokens.len() != token_count {
        return Err(anyhow!(format!(
            "Found {} arguments while '!cp' accepts {}.",
            arg_tokens.len(),
            token_count
        )));
    }

    let src = String::from(
        arg_tokens
            .pop_front()
            .expect("Unable to pop 'src' arg for '!rm'"),
    );
    Ok(Operation::REMOVE(src))
}

fn parse_swap_args(mut arg_tokens: VecDeque<&str>) -> Result<Operation> {
    let token_count = 2;
    if arg_tokens.len() != token_count {
        return Err(anyhow!(format!(
            "Found {} arguments while '!cp' accepts {}.",
            arg_tokens.len(),
            token_count
        )));
    }

    let src = String::from(
        arg_tokens
            .pop_front()
            .expect("Unable to pop 'src' arg for '!swp'"),
    );
    let dest = String::from(
        arg_tokens
            .pop_front()
            .expect("Unable to pop 'dest' arg for '!swp'"),
    );
    Ok(Operation::MOVE(src, dest))
}
