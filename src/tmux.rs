use std::{env, process::Output};

use tokio::process::Command;

use crate::dbg_file;

fn error_on_command_fail(output: &Output) -> anyhow::Result<()> {
    if !output.status.success() {
        Err(anyhow::anyhow!(
            "Tmux command failed with error: {:?}",
            String::from_utf8_lossy(&output.stderr)
        ))
    } else {
        Ok(())
    }
}

pub async fn session_exists(name: &str) -> anyhow::Result<bool> {
    let output = Command::new("tmux")
        .arg("list-sessions")
        .arg("-F")
        .arg("#{session_name}")
        .output()
        .await?;

    error_on_command_fail(&output)?;

    let output = String::from_utf8_lossy(&output.stdout);
    let session_names: Vec<_> = output.lines().map(|line| line.trim()).collect();

    dbg_file!("Found sessions: {:?}", session_names);

    Ok(session_names.contains(&name))
}

pub fn attatched_to_session() -> bool {
    env::var("TMUX").is_ok()
}

pub async fn session_attach(name: &str) -> anyhow::Result<()> {
    let command = if attatched_to_session() {
        "switch-client"
    } else {
        "attach-session"
    };

    let output = Command::new("tmux")
        .arg(command)
        .arg("-t")
        .arg(name)
        .output()
        .await?;

    error_on_command_fail(&output)?;

    Ok(())
}

pub async fn session_detach() -> anyhow::Result<()> {
    let output = Command::new("tmux").arg("detach").output().await?;
    error_on_command_fail(&output)?;
    Ok(())
}

pub async fn session_create(name: &str) -> anyhow::Result<()> {
    let output = Command::new("tmux")
        .arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(name)
        .output()
        .await?;

    error_on_command_fail(&output)?;

    Ok(())
}
