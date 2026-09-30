//! herdr-git-tree — Interactive git graph TUI for herdr.
//!
//! Entry point: sets up the terminal, detects the git repo, and runs the app.
//! Subcommands:
//!   (none)   — run the TUI directly
//!   toggle   — toggle the sidebar pane open/close via herdr plugin pane API

mod app;
mod git;
mod keys;
mod ui;

use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let subcommand = args.get(1).map(|s| s.as_str());

    match subcommand {
        Some("toggle") => return cmd_toggle(),
        _ => {} // Run the TUI
    }

    // Install panic hook that restores the terminal before printing the panic.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture);
        default_hook(info);
    }));

    // Detect repo path: prefer HERDR_ACTIVE_PANE_CWD, then current dir.
    let cwd = std::env::var("HERDR_ACTIVE_PANE_CWD")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let repo = git2::Repository::discover(&cwd).map_err(|e| {
        format!(
            "Not a git repository (searched from {}): {}",
            cwd.display(),
            e
        )
    })?;

    let repo_path = repo
        .workdir()
        .unwrap_or_else(|| repo.path())
        .to_path_buf();

    // Terminal setup.
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // Run the application.
    let mut app = App::new(repo_path)?;
    let result = app.run(&mut terminal);

    // Restore terminal.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

/// Toggle the git-tree sidebar pane open/close.
///
/// Checks if a git-tree plugin pane is already open in the current workspace.
/// If yes, closes it. If no, opens one as a split to the right.
fn cmd_toggle() -> Result<(), Box<dyn std::error::Error>> {
    let herdr = std::env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".to_string());

    // List panes in current workspace to find an existing git-tree pane
    let workspace_id = std::env::var("HERDR_WORKSPACE_ID")
        .or_else(|_| std::env::var("HERDR_ACTIVE_WORKSPACE_ID"))
        .unwrap_or_default();

    if !workspace_id.is_empty() {
        // Check if a git-tree pane already exists by looking at pane list
        let output = std::process::Command::new(&herdr)
            .args(["pane", "list", "--workspace", &workspace_id])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(panes) = json.pointer("/result/panes").and_then(|v| v.as_array()) {
                    // Find a pane that's running herdr-git-tree
                    for pane in panes {
                        let title = pane.pointer("/terminal_title_stripped")
                            .or_else(|| pane.pointer("/terminal_title"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let label = pane.pointer("/label")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        if title.contains("git-tree") || title.contains("Git Tree")
                            || label.contains("Git Tree") {
                            // Found it — close it
                            if let Some(pane_id) = pane.pointer("/pane_id").and_then(|v| v.as_str()) {
                                let _ = std::process::Command::new(&herdr)
                                    .args(["pane", "close", pane_id])
                                    .output();
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }

    // Not found — open a new git-tree sidebar pane
    let entrypoint = if cfg!(windows) { "tree-windows" } else { "tree" };
    let _ = std::process::Command::new(&herdr)
        .args([
            "plugin", "pane", "open",
            "--plugin", "git-tree",
            "--entrypoint", entrypoint,
            "--placement", "split",
            "--direction", "right",
        ])
        .output();

    Ok(())
}
