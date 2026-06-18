/// Input event handling.
use crate::app::{App, Mode};
use crossterm::event::{KeyCode, KeyEvent};

/// Dispatches a key event to update the application state.
pub fn handle_key(key: KeyEvent, app: &mut App) {
    match app.mode {
        Mode::Normal => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => app.move_selection_down(),
            KeyCode::Char('k') | KeyCode::Up => app.move_selection_up(),
            KeyCode::Char('a') => app.start_add(),
            KeyCode::Char('e') => app.start_edit(),
            KeyCode::Char('v') => app.toggle_multi_select(),
            KeyCode::Char(' ') | KeyCode::Char('d') => app.toggle_selected_done(),
            KeyCode::Char('x') => app.start_delete_confirm(),
            KeyCode::Char('/') => app.start_search(),
            KeyCode::Char('g') => app.toggle_global_view(),
            _ => {}
        },
        Mode::Adding => match key.code {
            KeyCode::Enter => {
                app.confirm_add();
            }
            KeyCode::Esc => app.cancel_add(),
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => app.input_buffer.push(c),
            _ => {}
        },
        Mode::Editing => match key.code {
            KeyCode::Enter => {
                app.confirm_edit();
            }
            KeyCode::Esc => app.cancel_edit(),
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => app.input_buffer.push(c),
            _ => {}
        },
        Mode::Searching => match key.code {
            KeyCode::Enter => app.apply_search(),
            KeyCode::Esc => app.clear_search(),
            KeyCode::Backspace => {
                app.search_query.pop();
            }
            KeyCode::Char(c) => app.search_query.push(c),
            _ => {}
        },
        Mode::ConfirmDelete => match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                app.confirm_delete();
            }
            KeyCode::Char('n') | KeyCode::Esc => app.cancel_delete(),
            _ => {}
        },
    }
}
