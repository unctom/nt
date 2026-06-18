# nt

`nt` is a context-aware terminal UI notes application written in Rust. It automatically scopes your notes to the current git repository (or directory if not in a repository), allowing you to maintain project-specific notes without manual organization. It features inline `#tags` parsing, global scope viewing, and full-text search.

## Keybindings

| Mode | Key | Action |
| --- | --- | --- |
| **Normal** | `j` / `Down` | Move selection down |
| **Normal** | `k` / `Up` | Move selection up |
| **Normal** | `a` | Add a new note |
| **Normal** | `space` / `d`| Toggle 'done' status of selected note |
| **Normal** | `x` | Delete selected note |
| **Normal** | `/` | Search/filter notes |
| **Normal** | `g` | Toggle global view (show all scopes) |
| **Normal** | `q` / `Esc` | Quit the application |
| **Adding** | `Enter` | Save note |
| **Adding** | `Esc` | Cancel adding |
| **Searching**| `Enter` | Apply filter |
| **Searching**| `Esc` | Clear search |
| **Confirm** | `y` / `Enter`| Confirm delete |
| **Confirm** | `n` / `Esc` | Cancel delete |

## Storage

Data is stored as plain pretty-printed JSON files (one per scope). They are located in your OS's default local data directory under `nt/notes`.
For Linux desktop, this is typically `~/.local/share/nt/notes/<scope>.json`.

## Build & Run

Ensure you have Rust installed, then run:

```bash
cargo build --release
cargo run
```
