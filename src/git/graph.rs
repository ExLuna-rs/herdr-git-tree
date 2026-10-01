//! Git graph construction — builds a commit DAG with column assignments for
//! branch-line rendering with VS Code Git Graph style.

use git2::{Oid, Repository, Sort};
use std::collections::HashMap;

/// A single character in the graph rendering, with its lane color index.
#[derive(Debug, Clone)]
pub struct GraphChar {
    pub ch: char,
    /// Lane color index — used by the UI to pick a color from the palette.
    pub color_index: usize,
}

/// A single commit in the graph.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CommitNode {
    pub oid: Oid,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub time: i64,
    pub parents: Vec<Oid>,
    pub refs: Vec<RefLabel>,
    /// Column index in the graph (0-based).
    pub column: usize,
    /// Whether this commit is a merge (>1 parent).
    pub is_merge: bool,
    /// Graph characters for the commit row.
    pub graph_chars: Vec<GraphChar>,
    /// Graph characters for the connector row (between this and next commit).
    pub connector_chars: Vec<GraphChar>,
}

/// A label (branch or tag) attached to a commit.
#[derive(Debug, Clone)]
pub struct RefLabel {
    pub name: String,
    pub kind: RefKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    Head,
    LocalBranch,
    RemoteBranch,
    Tag,
}

/// Build the commit graph from the repository.
///
/// Returns up to `max_commits` nodes with column assignments and graph art.
pub fn build_graph(repo: &Repository, max_commits: usize) -> Result<Vec<CommitNode>, String> {
    let ref_map = collect_refs(repo)?;

    // Collect remote commit OIDs to determine pushed/unpushed status.
    let remote_oids = collect_remote_oids(repo);

    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
    revwalk
        .set_sorting(Sort::TOPOLOGICAL | Sort::TIME)
        .map_err(|e| e.to_string())?;
    revwalk.push_head().map_err(|e| e.to_string())?;

    // Push all branch heads so we see all branches.
    if let Ok(branches) = repo.branches(None) {
        for b in branches.flatten() {
            if let Some(oid) = b.0.get().target() {
                let _ = revwalk.push(oid);
            }
        }
    }

    let mut nodes: Vec<CommitNode> = Vec::with_capacity(max_commits);
    let mut columns: Vec<Option<Oid>> = Vec::new();

    for oid_result in revwalk {
        if nodes.len() >= max_commits {
            break;
        }

        let oid = oid_result.map_err(|e| e.to_string())?;
        let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;

        let short_hash = format!("{:.7}", oid);
        let message = commit.summary().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("unknown").to_string();
        let time = commit.time().seconds();
        let parents: Vec<Oid> = (0..commit.parent_count())
            .filter_map(|i| commit.parent_id(i).ok())
            .collect();
        let refs = ref_map.get(&oid).cloned().unwrap_or_default();
        let is_merge = parents.len() > 1;
        let is_on_remote = remote_oids.contains(&oid);

        // Find or assign a column for this commit.
        let col = match find_column(&columns, oid) {
            Some(c) => c,
            None => {
                let free = columns.iter().position(|c| c.is_none());
                match free {
                    Some(idx) => {
                        columns[idx] = Some(oid);
                        idx
                    }
                    None => {
                        columns.push(Some(oid));
                        columns.len() - 1
                    }
                }
            }
        };

        // Consume this commit's column.
        columns[col] = None;

        // First parent continues in the same column (if not already elsewhere).
        // Track if the lane converges (branch base point).
        let mut converge_to: Option<usize> = None;
        if let Some(&first_parent) = parents.first() {
            let existing = find_column(&columns, first_parent);
            if existing.is_none() {
                columns[col] = Some(first_parent);
            } else {
                // First parent is already in another column — this lane ENDS here.
                // The branch converges back to the parent's column.
                converge_to = existing;
            }
        }

        // Additional parents get columns — track where they land for merge lines.
        let mut merge_parent_cols: Vec<usize> = Vec::new();
        for &parent in parents.iter().skip(1) {
            if let Some(existing_col) = find_column(&columns, parent) {
                merge_parent_cols.push(existing_col);
            } else {
                let free = columns.iter().position(|c| c.is_none());
                let new_col = match free {
                    Some(idx) => {
                        columns[idx] = Some(parent);
                        idx
                    }
                    None => {
                        columns.push(Some(parent));
                        columns.len() - 1
                    }
                };
                merge_parent_cols.push(new_col);
            }
        }

        // Trim trailing empty columns.
        while columns.last() == Some(&None) {
            columns.pop();
        }

        // Render rows.
        let graph_chars = render_commit_row(&columns, col, is_merge, is_on_remote, &merge_parent_cols);
        let connector_chars = render_connector_row(&columns, col, &merge_parent_cols, converge_to);

        nodes.push(CommitNode {
            oid,
            short_hash,
            message,
            author,
            time,
            parents,
            refs,
            column: col,
            is_merge,
            graph_chars,
            connector_chars,
        });
    }

    // Most recent at top (standard git log order).
    Ok(nodes)
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

/// Render the commit row: ● for pushed, ○ for unpushed/local-only, │ for lanes, ─╮/╭─ for merge lines.
fn render_commit_row(
    columns: &[Option<Oid>],
    active_col: usize,
    _is_merge: bool,
    is_on_remote: bool,
    merge_parent_cols: &[usize],
) -> Vec<GraphChar> {
    let max_col = merge_parent_cols
        .iter()
        .copied()
        .max()
        .unwrap_or(0)
        .max(active_col);
    let width = columns.len().max(max_col + 1);
    if width == 0 {
        return Vec::new();
    }
    let char_width = width * 2 - 1;

    // Initialize with spaces.
    let mut row: Vec<GraphChar> = vec![
        GraphChar {
            ch: ' ',
            color_index: 0,
        };
        char_width
    ];

    // Draw active lanes (│) — skip the commit column and merge endpoints.
    for i in 0..width {
        if i == active_col {
            continue;
        }
        // Merge endpoint columns get their own glyph below — skip │ for them.
        if merge_parent_cols.contains(&i) {
            continue;
        }
        if columns.get(i).and_then(|c| *c).is_some() {
            row[i * 2] = GraphChar {
                ch: '│',
                color_index: i,
            };
        }
    }

    // Draw commit node.
    row[active_col * 2] = GraphChar {
        ch: if !is_on_remote { '○' } else { '●' },
        color_index: active_col,
    };

    // Draw merge lines from commit to each merge parent.
    for &pcol in merge_parent_cols {
        if pcol == active_col {
            continue;
        }
        let (left, right) = if pcol < active_col {
            (pcol, active_col)
        } else {
            (active_col, pcol)
        };

        // Horizontal fill between the two columns (only between positions).
        for pos in (left * 2 + 1)..=(right * 2 - 1) {
            if pos < row.len() {
                if row[pos].ch == '│' {
                    // A lane crosses our horizontal merge line.
                    row[pos] = GraphChar {
                        ch: '┼',
                        color_index: row[pos].color_index,
                    };
                } else if pos != active_col * 2 {
                    // Color transition: chars near commit = commit color,
                    // chars near branch endpoint = branch color
                    let mid = (left * 2 + right * 2) / 2;
                    let color = if pos <= mid { active_col } else { pcol };
                    row[pos] = GraphChar {
                        ch: '─',
                        color_index: color,
                    };
                }
            }
        }

        // Merge endpoint: use curve that connects DOWN into the lane below.
        // Since graph reads top→bottom (newest first), merge parents are below,
        // so curves open downward.
        let endpoint_pos = pcol * 2;
        if endpoint_pos < row.len() {
            row[endpoint_pos] = GraphChar {
                ch: if pcol > active_col { '╮' } else { '╭' },
                color_index: pcol,
            };
        }
    }

    row
}

/// Render the connector row between commits — fully connected, no gaps.
///
/// Every active lane gets a `│`.  The commit column itself also gets `│` so the
/// line from one ● to the next ● is seamless.
/// If `converge_to` is set, draw a curve from `commit_col` to that column
/// (branch base point — lane ends and merges into parent's lane).
fn render_connector_row(
    columns: &[Option<Oid>],
    commit_col: usize,
    _merge_parent_cols: &[usize],
    converge_to: Option<usize>,
) -> Vec<GraphChar> {
    let max_col = converge_to.unwrap_or(0).max(commit_col);
    let width = columns.len().max(max_col + 1);
    if width == 0 {
        return Vec::new();
    }
    let char_width = width * 2 - 1;

    let mut row: Vec<GraphChar> = vec![
        GraphChar {
            ch: ' ',
            color_index: 0,
        };
        char_width
    ];

    // Draw │ for every active lane.
    for i in 0..width {
        if columns.get(i).and_then(|c| *c).is_some() {
            row[i * 2] = GraphChar {
                ch: '│',
                color_index: i,
            };
        }
    }

    // Draw convergence curve (branch base: this lane ends, connects to parent lane)
    if let Some(target_col) = converge_to {
        if target_col != commit_col {
            let (left, right) = if target_col < commit_col {
                (target_col, commit_col)
            } else {
                (commit_col, target_col)
            };

            // Starting point: curve at commit_col
            let start_pos = commit_col * 2;
            if start_pos < char_width {
                row[start_pos] = GraphChar {
                    ch: if target_col < commit_col { '╰' } else { '╯' },
                    color_index: commit_col,
                };
            }

            // Horizontal fill between
            for pos in (left * 2 + 1)..=(right * 2 - 1) {
                if pos < char_width {
                    if row[pos].ch == '│' {
                        row[pos] = GraphChar { ch: '┼', color_index: row[pos].color_index };
                    } else {
                        let mid = (left * 2 + right * 2) / 2;
                        let color = if pos <= mid {
                            if commit_col < target_col { commit_col } else { target_col }
                        } else {
                            if commit_col < target_col { target_col } else { commit_col }
                        };
                        row[pos] = GraphChar { ch: '─', color_index: color };
                    }
                }
            }

            // Endpoint: junction at target lane
            let end_pos = target_col * 2;
            if end_pos < char_width {
                row[end_pos] = GraphChar {
                    ch: if target_col < commit_col { '├' } else { '┤' },
                    color_index: target_col,
                };
            }
        }
    }

    row
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

fn find_column(columns: &[Option<Oid>], oid: Oid) -> Option<usize> {
    columns.iter().position(|c| *c == Some(oid))
}

/// Collect all commit OIDs reachable from remote branches.
/// A commit in this set is "pushed" (on the remote).
fn collect_remote_oids(repo: &Repository) -> std::collections::HashSet<Oid> {
    let mut remote_oids = std::collections::HashSet::new();

    let branches = match repo.branches(Some(git2::BranchType::Remote)) {
        Ok(b) => b,
        Err(_) => return remote_oids,
    };

    for branch_result in branches.flatten() {
        let (branch, _) = branch_result;
        if let Some(oid) = branch.get().target() {
            // Walk all ancestors of this remote branch
            if let Ok(mut revwalk) = repo.revwalk() {
                let _ = revwalk.push(oid);
                let _ = revwalk.set_sorting(Sort::TOPOLOGICAL);
                // Limit walk to avoid being too slow on huge repos
                for (count, oid_result) in revwalk.enumerate() {
                    if count >= 500 {
                        break;
                    }
                    if let Ok(commit_oid) = oid_result {
                        remote_oids.insert(commit_oid);
                    }
                }
            }
        }
    }

    remote_oids
}

/// Collect all branch and tag refs mapped to their target Oid.
fn collect_refs(repo: &Repository) -> Result<HashMap<Oid, Vec<RefLabel>>, String> {
    let mut map: HashMap<Oid, Vec<RefLabel>> = HashMap::new();
    let head_oid = repo.head().ok().and_then(|h| h.target());

    // Add HEAD label.
    if let Some(oid) = head_oid {
        map.entry(oid).or_default().push(RefLabel {
            name: "HEAD".to_string(),
            kind: RefKind::Head,
        });
    }

    // Branches.
    if let Ok(branches) = repo.branches(None) {
        for item in branches.flatten() {
            let (branch, branch_type) = item;
            if let (Some(name), Some(oid)) = (branch.name().ok().flatten(), branch.get().target())
            {
                let kind = match branch_type {
                    git2::BranchType::Local => RefKind::LocalBranch,
                    git2::BranchType::Remote => RefKind::RemoteBranch,
                };
                map.entry(oid).or_default().push(RefLabel {
                    name: name.to_string(),
                    kind,
                });
            }
        }
    }

    // Tags.
    if let Ok(tags) = repo.tag_names(None) {
        for tag_name in tags.iter().flatten() {
            if let Ok(reference) = repo.find_reference(&format!("refs/tags/{}", tag_name)) {
                let oid = reference
                    .peel_to_commit()
                    .map(|c| c.id())
                    .or_else(|_| reference.target().ok_or(()))
                    .unwrap_or_else(|_| Oid::zero());

                if !oid.is_zero() {
                    map.entry(oid).or_default().push(RefLabel {
                        name: tag_name.to_string(),
                        kind: RefKind::Tag,
                    });
                }
            }
        }
    }

    Ok(map)
}
