//! Application state, event loop, and view dispatch.

use crate::git::{branch::BranchInfo, diff::FileDiff, graph::CommitNode, watcher};
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

/// Which view is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Graph,
    DiffView,
    BranchList,
}

/// Application state.
pub struct App {
    pub repo_path: PathBuf,
    pub view: View,
    pub should_quit: bool,

    // Graph state
    pub commits: Vec<CommitNode>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub max_commits: usize,

    // Diff state
    pub diff_files: Vec<FileDiff>,
    pub diff_scroll: usize,
    pub diff_commit_hash: String,

    // Branch state
    pub branches: Vec<BranchInfo>,
    pub branch_selected: usize,
    pub current_branch: String,

    // File watcher
    watcher_rx: Receiver<()>,
    _watcher: notify::RecommendedWatcher,

    // Status message
    pub status_msg: String,
}

impl App {
    pub fn new(repo_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let (watcher_instance, watcher_rx) = watcher::start_watcher(&repo_path)?;

        let mut app = App {
            repo_path,
            view: View::Graph,
            should_quit: false,
            commits: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            max_commits: 200,
            diff_files: Vec::new(),
            diff_scroll: 0,
            diff_commit_hash: String::new(),
            branches: Vec::new(),
            branch_selected: 0,
            current_branch: String::new(),
            watcher_rx,
            _watcher: watcher_instance,
            status_msg: String::new(),
        };

        app.refresh_graph();
        app.refresh_branches();

        Ok(app)
    }

    fn open_repo(&self) -> Result<git2::Repository, String> {
        git2::Repository::open(&self.repo_path).map_err(|e| format!("Cannot open repo: {}", e))
    }

    pub fn refresh_graph(&mut self) {
        match self.open_repo() {
            Ok(repo) => {
                self.commits =
                    crate::git::graph::build_graph(&repo, self.max_commits).unwrap_or_default();
                if self.selected >= self.commits.len() && !self.commits.is_empty() {
                    self.selected = self.commits.len() - 1;
                }
            }
            Err(e) => {
                self.status_msg = e;
            }
        }
    }

    pub fn refresh_branches(&mut self) {
        match self.open_repo() {
            Ok(repo) => {
                self.branches = crate::git::branch::list_branches(&repo).unwrap_or_default();
                self.current_branch = crate::git::branch::current_branch_name(&repo)
                    .unwrap_or_else(|| "HEAD".to_string());
            }
            Err(e) => {
                self.status_msg = e;
            }
        }
    }

    /// Main event loop.
    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stderr>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        loop {
            // Update scroll offset before drawing so mouse clicks stay in sync.
            {
                let size = terminal.size()?;
                // visible area = total height - 6 (4 header + 2 footer)
                let content_lines = size.height.saturating_sub(6) as usize;
                let visible_commits = content_lines / 2; // 2 lines per commit
                if visible_commits > 0 {
                    if self.selected < self.scroll_offset {
                        self.scroll_offset = self.selected;
                    } else if self.selected >= self.scroll_offset + visible_commits {
                        self.scroll_offset = self.selected - visible_commits + 1;
                    }
                }
            }

            terminal.draw(|f| ui::draw(f, self))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => {
                        // Only handle key press, ignore release/repeat
                        if key.kind == crossterm::event::KeyEventKind::Press {
                            self.on_key(key.code, key.modifiers);
                        }
                    }
                    Event::Mouse(mouse) => self.on_mouse(mouse.kind, mouse.row),
                    Event::Resize(_, _) => {} // ratatui handles this
                    _ => {}
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    fn on_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        // Global keys
        if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return;
        }

        match self.view {
            View::Graph => self.on_key_graph(code),
            View::DiffView => self.on_key_diff(code),
            View::BranchList => self.on_key_branch(code),
        }
    }

    fn on_key_graph(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('J') | KeyCode::PageDown => self.move_selection(20),
            KeyCode::Char('K') | KeyCode::PageUp => self.move_selection(-20),
            KeyCode::Char('g') | KeyCode::Home => {
                self.selected = 0;
                self.scroll_offset = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                if !self.commits.is_empty() {
                    self.selected = self.commits.len() - 1;
                }
            }
            KeyCode::Enter => self.open_diff(),
            KeyCode::Char('b') => {
                self.refresh_branches();
                self.branch_selected = 0;
                self.view = View::BranchList;
            }
            KeyCode::Char('r') => {
                self.refresh_graph();
                self.refresh_branches();
                self.status_msg = "Refreshed".to_string();
            }
            _ => {}
        }
    }

    fn on_key_diff(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => self.view = View::Graph,
            KeyCode::Char('j') | KeyCode::Down => self.diff_scroll = self.diff_scroll.saturating_add(1),
            KeyCode::Char('k') | KeyCode::Up => self.diff_scroll = self.diff_scroll.saturating_sub(1),
            KeyCode::Char('J') | KeyCode::PageDown => self.diff_scroll = self.diff_scroll.saturating_add(20),
            KeyCode::Char('K') | KeyCode::PageUp => self.diff_scroll = self.diff_scroll.saturating_sub(20),
            KeyCode::Home => self.diff_scroll = 0,
            _ => {}
        }
    }

    fn on_key_branch(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => self.view = View::Graph,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.branch_selected + 1 < self.branches.len() {
                    self.branch_selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.branch_selected = self.branch_selected.saturating_sub(1);
            }
            KeyCode::Enter => self.checkout_selected_branch(),
            KeyCode::Char('d') => self.delete_selected_branch(),
            _ => {}
        }
    }

    fn on_mouse(&mut self, kind: MouseEventKind, row: u16) {
        match self.view {
            View::Graph => match kind {
                MouseEventKind::ScrollDown => self.move_selection(3),
                MouseEventKind::ScrollUp => self.move_selection(-3),
                MouseEventKind::Down(_) => {
                    // Each commit takes 2 rows. Header is 4 rows.
                    let graph_row = row.saturating_sub(4) as usize;
                    let commit_idx = self.scroll_offset + graph_row / 2;
                    if commit_idx < self.commits.len() {
                        self.selected = commit_idx;
                    }
                }
                _ => {}
            },
            View::DiffView => match kind {
                MouseEventKind::ScrollDown => self.diff_scroll = self.diff_scroll.saturating_add(3),
                MouseEventKind::ScrollUp => self.diff_scroll = self.diff_scroll.saturating_sub(3),
                _ => {}
            },
            View::BranchList => match kind {
                MouseEventKind::ScrollDown => {
                    if self.branch_selected + 1 < self.branches.len() {
                        self.branch_selected += 1;
                    }
                }
                MouseEventKind::ScrollUp => {
                    self.branch_selected = self.branch_selected.saturating_sub(1);
                }
                _ => {}
            },
        }
    }

    fn on_tick(&mut self) {
        // Check if the file watcher signalled changes.
        if self.watcher_rx.try_recv().is_ok() {
            // Drain any extra signals.
            while self.watcher_rx.try_recv().is_ok() {}
            self.refresh_graph();
            self.refresh_branches();
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.commits.is_empty() {
            return;
        }
        let new = if delta < 0 {
            self.selected.saturating_sub(delta.unsigned_abs() as usize)
        } else {
            (self.selected + delta as usize).min(self.commits.len() - 1)
        };

        // Load more commits if we're near the end.
        if new >= self.commits.len().saturating_sub(20) && self.commits.len() >= self.max_commits {
            self.max_commits += 200;
            self.refresh_graph();
        }

        self.selected = new;
    }

    fn open_diff(&mut self) {
        if let Some(commit) = self.commits.get(self.selected) {
            let oid = commit.oid;
            match self.open_repo() {
                Ok(repo) => {
                    self.diff_files =
                        crate::git::diff::get_commit_diff(&repo, oid).unwrap_or_default();
                    self.diff_commit_hash = commit.short_hash.clone();
                    self.diff_scroll = 0;
                    self.view = View::DiffView;
                }
                Err(e) => self.status_msg = e,
            }
        }
    }

    fn checkout_selected_branch(&mut self) {
        if let Some(branch) = self.branches.get(self.branch_selected) {
            if branch.is_remote {
                self.status_msg = "Cannot checkout a remote branch directly".to_string();
                return;
            }
            let name = branch.name.clone();
            match self.open_repo() {
                Ok(repo) => match crate::git::branch::switch_branch(&repo, &name) {
                    Ok(()) => {
                        self.status_msg = format!("Switched to {}", name);
                        self.view = View::Graph;
                        self.refresh_graph();
                        self.refresh_branches();
                    }
                    Err(e) => self.status_msg = format!("Checkout failed: {}", e),
                },
                Err(e) => self.status_msg = e,
            }
        }
    }

    fn delete_selected_branch(&mut self) {
        if let Some(branch) = self.branches.get(self.branch_selected) {
            if branch.is_head {
                self.status_msg = "Cannot delete the current branch".to_string();
                return;
            }
            if branch.is_remote {
                self.status_msg = "Cannot delete remote branches from here".to_string();
                return;
            }
            let name = branch.name.clone();
            match self.open_repo() {
                Ok(repo) => match crate::git::branch::delete_branch(&repo, &name) {
                    Ok(()) => {
                        self.status_msg = format!("Deleted branch {}", name);
                        self.refresh_branches();
                        if self.branch_selected >= self.branches.len()
                            && !self.branches.is_empty()
                        {
                            self.branch_selected = self.branches.len() - 1;
                        }
                    }
                    Err(e) => self.status_msg = format!("Delete failed: {}", e),
                },
                Err(e) => self.status_msg = e,
            }
        }
    }
}
