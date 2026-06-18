mod app;
mod event;
mod model;
mod scope;
mod storage;
mod ui;

use app::App;
use clap::Parser;
use crossterm::{
    event::{Event, poll, read},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{collections::HashMap, env, io, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about = "A Context-Aware Terminal Notes Tool", long_about = None)]
struct Args {
    /// Start in global view
    #[arg(short, long)]
    global: bool,

    /// Override automatic scope detection
    #[arg(short, long)]
    scope: Option<String>,
}

/// Guard to ensure terminal cleanup on exit or panic.
struct TerminalGuard;

impl TerminalGuard {
    /// Enables raw mode and enters the alternate screen.
    fn setup() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let current_scope = args.scope.unwrap_or_else(|| scope::detect_scope(&cwd));

    let data_dir = storage::data_dir();

    let mut notes = Vec::new();
    if args.global {
        let scopes = storage::list_scopes(&data_dir);
        for s in scopes {
            notes.extend(storage::load_notes(&data_dir, &s));
        }
    } else {
        notes = storage::load_notes(&data_dir, &current_scope);
    }

    let mut app = App::new(current_scope.clone(), notes);
    app.show_global = args.global;

    let _guard = TerminalGuard::setup()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        #[allow(clippy::collapsible_if)]
        if poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = read()? {
                event::handle_key(key, &mut app);

                // Persist after any potential mutation
                let mut grouped = HashMap::new();
                for note in &app.notes {
                    grouped
                        .entry(note.scope.clone())
                        .or_insert_with(Vec::new)
                        .push(note.clone());
                }

                // If we're not global, ensure at least the current scope is saved
                // even if we deleted the last note.
                if !app.show_global {
                    if !grouped.contains_key(&app.scope) {
                        grouped.insert(app.scope.clone(), Vec::new());
                    }
                } else {
                    grouped.entry(app.scope.clone()).or_insert_with(Vec::new);

                    let all_scopes = storage::list_scopes(&data_dir);
                    for s in all_scopes {
                        grouped.entry(s).or_insert_with(Vec::new);
                    }
                }

                for (scope_name, scope_notes) in grouped {
                    let _ = storage::save_notes(&data_dir, &scope_name, &scope_notes);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
