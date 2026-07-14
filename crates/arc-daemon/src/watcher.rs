use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::time::{Duration, Instant};

use anyhow::Result;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::UnboundedSender;

const DEBOUNCE_WINDOW: Duration = Duration::from_millis(500);

#[derive(Debug, Clone)]
pub enum WatchEvent {
    Changed(PathBuf),
    Removed(PathBuf),
}

/// Spawn a blocking watcher thread that emits debounced `WatchEvent`s for
/// `*.md` files under `notes_dir` (recursive) into `tx`.
///
/// Returns the `RecommendedWatcher` — the caller must keep it alive for the
/// duration of the program; dropping it stops the watch.
pub fn spawn(
    notes_dir: PathBuf,
    tx: UnboundedSender<WatchEvent>,
) -> Result<RecommendedWatcher> {
    let (notify_tx, notify_rx) = std_mpsc::channel();
    let mut watcher = notify::recommended_watcher(notify_tx)?;
    watcher.watch(&notes_dir, RecursiveMode::Recursive)?;

    std::thread::spawn(move || debounce_loop(notify_rx, tx));

    Ok(watcher)
}

fn is_markdown(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}

fn debounce_loop(
    rx: std_mpsc::Receiver<notify::Result<notify::Event>>,
    tx: UnboundedSender<WatchEvent>,
) {
    let mut pending: HashMap<PathBuf, (Instant, bool /* removed */)> = HashMap::new();
    loop {
        // Block briefly for the first event, then drain.
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(ev)) => ingest(&ev, &mut pending),
            Ok(Err(_)) => {}
            Err(std_mpsc::RecvTimeoutError::Timeout) => {}
            Err(std_mpsc::RecvTimeoutError::Disconnected) => return,
        }
        // Drain additional events without blocking.
        while let Ok(maybe_ev) = rx.try_recv() {
            if let Ok(ev) = maybe_ev {
                ingest(&ev, &mut pending);
            }
        }
        // Emit any entries past the debounce window.
        let now = Instant::now();
        let mut to_emit: Vec<(PathBuf, bool)> = Vec::new();
        pending.retain(|path, (at, removed)| {
            if now.duration_since(*at) >= DEBOUNCE_WINDOW {
                to_emit.push((path.clone(), *removed));
                false
            } else {
                true
            }
        });
        for (path, removed) in to_emit {
            let ev = if removed {
                WatchEvent::Removed(path)
            } else {
                WatchEvent::Changed(path)
            };
            if tx.send(ev).is_err() {
                return;
            }
        }
    }
}

fn ingest(ev: &notify::Event, pending: &mut HashMap<PathBuf, (Instant, bool)>) {
    let removed = matches!(ev.kind, EventKind::Remove(_));
    for path in &ev.paths {
        if !is_markdown(path) {
            continue;
        }
        pending.insert(path.clone(), (Instant::now(), removed));
    }
}
