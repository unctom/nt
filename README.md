# nt

> A fast, context-aware terminal UI notes application written in Rust.

`nt` automatically scopes your notes to the current git repository (or directory if not in a repository). This allows you to maintain project-specific notes without manual organization. It features inline `#tags` parsing, global scope viewing, fuzzy search, bulk actions, and dynamic due dates.

## Installation

### Method 1: Build from Source (Cargo)

If you have Rust installed, you can build and install directly from the repository:

```bash
cargo install --git https://github.com/unctom/nt.git
```

Make sure your `~/.cargo/bin/` is in your system's `$PATH`.

### Method 2: Clone and Build

```bash
git clone https://github.com/unctom/nt.git
cd nt
cargo install --path .
```

## Features

- **Context-Aware Scoping:** Notes are automatically tied to the git repository you are currently working in.
- **Inline Tags:** Add `#urgent` or `#bug` anywhere in your note to tag it.
- **Dynamic Due Dates:** Tag a note with `#due:YYYY-MM-DD` and `nt` will dynamically highlight it yellow if it's due today, and red if it's overdue.
- **Fuzzy Searching:** Instantly filter your notes by text or tags using an advanced fuzzy matcher.
- **Bulk Actions:** Select multiple notes using `v` and delete or mark them as done all at once.
- **In-Place Editing:** Easily edit your notes with a single keystroke.
- **Data Export:** Export your scope's notes to a clean Markdown file.

## Keybindings

| Mode               | Key           | Action                               |
| ------------------ | ------------- | ------------------------------------ |
| **Normal**         | `j` / `Down`  | Move selection down                  |
| **Normal**         | `k` / `Up`    | Move selection up                    |
| **Normal**         | `a`           | Add a new note                       |
| **Normal**         | `e`           | Edit the selected note               |
| **Normal**         | `v`           | Toggle selection for bulk actions    |
| **Normal**         | `space` / `d` | Toggle 'done' status                 |
| **Normal**         | `x`           | Delete note(s)                       |
| **Normal**         | `/`           | Search/filter notes                  |
| **Normal**         | `g`           | Toggle global view (show all scopes) |
| **Normal**         | `q` / `Esc`   | Quit the application                 |
| **Adding/Editing** | `Enter`       | Save note                            |
| **Adding/Editing** | `Esc`         | Cancel                               |
| **Searching**      | `Enter`       | Apply filter                         |
| **Searching**      | `Esc`         | Clear search                         |
| **Confirm**        | `y` / `Enter` | Confirm delete                       |
| **Confirm**        | `n` / `Esc`   | Cancel delete                        |

## Usage & Exporting

To launch the TUI, simply run `nt` from your terminal.

You can also bypass the TUI and export your notes for the current scope to a Markdown file:

```bash
nt --export ./project-notes.md
```

## Storage

Data is stored locally as plain, pretty-printed JSON files (one per scope). They are located in your OS's default local data directory under `nt/notes`.
For Linux desktop, this is typically `~/.local/share/nt/notes/<scope>.json`.
