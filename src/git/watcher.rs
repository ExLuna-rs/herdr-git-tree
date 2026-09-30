//! File system watcher — monitors the repo for changes and sends refresh
//! signals through an mpsc channel with debouncing.

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

/// Start watching the repository for changes.
///
/// Returns the watcher (must be kept alive) and a receiver that gets a `()`
/// message whenever the repo changed (debounced to 300ms).
pub fn start_watcher(
    repo_path: &Path,
) -> Result<(RecommendedWatcher, Receiver<()>), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<()>();

    // Debounce wrapper: only forward a signal after 300ms of quiet.
    let debounce_tx = tx.clone();
    let (raw_tx, raw_rx) = mpsc::channel::<()>();

    std::thread::spawn(move || {
        let debounce = Duration::from_millis(300);
        let mut last_event = Instant::now() - debounce;

        loop {
            match raw_rx.recv_timeout(debounce) {
                Ok(()) => {
                    last_event = Instant::now();
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if last_event.elapsed() >= debounce
                        && last_event.elapsed() < debounce + Duration::from_millis(350)
                    {
                        let _ = debounce_tx.send(());
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    let event_tx: Sender<()> = raw_tx;
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            // Only react to meaningful changes.
            use notify::EventKind;
            match event.kind {
                EventKind::Create(_)
                | EventKind::Modify(_)
                | EventKind::Remove(_) => {
                    let _ = event_tx.send(());
                }
                _ => {}
            }
        }
    })?;

    // Watch .git directory for ref/HEAD changes.
    let git_dir = repo_path.join(".git");
    if git_dir.is_dir() {
        watcher.watch(&git_dir, RecursiveMode::Recursive)?;
    }

    // Watch working tree (non-recursive for performance — catches top-level changes;
    // nested changes are picked up via .git/index updates).
    watcher.watch(repo_path, RecursiveMode::NonRecursive)?;

    Ok((watcher, rx))
}
