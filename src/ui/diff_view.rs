//! Diff view — scrollable rendering of file diffs with colour-coded lines.

use crate::app::App;
use crate::git::diff::LineKind;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    if app.diff_files.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No changes in this commit",
            Style::default().fg(Color::DarkGray),
        )));
    }

    for file in &app.diff_files {
        // File header.
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} ", file.path),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("+{}", file.additions),
                Style::default().fg(Color::Green),
            ),
            Span::styled(" ", Style::default()),
            Span::styled(
                format!("-{}", file.deletions),
                Style::default().fg(Color::Red),
            ),
        ]));

        // Separator.
        lines.push(Line::from(Span::styled(
            " ─".to_string() + &"─".repeat(area.width.saturating_sub(3) as usize),
            Style::default().fg(Color::DarkGray),
        )));

        for hunk in &file.hunks {
            // Hunk header.
            lines.push(Line::from(Span::styled(
                format!(" {}", hunk.header),
                Style::default().fg(Color::Magenta),
            )));

            for line in &hunk.lines {
                let (prefix, fg, bg) = match line.kind {
                    LineKind::Addition => ("+", Color::Green, Color::Rgb(0, 40, 0)),
                    LineKind::Deletion => ("-", Color::Red, Color::Rgb(40, 0, 0)),
                    LineKind::Context => (" ", Color::Gray, Color::Reset),
                    LineKind::Header => ("@", Color::Magenta, Color::Reset),
                };

                let content = line.content.trim_end_matches('\n');
                lines.push(Line::from(Span::styled(
                    format!(" {}{}", prefix, content),
                    Style::default().fg(fg).bg(bg),
                )));
            }

            // Blank line between hunks.
            lines.push(Line::from(""));
        }

        // Blank line between files.
        lines.push(Line::from(""));
    }

    let title = format!(" Diff: {} ", app.diff_commit_hash);
    let block = Block::default()
        .borders(Borders::NONE)
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    // Apply scroll.
    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((app.diff_scroll as u16, 0));

    f.render_widget(paragraph, area);
}
