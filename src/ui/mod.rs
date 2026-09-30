pub mod branch_list;
pub mod commit_detail;
pub mod diff_view;
pub mod graph_view;
pub mod status_bar;

use crate::app::{App, View};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

/// Top-level draw dispatcher.
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // status bar top
            Constraint::Min(1),   // main content
            Constraint::Length(1), // help bar bottom
        ])
        .split(f.area());

    status_bar::draw_top(f, app, chunks[0]);

    match app.view {
        View::Graph => graph_view::draw(f, app, chunks[1]),
        View::DiffView => diff_view::draw(f, app, chunks[1]),
        View::BranchList => branch_list::draw(f, app, chunks[1]),
    }

    status_bar::draw_bottom(f, app, chunks[2]);
}
