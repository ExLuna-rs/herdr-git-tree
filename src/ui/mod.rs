pub mod branch_list;
pub mod commit_detail;
pub mod diff_view;
pub mod graph_view;
pub mod status_bar;

use crate::app::{App, View};
use ratatui::Frame;

/// Top-level draw dispatcher.
pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    match app.view {
        View::Graph => graph_view::draw(f, app, area),
        View::DiffView => diff_view::draw(f, app, area),
        View::BranchList => branch_list::draw(f, app, area),
    }
}
