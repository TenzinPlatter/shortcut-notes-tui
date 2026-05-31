pub mod install;
pub mod scanner;
pub mod scheduler;
pub mod store;
pub mod watcher;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use chrono::{Datelike, Local, TimeZone};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::daemon::scheduler::{DBusNotifier, ScheduledTodo, Scheduler, SystemClock};
use crate::daemon::watcher::WatchEvent;
use crate::todos::TodoSource;

pub async fn run(config: Config) -> Result<()> {
    init_tracing();
    info!("arc daemon starting; notes_dir={}", config.notes_dir.display());

    if !config.notes_dir.exists() {
        info!("notes_dir {} missing, creating", config.notes_dir.display());
        std::fs::create_dir_all(&config.notes_dir)?;
    }

    // Bootstrap: full scan.
    bootstrap_scan(&config.notes_dir, &config.cache_dir).await?;

    // Spawn the watcher.
    let (tx, mut rx) = mpsc::unbounded_channel::<WatchEvent>();
    let _watcher = watcher::spawn(config.notes_dir.clone(), tx)?;

    // Scheduler.
    let clock = Arc::new(SystemClock);
    let notifier = Arc::new(DBusNotifier);
    let mut sched = Scheduler::new(clock, notifier);
    rebuild_schedule(&config.cache_dir, &mut sched).await?;
    // First tick handles overdue batching.
    sched.tick();

    // Main loop.
    loop {
        let sleep_for = match sched.next_fire() {
            Some(at) => {
                let now = Local::now();
                let dur = (at - now).to_std().unwrap_or(std::time::Duration::ZERO);
                dur.min(std::time::Duration::from_secs(60))
            }
            None => std::time::Duration::from_secs(60),
        };

        tokio::select! {
            event = rx.recv() => {
                let Some(event) = event else {
                    warn!("watcher channel closed; exiting");
                    return Ok(());
                };
                if let Err(e) = handle_event(&config, &mut sched, event).await {
                    error!("event handling failed: {e:#}");
                }
            }
            _ = tokio::time::sleep(sleep_for) => {
                sched.tick();
            }
        }
    }
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_env("ARC_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

async fn bootstrap_scan(notes_dir: &Path, cache_dir: &Path) -> Result<()> {
    let files = list_markdown(notes_dir)?;
    info!("bootstrap: scanning {} note files", files.len());
    for abs in files {
        let rel = abs.strip_prefix(notes_dir).unwrap_or(&abs).to_path_buf();
        let content = match std::fs::read_to_string(&abs) {
            Ok(c) => c,
            Err(e) => {
                warn!("bootstrap read {} failed: {e}", abs.display());
                continue;
            }
        };
        let parsed = scanner::parse_note(&content, &rel);
        if let Err(e) = store::merge_file(cache_dir, &rel, parsed).await {
            warn!("merge {} failed: {e:#}", rel.display());
        }
    }
    Ok(())
}

fn list_markdown(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("md"))
            {
                out.push(p);
            }
        }
    }
    Ok(out)
}

async fn handle_event(
    config: &Config,
    sched: &mut Scheduler,
    event: WatchEvent,
) -> Result<()> {
    let (abs, removed) = match event {
        WatchEvent::Changed(p) => (p, false),
        WatchEvent::Removed(p) => (p, true),
    };
    let rel = abs
        .strip_prefix(&config.notes_dir)
        .unwrap_or(&abs)
        .to_path_buf();

    let treat_as_removed = removed
        || matches!(
            std::fs::symlink_metadata(&abs),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound
        );

    if treat_as_removed {
        let _ = store::drop_file(&config.cache_dir, &rel).await?;
    } else {
        let content = match std::fs::read_to_string(&abs) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Raced with delete/rename — treat as removal.
                let _ = store::drop_file(&config.cache_dir, &rel).await?;
                rebuild_schedule(&config.cache_dir, sched).await?;
                return Ok(());
            }
            Err(e) => {
                warn!("read {} failed: {e}", abs.display());
                return Ok(());
            }
        };
        let parsed = scanner::parse_note(&content, &rel);
        let _ = store::merge_file(&config.cache_dir, &rel, parsed).await?;
    }
    rebuild_schedule(&config.cache_dir, sched).await?;
    Ok(())
}

async fn rebuild_schedule(cache_dir: &Path, sched: &mut Scheduler) -> Result<()> {
    let todos = crate::todos::load_todos(cache_dir).await;
    // Cancel everything we know about, then re-insert.
    sched.clear();
    for t in todos {
        if t.completed {
            continue;
        }
        let Some(date) = t.date else { continue };
        let fire_at = Local
            .with_ymd_and_hms(date.year(), date.month(), date.day(), 9, 0, 0)
            .single()
            .unwrap_or_else(Local::now);
        let title = match &t.source {
            TodoSource::NoteParsed { file, .. } => file
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| format!("arc — {}", s))
                .unwrap_or_else(|| "arc".to_string()),
            TodoSource::Manual => "arc".to_string(),
        };
        sched.upsert(ScheduledTodo {
            id: t.id,
            title,
            body: t.text.clone(),
            fire_at,
        });
    }
    Ok(())
}
