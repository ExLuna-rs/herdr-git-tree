//! Git graph construction — builds a commit DAG with column assignments for
//! branch-line rendering.

use git2::{Oid, Repository, Sort};
use std::collections::HashMap;

/// A single commit in the graph.
#[derive(Debug, Clone)]
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
    /// The graph prefix lines (Unicode art) for this commit's row.
    pub graph_line: String,
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
    // Collect ref labels for each commit.
    let ref_map = collect_refs(repo)?;

    // Walk commits topologically.
    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
    revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME).map_err(|e| e.to_string())?;
    revwalk.push_head().map_err(|e| e.to_string())?;

    // Also push all branch heads so we see all branches.
    if let Ok(branches) = repo.branches(None) {
        for b in branches.flatten() {
            if let Some(oid) = b.0.get().target() {
                let _ = revwalk.push(oid);
            }
        }
    }

    let mut nodes: Vec<CommitNode> = Vec::with_capacity(max_commits);

    // Active columns: each entry is the Oid that "owns" that column lane.
    let mut columns: Vec<Option<Oid>> = Vec::new();

    for oid_result in revwalk {
        if nodes.len() >= max_commits {
            break;
        }

        let oid = oid_result.map_err(|e| e.to_string())?;
        let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;

        let short_hash = format!("{:.7}", oid);
        let message = commit
            .summary()
            .unwrap_or("")
            .to_string();
        let author = commit.author().name().unwrap_or("unknown").to_string();
        let time = commit.time().seconds();
        let parents: Vec<Oid> = (0..commit.parent_count())
            .filter_map(|i| commit.parent_id(i).ok())
            .collect();
        let refs = ref_map.get(&oid).cloned().unwrap_or_default();

        // Find or assign a column for this commit.
        let col = find_column(&columns, oid);
        let col = match col {
            Some(c) => c,
            None => {
                // New lane — find the first free column.
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

        // Build the graph line art for this row.
        let graph_line = render_graph_line(&columns, col);

        // Update columns: this commit consumed its column, now its parents take over.
        columns[col] = None;

        // First parent continues in the same column.
        if let Some(&first_parent) = parents.first() {
            if find_column(&columns, first_parent).is_none() {
                columns[col] = Some(first_parent);
            }
        }

        // Additional parents get new columns if they don't have one.
        for &parent in parents.iter().skip(1) {
            if find_column(&columns, parent).is_none() {
                let free = columns.iter().position(|c| c.is_none());
                match free {
                    Some(idx) => columns[idx] = Some(parent),
                    None => columns.push(Some(parent)),
                }
            }
        }

        // Trim trailing None columns.
        while columns.last() == Some(&None) {
            columns.pop();
        }

        nodes.push(CommitNode {
            oid,
            short_hash,
            message,
            author,
            time,
            parents,
            refs,
            column: col,
            graph_line,
        });
    }

    Ok(nodes)
}

fn find_column(columns: &[Option<Oid>], oid: Oid) -> Option<usize> {
    columns.iter().position(|c| *c == Some(oid))
}

/// Render the graph prefix for one commit row.
fn render_graph_line(columns: &[Option<Oid>], active_col: usize) -> String {
    let mut line = String::new();
    let width = columns.len().max(active_col + 1);

    for i in 0..width {
        if i == active_col {
            line.push('*');
        } else if columns.get(i).and_then(|c| *c).is_some() {
            line.push('│');
        } else {
            line.push(' ');
        }
        // Spacing between columns.
        if i + 1 < width {
            line.push(' ');
        }
    }

    line
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
                // Peel to the commit for annotated tags.
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
