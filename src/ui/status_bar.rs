//! Status bar — top line (branch info) and bottom line (keybinding help).

use crate::app::{App, View};
use crate::keys;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Top bar: current branch, ahead/behind.
pub fn draw_top(f: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    spans.push(Span::styled(
        " 🌿 ",
        Style::default().fg(Color::Green),
    ));

    spans.push(Span::styled(
        &app.current_branch,
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    ));

    // Find current branch ahead/behind.
    if let Some(branch) = app.branches.iter().find(|b| b.is_head) {
        if branch.ahead > 0 {
            spans.push(Span::styled(
                format!(" ↑{}", branch.ahead),
                Style::default().fg(Color::Green),
            ));
        }
        if branch.behind > 0 {
            spans.push(Span::styled(
                format!(" ↓{}", branch.behind),
                Style::default().fg(Color::Red),
            ));
        }
    }

    // Status message on the right side.
    if !app.status_msg.is_empty() {
        let pad = area
            .width
            .saturating_sub(
                spans.iter().map(|s| s.content.len() as u16).sum::<u16>()
                    + app.status_msg.len() as u16
                    + 1,
            );
        spans.push(Span::styled(
            " ".repeat(pad as usize),
            Style::default(),
        ));
        spans.push(Span::styled(
            &app.status_msg,
            Style::default().fg(Color::Yellow),
        ));
    }

    let line = Line::from(spans);
    let bar = Paragraph::new(line).style(Style::default().bg(Color::Rgb(30, 30, 46)));
    f.render_widget(bar, area);
}

/// Bottom bar: contextual keybinding help.
pub fn draw_bottom(f: &mut Frame, app: &App, area: Rect) {
    let help = match app.view {
        View::Graph => keys::GRAPH_HELP,
        View::DiffView => keys::DIFF_HELP,
        View::BranchList => keys::BRANCH_HELP,
    };

    let bar = Paragraph::new(Line::from(Span::styled(
        help,
        Style::default().fg(Color::DarkGray),
    )))
    .style(Style::default().bg(Color::Rgb(30, 30, 46)));

    f.render_widget(bar, area);
}
