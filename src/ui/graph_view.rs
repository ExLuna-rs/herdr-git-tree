//! Graph view — renders the commit graph as a scrollable list.

use crate::app::App;
use crate::git::graph::{RefKind, RefLabel};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Branch-lane colours, cycled by column index.
const LANE_COLORS: &[Color] = &[
    Color::Blue,
    Color::Green,
    Color::Yellow,
    Color::Magenta,
    Color::Cyan,
    Color::Red,
    Color::LightBlue,
    Color::LightGreen,
];

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let visible_lines = area.height.saturating_sub(2) as usize; // minus border
    let lines_per_commit = 2usize;
    let visible_commits = visible_lines / lines_per_commit;

    // Adjust scroll so the selected commit is visible.
    let scroll_offset = compute_scroll(app.selected, app.scroll_offset, visible_commits);

    let mut lines: Vec<Line> = Vec::new();

    let end = (scroll_offset + visible_commits).min(app.commits.len());
    for idx in scroll_offset..end {
        let commit = &app.commits[idx];
        let is_selected = idx == app.selected;
        let bg = if is_selected {
            Color::DarkGray
        } else {
            Color::Reset
        };

        // Line 1: graph + hash + refs + message
        let mut spans: Vec<Span> = Vec::new();

        // Graph prefix — colour each column.
        for (col_idx, ch) in commit.graph_line.chars().enumerate() {
            let col_color = if ch == '*' {
                LANE_COLORS[commit.column % LANE_COLORS.len()]
            } else if ch == '│' {
                LANE_COLORS[(col_idx / 2) % LANE_COLORS.len()]
            } else {
                Color::DarkGray
            };
            let style = if ch == '*' {
                Style::default()
                    .fg(col_color)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(col_color).bg(bg)
            };
            spans.push(Span::styled(ch.to_string(), style));
        }

        spans.push(Span::styled(" ", Style::default().bg(bg)));

        // Short hash.
        spans.push(Span::styled(
            &commit.short_hash,
            Style::default()
                .fg(Color::Yellow)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        ));

        // Ref labels.
        if !commit.refs.is_empty() {
            spans.push(Span::styled(" ", Style::default().bg(bg)));
            for (i, label) in commit.refs.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::styled(", ", Style::default().fg(Color::DarkGray).bg(bg)));
                }
                spans.push(render_ref_label(label, bg));
            }
        }

        // Commit message.
        spans.push(Span::styled(" ", Style::default().bg(bg)));
        spans.push(Span::styled(
            &commit.message,
            Style::default().fg(Color::White).bg(bg),
        ));

        lines.push(Line::from(spans));

        // Line 2: author + time (indented under graph).
        let indent = " ".repeat(commit.graph_line.len() + 1);
        let time_str = relative_time(commit.time);
        lines.push(Line::from(vec![
            Span::styled(indent, Style::default().bg(bg)),
            Span::styled(
                format!("{} · {}", commit.author, time_str),
                Style::default().fg(Color::DarkGray).bg(bg),
            ),
        ]));
    }

    let block = Block::default()
        .borders(Borders::NONE)
        .title(format!(
            " Git Tree — {} commits ",
            app.commits.len()
        ))
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_ref_label(label: &RefLabel, bg: Color) -> Span<'static> {
    let (fg, prefix) = match label.kind {
        RefKind::Head => (Color::Cyan, "→ "),
        RefKind::LocalBranch => (Color::Green, ""),
        RefKind::RemoteBranch => (Color::Red, ""),
        RefKind::Tag => (Color::Yellow, "🏷 "),
    };

    Span::styled(
        format!("({}{})", prefix, label.name),
        Style::default()
            .fg(fg)
            .bg(bg)
            .add_modifier(Modifier::BOLD),
    )
}

fn compute_scroll(selected: usize, current_scroll: usize, visible: usize) -> usize {
    if visible == 0 {
        return 0;
    }
    if selected < current_scroll {
        selected
    } else if selected >= current_scroll + visible {
        selected - visible + 1
    } else {
        current_scroll
    }
}

/// Convert a Unix timestamp to a human-readable relative time string.
fn relative_time(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else if diff < 2592000 {
        format!("{}w ago", diff / 604800)
    } else {
        format!("{}mo ago", diff / 2592000)
    }
}
