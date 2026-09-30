//! herdr-git-tree — Interactive git graph TUI for herdr.
//!
//! Entry point: sets up the terminal, detects the git repo, and runs the app.

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
