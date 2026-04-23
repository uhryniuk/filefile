use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::Path;
use std::process::{Command, Stdio};

/// Operations attached to a node via YAML tags.
///
/// `Git(url)` → `!git <url>`: clone `<url>` into the node's path at apply time.
/// `Sh(cmd)` → `!sh "<cmd>"`: run `<cmd>` with cwd = the node's parent directory
///   and write the captured stdout to the node's path. Stderr is inherited.
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
    pub fn execute(&self, node_path: &Path, dry: bool) -> Result<()> {
        match self {
            Operation::Git(url) => {
                if dry {
                    eprintln!("DRY git clone {} {:?}", url, node_path);
                    return Ok(());
                }
                let status = Command::new("git")
                    .arg("clone")
                    .arg(url)
                    .arg(node_path)
                    .status()?;
                if !status.success() {
                    anyhow::bail!("git clone failed: {}", url);
                }
            }
            Operation::Sh(cmd) => {
                let cwd = node_path.parent().unwrap_or(Path::new("."));
                if dry {
                    eprintln!("DRY sh -c {:?} (cwd {:?}) -> {:?}", cmd, cwd, node_path);
                    return Ok(());
                }
                let out = Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .current_dir(cwd)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .output()?;
                if !out.status.success() {
                    anyhow::bail!("sh failed: {}", cmd);
                }
                std::fs::write(node_path, &out.stdout)?;
            }
            Operation::Noop => {}
        }
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

    #[test]
    fn sh_execute_writes_stdout_to_node_path() {
        let td = tempfile::tempdir().unwrap();
        let node_path = td.path().join("marker");
        let op = Operation::Sh("printf hi".into());
        op.execute(&node_path, false).unwrap();
        assert_eq!(std::fs::read_to_string(&node_path).unwrap(), "hi");
    }

    #[test]
    fn sh_execute_runs_with_cwd_of_parent_dir() {
        // The command reads from a sibling file — only works if cwd is the
        // node's parent (where that sibling lives).
        let td = tempfile::tempdir().unwrap();
        std::fs::write(td.path().join("sibling"), "from-sibling").unwrap();
        let node_path = td.path().join("out");
        let op = Operation::Sh("cat sibling".into());
        op.execute(&node_path, false).unwrap();
        assert_eq!(std::fs::read_to_string(&node_path).unwrap(), "from-sibling");
    }

    #[test]
    fn sh_execute_dry_run_does_not_write() {
        let td = tempfile::tempdir().unwrap();
        let node_path = td.path().join("marker");
        let op = Operation::Sh("printf hi".into());
        op.execute(&node_path, true).unwrap();
        assert!(!node_path.exists());
    }

    #[test]
    fn sh_execute_bails_on_nonzero_exit() {
        let td = tempfile::tempdir().unwrap();
        let node_path = td.path().join("out");
        let op = Operation::Sh("exit 1".into());
        assert!(op.execute(&node_path, false).is_err());
        assert!(!node_path.exists());
    }

    #[test]
    #[ignore] // requires network
    fn git_execute_clones() {
        let td = tempfile::tempdir().unwrap();
        let node_path = td.path().join("repo");
        let op = Operation::Git("https://github.com/octocat/Hello-World.git".into());
        op.execute(&node_path, false).unwrap();
        assert!(node_path.join(".git").is_dir());
    }
}
