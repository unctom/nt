/// Terminal UI rendering using ratatui.
use crate::app::{App, Mode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Renders the main application UI into the given terminal frame.
pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // List
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_list(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);

    match app.mode {
        Mode::Adding | Mode::Editing | Mode::Searching => render_input(f, app),
        Mode::ConfirmDelete => render_confirm_delete(f),
        Mode::Normal => {}
    }
}

/// Renders the header containing the scope title.
fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.show_global {
        "ALL SCOPES".to_string()
    } else {
        format!("Scope: {}", app.scope)
    };

    let p = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(p, area);
}

/// Renders the main list of notes.
fn render_list(f: &mut Frame, app: &App, area: Rect) {
    let visible = app.visible_notes();

    if visible.is_empty() {
        let p = Paragraph::new("No notes found.")
            .block(Block::default().borders(Borders::ALL).title("Notes"));
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, note)| {
            let status = if note.done { "[x]" } else { "[ ]" };
            let prefix = if app.multi_selected.contains(&note.id) { "*" } else { " " };
            let scope_label = if app.show_global {
                format!(" ({})", note.scope)
            } else {
                String::new()
            };

            let base_style = if note.done {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            let mut spans = vec![
                Span::styled(format!("{}{} ", prefix, status), if app.multi_selected.contains(&note.id) { Style::default().fg(Color::Yellow) } else { base_style }),
                Span::styled(&note.text, base_style),
                Span::styled(scope_label, Style::default().fg(Color::DarkGray)),
            ];

            if !note.tags.is_empty() {
                spans.push(Span::raw(" "));
                for tag in &note.tags {
                    let mut tag_style = Style::default().fg(Color::LightBlue);
                    if let Some(stripped) = tag.strip_prefix("due:") {
                        #[allow(clippy::collapsible_if)]
                        if let Ok(date) = chrono::NaiveDate::parse_from_str(stripped, "%Y-%m-%d") {
                            let today = chrono::Utc::now().naive_utc().date();
                            if date < today {
                                tag_style = tag_style.fg(Color::Red).add_modifier(Modifier::BOLD);
                            } else if date == today {
                                tag_style = tag_style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                            }
                        }
                    }
                    spans.push(Span::styled(
                        format!("#{} ", tag),
                        tag_style,
                    ));
                }
            }

            let mut item = ListItem::new(Line::from(spans));
            if i == app.selected {
                item = item.style(Style::default().bg(Color::DarkGray).fg(Color::White));
            }
            item
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Notes"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected));

    f.render_stateful_widget(list, area, &mut state);
}

/// Renders the footer containing statistics and keybindings.
fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let total = app.visible_notes().len();
    let open = app.visible_notes().iter().filter(|n| !n.done).count();

    let stats = format!("{} total, {} open", total, open);
    let binds = match app.mode {
        Mode::Normal => "q/Esc: Quit | j/k: Nav | a: Add | e: Edit | v: Select | space: Toggle | x: Del | /: Search | g: Global",
        Mode::Adding | Mode::Editing => "Enter: Save | Esc: Cancel",
        Mode::Searching => "Enter: Filter | Esc: Clear",
        Mode::ConfirmDelete => "y/Enter: Confirm | n/Esc: Cancel",
    };

    let line = format!("{} | {}", stats, binds);

    let p = Paragraph::new(line)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(p, area);
}

/// Renders an overlay input dialog for adding notes or searching.
fn render_input(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.area());
    let (title, content) = match app.mode {
        Mode::Adding => ("Add Note", &app.input_buffer),
        Mode::Editing => ("Edit Note", &app.input_buffer),
        Mode::Searching => ("Search Notes", &app.search_query),
        _ => ("", &String::new()),
    };

    let block = Block::default().title(title).borders(Borders::ALL);
    let p = Paragraph::new(content.as_str()).block(block);
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

/// Renders an overlay dialog to confirm note deletion.
fn render_confirm_delete(f: &mut Frame) {
    let area = centered_rect(40, 20, f.area());
    let block = Block::default()
        .title("Confirm Delete")
        .borders(Borders::ALL);
    let p = Paragraph::new("Are you sure you want to delete this note? (y/N)")
        .block(block)
        .style(Style::default().fg(Color::Red));
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

/// Helper function to create a centered rectangle within another rectangle.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Note;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_render_empty_normal_mode() {
        let app = App::new("test".to_string(), vec![]);
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_with_notes_normal_mode() {
        let mut app = App::new("test".to_string(), vec![]);
        app.notes
            .push(Note::new("task 1".to_string(), "test".to_string()));
        app.notes
            .push(Note::new("task 2 #urgent".to_string(), "test".to_string()));
        app.notes[0].toggle_done();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_modes() {
        let mut app = App::new("test".to_string(), vec![]);
        app.notes
            .push(Note::new("task".to_string(), "test".to_string()));

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        app.mode = Mode::Adding;
        app.input_buffer = "typing...".to_string();
        terminal.draw(|f| render(f, &app)).unwrap();

        app.mode = Mode::Searching;
        app.search_query = "q".to_string();
        terminal.draw(|f| render(f, &app)).unwrap();

        app.mode = Mode::ConfirmDelete;
        terminal.draw(|f| render(f, &app)).unwrap();
    }
}
