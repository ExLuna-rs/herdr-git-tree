//! Branch list view — lists local and remote branches with actions.

use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    if app.branches.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No branches found",
            Style::default().fg(Color::DarkGray),
        )));
    }

    for (idx, branch) in app.branches.iter().enumerate() {
        let is_selected = idx == app.branch_selected;
        let bg = if is_selected {
            Color::DarkGray
        } else {
            Color::Reset
        };

        let mut spans: Vec<Span> = Vec::new();

        // Current branch indicator.
        if branch.is_head {
            spans.push(Span::styled(
                " * ",
                Style::default()
                    .fg(Color::Green)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled("   ", Style::default().bg(bg)));
        }

        // Branch name.
        let name_color = if branch.is_remote {
            Color::Red
        } else if branch.is_head {
            Color::Green
        } else {
            Color::White
        };

        spans.push(Span::styled(
            &branch.name,
            Style::default()
                .fg(name_color)
                .bg(bg)
                .add_modifier(if branch.is_head {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ));

        // Ahead/behind info.
        if branch.ahead > 0 || branch.behind > 0 {
            spans.push(Span::styled(" ", Style::default().bg(bg)));

            if branch.ahead > 0 {
                spans.push(Span::styled(
                    format!("↑{}", branch.ahead),
                    Style::default().fg(Color::Green).bg(bg),
                ));
            }
            if branch.behind > 0 {
                if branch.ahead > 0 {
                    spans.push(Span::styled(" ", Style::default().bg(bg)));
                }
                spans.push(Span::styled(
                    format!("↓{}", branch.behind),
                    Style::default().fg(Color::Red).bg(bg),
                ));
            }
        }

        // Upstream.
        if let Some(ref upstream) = branch.upstream {
            spans.push(Span::styled(
                format!(" → {}", upstream),
                Style::default().fg(Color::DarkGray).bg(bg),
            ));
        }

        lines.push(Line::from(spans));
    }

    let block = Block::default()
        .borders(Borders::NONE)
        .title(format!(" Branches ({}) ", app.branches.len()))
        .title_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        );

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
