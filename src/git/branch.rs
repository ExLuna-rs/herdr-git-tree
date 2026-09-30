//! Branch operations: list, create, switch, delete, merge.

use git2::{BranchType, Repository};

/// Info about a branch.
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

/// Get the name of the current branch (or None for detached HEAD).
pub fn current_branch_name(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;
    if head.is_branch() {
        head.shorthand().map(|s| s.to_string())
    } else {
        None
    }
}

/// List all branches (local and remote).
pub fn list_branches(repo: &Repository) -> Result<Vec<BranchInfo>, String> {
    let mut branches = Vec::new();
    let head_name = current_branch_name(repo);

    for item in repo
        .branches(None)
        .map_err(|e| e.to_string())?
        .flatten()
    {
        let (branch, branch_type) = item;
        let name = match branch.name() {
            Ok(Some(n)) => n.to_string(),
            _ => continue,
        };

        let is_remote = branch_type == BranchType::Remote;
        let is_head = !is_remote && head_name.as_deref() == Some(name.as_str());

        let (ahead, behind, upstream) = if !is_remote {
            match branch.upstream() {
                Ok(upstream_branch) => {
                    let upstream_name = upstream_branch
                        .name()
                        .ok()
                        .flatten()
                        .map(|s| s.to_string());
                    let (a, b) = match (branch.get().target(), upstream_branch.get().target()) {
                        (Some(local), Some(remote)) => {
                            repo.graph_ahead_behind(local, remote).unwrap_or((0, 0))
                        }
                        _ => (0, 0),
                    };
                    (a, b, upstream_name)
                }
                Err(_) => (0, 0, None),
            }
        } else {
            (0, 0, None)
        };

        branches.push(BranchInfo {
            name,
            is_head,
            is_remote,
            upstream,
            ahead,
            behind,
        });
    }

    // Sort: HEAD first, then local, then remote.
    branches.sort_by(|a, b| {
        b.is_head
            .cmp(&a.is_head)
            .then(a.is_remote.cmp(&b.is_remote))
            .then(a.name.cmp(&b.name))
    });

    Ok(branches)
}

/// Switch to a local branch.
pub fn switch_branch(repo: &Repository, name: &str) -> Result<(), String> {
    let obj = repo
        .revparse_single(&format!("refs/heads/{}", name))
        .map_err(|e| format!("Branch not found: {}", e))?;

    repo.checkout_tree(&obj, None)
        .map_err(|e| format!("Checkout failed: {}", e))?;

    repo.set_head(&format!("refs/heads/{}", name))
        .map_err(|e| format!("Set head failed: {}", e))?;

    Ok(())
}

/// Create a new branch at the given target (or HEAD if None).
pub fn create_branch(
    repo: &Repository,
    name: &str,
    target: Option<git2::Oid>,
) -> Result<(), String> {
    let commit = match target {
        Some(oid) => repo.find_commit(oid).map_err(|e| e.to_string())?,
        None => {
            let head = repo.head().map_err(|e| e.to_string())?;
            head.peel_to_commit().map_err(|e| e.to_string())?
        }
    };

    repo.branch(name, &commit, false)
        .map_err(|e| format!("Create branch failed: {}", e))?;

    Ok(())
}

/// Delete a local branch.
pub fn delete_branch(repo: &Repository, name: &str) -> Result<(), String> {
    let mut branch = repo
        .find_branch(name, BranchType::Local)
        .map_err(|e| format!("Branch not found: {}", e))?;

    branch
        .delete()
        .map_err(|e| format!("Delete failed: {}", e))?;

    Ok(())
}
