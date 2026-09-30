//! Diff computation using git2's diff API.

use git2::{DiffOptions, Oid, Repository};

/// A file that changed in a diff.
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: Vec<DiffHunk>,
}

/// A single hunk within a file diff.
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

/// A single line in a diff hunk.
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: LineKind,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LineKind {
    Context,
    Addition,
    Deletion,
    Header,
}

/// Get the diff for a specific commit against its first parent.
pub fn get_commit_diff(repo: &Repository, oid: Oid) -> Result<Vec<FileDiff>, String> {
    let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;

    let tree = commit.tree().map_err(|e| e.to_string())?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(
            commit
                .parent(0)
                .map_err(|e| e.to_string())?
                .tree()
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };

    let mut opts = DiffOptions::new();
    opts.context_lines(3);

    let diff = repo
        .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut opts))
        .map_err(|e| e.to_string())?;

    parse_diff(&diff)
}

/// Get the diff of the working tree against HEAD.
#[allow(dead_code)]
pub fn get_working_diff(repo: &Repository) -> Result<Vec<FileDiff>, String> {
    let head_tree = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_tree().ok());

    let mut opts = DiffOptions::new();
    opts.context_lines(3);
    opts.include_untracked(true);

    let diff = repo
        .diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut opts))
        .map_err(|e| e.to_string())?;

    parse_diff(&diff)
}

fn parse_diff(diff: &git2::Diff) -> Result<Vec<FileDiff>, String> {
    let mut files: Vec<FileDiff> = Vec::new();

    // Use diff.print which gives us a single callback for all lines.
    diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());

        // Ensure we have a FileDiff for this file.
        let needs_new = files.last().map_or(true, |f| f.path != path);
        if needs_new {
            files.push(FileDiff {
                path,
                additions: 0,
                deletions: 0,
                hunks: Vec::new(),
            });
        }

        let file = files.last_mut().unwrap();

        match line.origin() {
            'H' | 'F' => {
                // File header or footer — skip.
            }
            '+' => {
                file.additions += 1;
                // Ensure we have a hunk.
                if file.hunks.is_empty() {
                    let header = hunk
                        .map(|h| String::from_utf8_lossy(h.header()).trim().to_string())
                        .unwrap_or_default();
                    file.hunks.push(DiffHunk {
                        header,
                        lines: Vec::new(),
                    });
                }
                let content = String::from_utf8_lossy(line.content()).to_string();
                file.hunks.last_mut().unwrap().lines.push(DiffLine {
                    kind: LineKind::Addition,
                    content,
                });
            }
            '-' => {
                file.deletions += 1;
                if file.hunks.is_empty() {
                    let header = hunk
                        .map(|h| String::from_utf8_lossy(h.header()).trim().to_string())
                        .unwrap_or_default();
                    file.hunks.push(DiffHunk {
                        header,
                        lines: Vec::new(),
                    });
                }
                let content = String::from_utf8_lossy(line.content()).to_string();
                file.hunks.last_mut().unwrap().lines.push(DiffLine {
                    kind: LineKind::Deletion,
                    content,
                });
            }
            ' ' => {
                // Context line.
                if file.hunks.is_empty() {
                    let header = hunk
                        .map(|h| String::from_utf8_lossy(h.header()).trim().to_string())
                        .unwrap_or_default();
                    file.hunks.push(DiffHunk {
                        header,
                        lines: Vec::new(),
                    });
                }
                let content = String::from_utf8_lossy(line.content()).to_string();
                file.hunks.last_mut().unwrap().lines.push(DiffLine {
                    kind: LineKind::Context,
                    content,
                });
            }
            _ => {
                // Hunk header line.
                let header = hunk
                    .map(|h| String::from_utf8_lossy(h.header()).trim().to_string())
                    .unwrap_or_default();
                if !header.is_empty() {
                    file.hunks.push(DiffHunk {
                        header,
                        lines: Vec::new(),
                    });
                }
            }
        }

        true
    })
    .map_err(|e| e.to_string())?;

    Ok(files)
}
