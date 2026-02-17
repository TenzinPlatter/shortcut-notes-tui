use std::{
    io::Write,
    path::Path,
    process::{Command as StdCommand, Stdio},
};

use anyhow::Context;
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::DefaultTerminal;
use slugify::slugify;
use tokio::process::Command as TokioCommand;
use which::which;

use crate::{config::Config, dbg_file};

pub fn check_worktree_dependencies() -> anyhow::Result<()> {
    which("fd").context("Please make sure 'fd' is installed and in $PATH")?;
    which("fzf").context("Please make sure 'fzf' is installed and in $PATH")?;
    Ok(())
}

/// Returns a string with newline seperated repo names
pub async fn get_repo_list(config: &Config) -> anyhow::Result<String> {
    let fd_output = TokioCommand::new("fd")
        .args([
            "-t",
            "d",
            "-I",
            "-H",
            "^\\.git$",
            config.repositories_directory.to_str().unwrap(),
            "-x",
            "dirname",
            "{}",
        ])
        .output()
        .await?;

    if !fd_output.status.success() {
        anyhow::bail!(
            "fd failed with stderr: {}",
            String::from_utf8_lossy(fd_output.stderr.as_slice())
        );
    }

    let stdout = String::from_utf8_lossy(&fd_output.stdout).to_string();
    Ok(stdout
        .lines()
        .map(|line| {
            line.replace(
                &format!("{}/", config.repositories_directory.to_str().unwrap()),
                "",
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

pub fn select_repo_with_fzf(
    repo_list: &str,
    terminal: &mut DefaultTerminal,
) -> anyhow::Result<String> {
    std::io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    let mut fzf = StdCommand::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn fzf")?;

    fzf.stdin
        .take()
        .context("failed to open stdin")?
        .write_all(repo_list.as_bytes())?;

    let output = fzf.wait_with_output()?;

    std::io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;

    if !output.status.success() {
        anyhow::bail!("fzf was cancelled or failed");
    }

    Ok(String::from_utf8(output.stdout)
        .context("fzf output was not valid UTF-8")?
        .trim()
        .to_string())
}

pub async fn create_worktree(repo_path: &Path, branch_name: &str) -> anyhow::Result<()> {
    let slug = slugify!(branch_name);
    let worktree_path = format!(".worktrees/{}", slug);

    dbg_file!("repo path {}", repo_path.display());
    TokioCommand::new("mkdir")
        .args(["-p", ".worktrees"])
        .current_dir(repo_path)
        .status()
        .await?;

    if !branch_exists(repo_path, branch_name).await {
        create_branch(repo_path, branch_name).await?;
    }

    let output = TokioCommand::new("git")
        .args(["worktree", "add", &worktree_path, branch_name])
        .current_dir(repo_path)
        .output()
        .await
        .with_context(|| {
            format!(
                "failed to run git worktree add {} {}",
                worktree_path, branch_name
            )
        })?;

    if !output.status.success() {
        anyhow::bail!(
            "git worktree add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

async fn branch_exists(repo_path: &Path, branch_name: &str) -> bool {
    let result = TokioCommand::new("git")
        .args([
            "rev-parse",
            "--verify",
            &format!("refs/heads/{}", branch_name),
        ])
        .current_dir(repo_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    result.map(|s| s.success()).unwrap_or(false)
}

async fn create_branch(repo_path: &Path, branch_name: &str) -> anyhow::Result<()> {
    let status = TokioCommand::new("git")
        .args(["branch", branch_name])
        .current_dir(repo_path)
        .status()
        .await;

    let success = status.map(|s| s.success()).unwrap_or(false);

    if success {
        Ok(())
    } else {
        anyhow::bail!(
            "Failed to create branch {} for repo at {}",
            branch_name,
            repo_path.display()
        )
    }
}
