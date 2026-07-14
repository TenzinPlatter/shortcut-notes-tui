use std::process::Output;

use tokio::process::Command;

use arc_core::dbg_file;

fn error_on_command_fail(output: &Output) -> anyhow::Result<()> {
    if !output.status.success() {
        Err(anyhow::anyhow!(
            "Zellij command failed with error: {:?}",
            String::from_utf8_lossy(&output.stderr)
        ))
    } else {
        Ok(())
    }
}

pub async fn session_exists(name: &str) -> anyhow::Result<bool> {
    let output = Command::new("zellij")
        .arg("list-sessions")
        .output()
        .await?;

    // zellij list-sessions exits non-zero when no sessions exist; treat that as no sessions
    if !output.status.success() {
        return Ok(false);
    }

    let output = String::from_utf8_lossy(&output.stdout);

    // Each line is like: "session-name [Created ...] (current)" — match on first word
    let exists = output
        .lines()
        .any(|line| line.split_whitespace().next() == Some(name));

    dbg_file!("Found zellij sessions, '{}' exists: {}", name, exists);

    Ok(exists)
}

pub async fn session_attach(name: &str) -> anyhow::Result<()> {
    let output = Command::new("zellij")
        .arg("attach")
        .arg(name)
        .output()
        .await?;

    error_on_command_fail(&output)?;

    Ok(())
}

pub async fn session_create(name: &str) -> anyhow::Result<()> {
    // `--create-background` creates a detached session without attaching (equivalent to `tmux new-session -d`)
    let output = Command::new("zellij")
        .arg("attach")
        .arg("--create-background")
        .arg(name)
        .output()
        .await?;

    error_on_command_fail(&output)?;

    Ok(())
}
