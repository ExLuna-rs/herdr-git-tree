# herdr-git-tree

Interactive git graph with real-time diffs and branch management — a cross-platform [herdr](https://herdr.dev) plugin.

## Features

- **Git Graph** — visual commit tree with colored branch lanes, tags, and HEAD marker
- **Real-time refresh** — file watcher detects changes and auto-updates the graph
- **Diff viewer** — color-coded diffs (green additions, red deletions) per commit
- **Branch management** — list, checkout, and delete branches
- **Mouse + keyboard** — click to select, scroll to navigate, full keyboard controls
- **Cross-platform** — works on Linux, macOS, and Windows
- **Standalone** — works inside herdr or as a standalone terminal app

## Installation

### From GitHub (herdr plugin)

```bash
herdr plugin install ExLuna-rs/herdr-git-tree
```

### From source

```bash
git clone https://github.com/ExLuna-rs/herdr-git-tree.git
cd herdr-git-tree
cargo build --release
herdr plugin link .
```

### Standalone (no herdr needed)

```bash
cargo install --path .
cd your-repo
herdr-git-tree
```

## Keybindings

Add to your herdr `config.toml`:

```toml
[[keys.command]]
key = "prefix+alt+t"
type = "plugin_action"
command = "git-tree.open-windows"    # or "git-tree.open" on Linux/macOS
description = "open git tree"
```

### Graph view

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `J` / `PgDn` | Page down |
| `K` / `PgUp` | Page up |
| `g` / `Home` | Go to top |
| `G` / `End` | Go to bottom |
| `Enter` | Open diff for selected commit |
| `b` | Open branch list |
| `r` | Refresh |
| `q` / `Esc` | Quit |
| Mouse scroll | Navigate up/down |
| Mouse click | Select commit |

### Diff view

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `J` / `PgDn` | Page down |
| `K` / `PgUp` | Page up |
| `Esc` / `q` | Back to graph |
| Mouse scroll | Scroll |

### Branch list

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Checkout branch |
| `d` | Delete branch |
| `Esc` / `q` | Back to graph |

## UI Preview

```
 🌿 main ↑2 ↓0
 * a1b2c3d (HEAD)(main) fix: correct event parsing
 │         corentinjsn · 2h ago
 * f643a8d (v0.2.0) release: v0.2.0
 │         corentinjsn · 3h ago
 │ * d14dafa (feat/save) feat: save template
 │/│       corentinjsn · 4h ago
 * eafc917 docs: update README
 │         corentinjsn · 5h ago
 * 02bd94d feat: initial plugin
           corentinjsn · 6h ago
 [↑/k] up  [↓/j] down  [Enter] diff  [b] branches  [r] refresh  [q] quit
```

## Requirements

- herdr ≥ 0.8.0 (for plugin usage)
- Rust toolchain (for building from source)
- A git repository

## Roadmap

- [ ] Syntax highlighting in diffs (syntect)
- [ ] Search commits by message (`/`)
- [ ] Create branch from commit
- [ ] Merge branch visualization
- [ ] Cherry-pick support
- [ ] Stash viewer

## License

MIT
