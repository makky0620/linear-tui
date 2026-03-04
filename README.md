# linear-tui

A TUI client for [Linear.app](https://linear.app) — manage issues, projects, and cycles from your terminal.

Built with Rust using [ratatui](https://ratatui.rs/) and the Linear GraphQL API.

## Features

- **Issue management** — Browse, search, filter, and mutate issues (status, priority, assignee, comments)
- **Multiple views** — Issues, My Issues, Projects, Cycles with tab-based navigation
- **Project & Cycle detail** — Drill into projects/cycles to see their issues
- **Vim-style keybindings** — `j`/`k` navigation, `/` search, `?` help
- **OAuth2 + PKCE authentication** — Secure login via browser, or use a personal API key
- **Theme support** — Default (dark), Light, and Ocean color schemes
- **Pagination** — Cursor-based infinite scrolling for large issue lists

## Installation

### From crates.io

```sh
cargo install linear-tui
```

### From source

```sh
git clone https://github.com/k1-c/linear-tui.git
cd linear-tui
cargo install --path .
```

## Authentication

### OAuth2 (recommended)

```sh
linear-tui auth login
```

This opens your browser for Linear OAuth authorization. Tokens are stored locally and refreshed automatically.

### Personal API Key

Generate a key at [Linear Settings > API](https://linear.app/settings/api), then:

```sh
linear-tui auth token <your-api-key>
```

### Logout

```sh
linear-tui auth logout
```

## Usage

```sh
linear-tui
```

### Keybindings

| Key         | Action                                               |
| ----------- | ---------------------------------------------------- |
| `j` / `k`   | Move cursor down / up                                |
| `g` / `G`   | Jump to first / last item                            |
| `Enter`     | Open detail view                                     |
| `Esc` / `q` | Back / quit                                          |
| `1`-`4`     | Switch tabs (Issues / My Issues / Projects / Cycles) |
| `s`         | Change status                                        |
| `p`         | Change priority                                      |
| `a`         | Change assignee                                      |
| `c`         | Add comment (Ctrl+Enter to send)                     |
| `t`         | Switch team                                          |
| `f` / `F`   | Filter / clear filters                               |
| `/`         | Search issues                                        |
| `r`         | Reload data                                          |
| `?`         | Toggle help                                          |

## Configuration

Config file location: `~/.config/linear-tui/config.toml`

```toml
[auth]
# OAuth tokens are managed automatically via `linear-tui auth login`
# To use a personal API key instead:
# api_key = "lin_api_xxxxx"

[ui]
default_team = "Core"      # Auto-select this team on startup
items_per_page = 50         # Issues per page (pagination)
theme = "default"           # "default" | "light" | "ocean"
```

### Themes

| Theme     | Description                        |
| --------- | ---------------------------------- |
| `default` | Dark theme with cyan accents       |
| `light`   | Light background with blue accents |
| `ocean`   | Dark blue palette with soft colors |

## License

MIT
