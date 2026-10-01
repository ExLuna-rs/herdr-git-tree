# herdr-git-tree

[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)](https://www.rust-lang.org/)
[![Build](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/ExLuna-rs/herdr-git-tree)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Herdr](https://img.shields.io/badge/Herdr-plugin-purple)](https://herdr.dev)
[![Built With Ratatui](https://img.shields.io/badge/Built_With-Ratatui-000?logo=ratatui&logoColor=fff&labelColor=000&color=fff)](https://ratatui.rs)
[![git2](https://img.shields.io/badge/powered%20by-libgit2-red)](https://libgit2.org)

Interactive git graph with real-time updates and branch management — a cross-platform [herdr](https://herdr.dev) plugin.

## Features

- **Git Graph** — visual commit tree with colored branch lanes, rounded curves (`╭╮╰╯`), and proper junctions (`├┤┼`)
- **Pushed / Unpushed** — `●` for commits on remote, `○` for local-only commits
- **Real-time refresh** — file watcher detects changes and auto-updates the graph
- **Diff viewer** — color-coded diffs (green additions, red deletions) per commit
- **Branch management** — list, checkout, and delete branches
- **Sidebar toggle** — open/close with one keybinding, works as a split pane
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
# Toggle sidebar
[[keys.command]]
key = "prefix+alt+t"
type = "plugin_action"
command = "git-tree.toggle-windows"    # or "git-tree.toggle" on Linux/macOS
description = "toggle git tree sidebar"
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
| `Esc` / `q` | Back to graph |

### Branch list

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Checkout branch |
| `d` | Delete branch |
| `Esc` / `q` | Back to graph |

## Graph symbols

| Symbol | Meaning |
|--------|---------|
| `●` | Commit (on remote) |
| `○` | Commit (local only / unpushed) |
| `│` | Branch lane |
| `─` | Merge/branch connection |
| `╭` `╮` | Branch curve (top) |
| `╰` `╯` | Branch curve (bottom) |
| `├` `┤` | Junction (branch meets lane) |
| `┼` | Crossing (branch crosses lane) |

## Requirements

- herdr ≥ 0.8.0 (for plugin usage)
- Rust toolchain (for building from source)
- A git repository

## Tech Stack

- [Rust](https://www.rust-lang.org/) — cross-platform, zero runtime dependencies
- [ratatui](https://ratatui.rs) — terminal UI framework
- [git2](https://docs.rs/git2) — libgit2 bindings for native git access
- [crossterm](https://docs.rs/crossterm) — terminal events (keyboard + mouse)
- [notify](https://docs.rs/notify) — file system watcher for real-time updates

## Roadmap

- [ ] Uncommitted changes indicator at top of graph
- [ ] Stash viewer
- [ ] Search commits by message (`/`)
- [ ] Pixel rendering via Kitty/iTerm2 graphics protocol (auto-detected)
- [ ] Syntax highlighting in diffs

## License

MIT
