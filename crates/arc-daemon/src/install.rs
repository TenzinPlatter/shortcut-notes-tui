use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

const UNIT_NAME: &str = "arc.service";

pub fn install() -> Result<()> {
    let exe = std::env::current_exe().context("current_exe")?;
    let unit_path = unit_path()?;
    if let Some(p) = unit_path.parent() {
        std::fs::create_dir_all(p)?;
    }

    let unit = format!(
        r#"[Unit]
Description=arc todo-extraction daemon
After=default.target

[Service]
ExecStart={} daemon run
Restart=on-failure
RestartSec=5
Environment=ARC_LOG=info

[Install]
WantedBy=default.target
"#,
        exe.display()
    );

    let mut f = std::fs::File::create(&unit_path)
        .with_context(|| format!("write {}", unit_path.display()))?;
    f.write_all(unit.as_bytes())?;

    run("systemctl", &["--user", "daemon-reload"])?;
    run("systemctl", &["--user", "enable", "--now", UNIT_NAME])?;
    println!(
        "Installed {} and enabled the user service.",
        unit_path.display()
    );
    Ok(())
}

fn unit_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME unset")?;
    Ok(PathBuf::from(home)
        .join(".config/systemd/user")
        .join(UNIT_NAME))
}

fn run(prog: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(prog).args(args).status()?;
    if !status.success() {
        anyhow::bail!("{} {} failed", prog, args.join(" "));
    }
    Ok(())
}
