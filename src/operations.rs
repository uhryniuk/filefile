use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Operations attached to a node via YAML tags.
///
/// `Git(url)` → `!git <url>`: clone `<url>` into the node's path at apply time.
/// `Sh(cmd)` → `!sh "<cmd>"`: run `<cmd>` with cwd = the node's parent directory.
/// `Noop` → default, do nothing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Git(String),
    Sh(String),
    Noop,
}

impl Operation {
    #[allow(dead_code)]
    pub fn from_raw_token(raw_token: &str) -> Result<Operation> {
        let mut tokens: VecDeque<&str> = raw_token.split_whitespace().collect();
        let op_token = tokens
            .pop_front()
            .ok_or_else(|| anyhow!("Empty op token"))?;
        let rest: String = tokens.into_iter().collect::<Vec<_>>().join(" ");
        Operation::from_tokens(op_token, &rest)
    }

    pub fn from_tokens(op_token: &str, op_args: &str) -> Result<Operation> {
        match op_token {
            "!git" => {
                let url = op_args.trim();
                if url.is_empty() {
                    return Err(anyhow!("'!git' requires a URL"));
                }
                Ok(Operation::Git(url.to_string()))
            }
            "!sh" => {
                let cmd = op_args.trim();
                if cmd.is_empty() {
                    return Err(anyhow!("'!sh' requires a command"));
                }
                Ok(Operation::Sh(cmd.to_string()))
            }
            other => Err(anyhow!("Unknown op tag: {}", other)),
        }
    }

    /// Execute the operation against a node at `node_path`. Honors `dry`.
    /// Stub for now; real implementation lands in a follow-up commit.
    pub fn execute(&self, _node_path: &std::path::Path, _dry: bool) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_git_tag() {
        let op = Operation::from_tokens("!git", "https://example.com/repo.git").unwrap();
        assert_eq!(op, Operation::Git("https://example.com/repo.git".into()));
    }

    #[test]
    fn parses_sh_tag() {
        let op = Operation::from_tokens("!sh", "echo hi > out").unwrap();
        assert_eq!(op, Operation::Sh("echo hi > out".into()));
    }

    #[test]
    fn unknown_tag_errors() {
        assert!(Operation::from_tokens("!nope", "whatever").is_err());
    }

    #[test]
    fn git_requires_url() {
        assert!(Operation::from_tokens("!git", "").is_err());
        assert!(Operation::from_tokens("!git", "   ").is_err());
    }

    #[test]
    fn sh_requires_command() {
        assert!(Operation::from_tokens("!sh", "").is_err());
    }
}
