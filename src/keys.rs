//! Keybinding definitions — centralised reference for all views.

/// Help text for the graph view.
#[allow(dead_code)]
pub const GRAPH_HELP: &str =
    " [↑/k] up  [↓/j] down  [Enter] diff  [b] branches  [r] refresh  [q] quit";

/// Help text for the diff view.
#[allow(dead_code)]
pub const DIFF_HELP: &str = " [↑/k] up  [↓/j] down  [PgUp/K] page up  [PgDn/J] page down  [Esc] back";

/// Help text for the branch list.
#[allow(dead_code)]
pub const BRANCH_HELP: &str =
    " [↑/k] up  [↓/j] down  [Enter] checkout  [d] delete  [Esc] back";
