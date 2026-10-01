//! Graph view — compact sidebar-optimized git graph.
//!
//! Each commit takes two terminal lines:
//! 1. Graph chars + commit MESSAGE (white, bold for selected)
//! 2. Graph chars (continuation) + short HASH (dimmed) + ref labels (colored)

use crate::app::App;
use crate::git::graph::{RefKind, RefLabel};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Vivid lane colours for branch lines.
const LANE_COLORS: &[Color] = &[
    Color::Rgb(0, 217, 204),   // #00d9cc cyan/teal
    Color::Rgb(217, 133, 0),   // #d98500 yellow/orange
    Color::Rgb(0, 217, 10),    // #00d90a green
    Color::Rgb(217, 0, 143),   // #d9008f magenta/pink
    Color::Rgb(0, 133, 217),   // #0085d9 blue
    Color::Rgb(163, 0, 217),   // #a300d9 purple
    Color::Rgb(255, 0, 0),     // #ff0000 red
    Color::Rgb(225, 56, 232),  // #e138e8 bright magenta
];

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    // Split into 3 zones: header (3 lines), graph box (rest), footer (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header: repo + branch + empty
            Constraint::Min(5),    // graph box with border
            Constraint::Length(1), // footer: help text
        ])
        .split(area);

    // ── Header ──────────────────────────────────────────────
    let repo_name = app
        .repo_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("repo");

    let header_lines = vec![
        Line::from(Span::styled(
            format!(" {}", repo_name),
            Style::default().fg(Color::Rgb(100, 100, 120)),
        )),
        Line::from(Span::styled(
            format!(" {}", app.current_branch),
            Style::default()
                .fg(Color::Rgb(152, 195, 121))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    let header = Paragraph::new(header_lines);
    f.render_widget(header, chunks[0]);

    // ── Graph box (with ratatui Block border) ───────────────
    let pos_text = if app.commits.is_empty() {
        " No commits ".to_string()
    } else {
        format!(
            " {}/{} · {} ",
            app.selected + 1,
            app.commits.len(),
            repo_name,
        )
    };

    let graph_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 80)))
        .title(Span::styled(
            " GIT GRAPH ",
            Style::default()
                .fg(Color::Rgb(0, 217, 204))
                .add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Span::styled(
            pos_text,
            Style::default().fg(Color::Rgb(80, 80, 100)),
        ));

    let inner_area = graph_block.inner(chunks[1]);
    f.render_widget(graph_block, chunks[1]);

    // Render commits inside the graph box
    let inner_width = inner_area.width as usize;
    let inner_height = inner_area.height as usize;
    let lines_per_commit = 2usize;
    let visible_commits = inner_height / lines_per_commit;

    let scroll_offset = compute_scroll(app.selected, app.scroll_offset, visible_commits);
    let end = (scroll_offset + visible_commits).min(app.commits.len());

    // Compute graph width for alignment
    let graph_width: usize = app.commits[scroll_offset..end]
        .iter()
        .map(|c| c.graph_chars.len())
        .max()
        .unwrap_or(0);

    let mut lines: Vec<Line> = Vec::new();

    for idx in scroll_offset..end {
        let commit = &app.commits[idx];
        let is_selected = idx == app.selected;
        let bg = if is_selected {
            Color::Rgb(40, 50, 65)
        } else {
            Color::Reset
        };

        // --- Line 1: graph + commit message ---
        let mut spans1: Vec<Span> = Vec::new();
        let mut used1: usize = 0;

        // Graph characters (colored per lane), padded to graph_width.
        for gc in &commit.graph_chars {
            let color = lane_color(gc.color_index);
            let style = if gc.ch == '●' || gc.ch == '○' {
                Style::default().fg(color).bg(bg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color).bg(bg)
            };
            spans1.push(Span::styled(gc.ch.to_string(), style));
            used1 += 1;
        }
        let pad = graph_width.saturating_sub(commit.graph_chars.len());
        if pad > 0 {
            spans1.push(Span::styled(" ".repeat(pad), Style::default().bg(bg)));
            used1 += pad;
        }

        // Two spaces gap.
        spans1.push(Span::styled("  ", Style::default().bg(bg)));
        used1 += 2;

        // Commit message (truncate to fit).
        let msg_max = inner_width.saturating_sub(used1);
        let msg = truncate(&commit.message, msg_max);
        let msg_len = msg.chars().count();
        let msg_style = if is_selected {
            Style::default().fg(Color::White).bg(bg).add_modifier(Modifier::BOLD)
        } else if commit.is_merge {
            Style::default().fg(Color::Rgb(140, 140, 155)).bg(bg)
        } else {
            Style::default().fg(Color::White).bg(bg)
        };
        spans1.push(Span::styled(msg, msg_style));
        used1 += msg_len;

        // Fill remaining with bg
        if used1 < inner_width {
            spans1.push(Span::styled(" ".repeat(inner_width - used1), Style::default().bg(bg)));
        }
        lines.push(Line::from(spans1));

        // --- Line 2: connector graph + hash + refs ---
        let mut spans2: Vec<Span> = Vec::new();
        let mut _used2: usize = 0;

        for gc in &commit.connector_chars {
            let color = lane_color(gc.color_index);
            spans2.push(Span::styled(gc.ch.to_string(), Style::default().fg(color)));
            _ _used2 += 1;
        }
        let pad2 = graph_width.saturating_sub(commit.connector_chars.len());
        if pad2 > 0 {
            spans2.push(Span::raw(" ".repeat(pad2)));
            _ _used2 += pad2;
        }

        spans2.push(Span::raw("  "));
        _ _used2 += 2;

        spans2.push(Span::styled(
            &commit.short_hash,
            Style::default().fg(Color::Rgb(100, 100, 120)),
        ));
        _ _used2 += commit.short_hash.len();

        let refs_text = compact_refs(&commit.refs);
        if !refs_text.is_empty() {
            spans2.push(Span::raw(" "));
            _ _used2 += 1;
            for span in render_refs_inline(&commit.refs, Color::Reset) {
                _ _used2 += span.content.len();
                spans2.push(span);
            }
        }

        lines.push(Line::from(spans2));
    }

    let graph_paragraph = Paragraph::new(lines);
    f.render_widget(graph_paragraph, inner_area);

    // ── Footer: help text ───────────────────────────────────
    let mut help_spans: Vec<Span> = Vec::new();
    help_spans.push(Span::raw(" "));
    let keys_and_labels = [
        ("/", " search  "),
        ("?", " help  "),
        ("q", " quit  "),
        ("↑↓", " move"),
    ];
    for (key, label) in &keys_and_labels {
        help_spans.push(Span::styled(
            *key,
            Style::default()
                .fg(Color::Rgb(0, 217, 204))
                .add_modifier(Modifier::BOLD),
        ));
        help_spans.push(Span::styled(
            *label,
            Style::default().fg(Color::Rgb(80, 80, 100)),
        ));
    }
    let help = Paragraph::new(Line::from(help_spans));
    f.render_widget(help, chunks[2]);
}

// ---------------------------------------------------------------------------
// Ref label rendering
// ---------------------------------------------------------------------------

/// Build a plain text representation of refs (for width calculation).
fn compact_refs(refs: &[RefLabel]) -> String {
    let mut parts = Vec::new();
    let has_head = refs.iter().any(|r| r.kind == RefKind::Head);
    let head_branch = if has_head {
        refs.iter()
            .find(|r| r.kind == RefKind::LocalBranch)
            .map(|r| r.name.clone())
    } else {
        None
    };

    for label in refs {
        match label.kind {
            RefKind::Head => {
                if let Some(ref branch) = head_branch {
                    parts.push(format!("HEAD → {}", branch));
                } else {
                    parts.push("HEAD".to_string());
                }
            }
            RefKind::LocalBranch => {
                if has_head && head_branch.as_deref() == Some(&label.name) {
                    continue;
                }
                parts.push(label.name.clone());
            }
            RefKind::RemoteBranch => {
                parts.push(label.name.clone());
            }
            RefKind::Tag => {
                parts.push(format!("tag: {}", label.name));
            }
        }
    }
    parts.join(", ")
}

/// Render ref labels as colored spans.
fn render_refs_inline(refs: &[RefLabel], bg: Color) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let has_head = refs.iter().any(|r| r.kind == RefKind::Head);
    let head_branch = if has_head {
        refs.iter()
            .find(|r| r.kind == RefKind::LocalBranch)
            .map(|r| r.name.clone())
    } else {
        None
    };

    let mut first = true;
    for label in refs {
        let (text, color) = match label.kind {
            RefKind::Head => {
                let t = if let Some(ref branch) = head_branch {
                    format!("HEAD → {}", branch)
                } else {
                    "HEAD".to_string()
                };
                (t, Color::Rgb(86, 182, 194))
            }
            RefKind::LocalBranch => {
                if has_head && head_branch.as_deref() == Some(&label.name) {
                    continue;
                }
                (label.name.clone(), Color::Rgb(152, 195, 121))
            }
            RefKind::RemoteBranch => {
                (label.name.clone(), Color::Rgb(224, 108, 117))
            }
            RefKind::Tag => {
                (format!("tag: {}", label.name), Color::Rgb(229, 192, 123))
            }
        };

        if !first {
            spans.push(Span::styled(", ", Style::default().fg(Color::Rgb(60, 60, 80)).bg(bg)));
        }
        spans.push(Span::styled(text, Style::default().fg(color).bg(bg).add_modifier(Modifier::BOLD)));
        first = false;
    }

    spans
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn lane_color(index: usize) -> Color {
    LANE_COLORS[index % LANE_COLORS.len()]
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

/// Truncate a string to at most `max_chars` characters, appending "…" if cut.
fn truncate(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else if max_chars <= 1 {
        "…".to_string()
    } else {
        let mut result: String = chars[..max_chars - 1].iter().collect();
        result.push('…');
        result
    }
}
