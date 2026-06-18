use crate::model::Note;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Adding,
    Searching,
    ConfirmDelete,
}

pub struct App {
    pub notes: Vec<Note>,
    pub scope: String,
    pub selected: usize,
    pub mode: Mode,
    pub input_buffer: String,
    pub search_query: String,
    pub show_global: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(scope: String, notes: Vec<Note>) -> Self {
        Self {
            notes,
            scope,
            selected: 0,
            mode: Mode::Normal,
            input_buffer: String::new(),
            search_query: String::new(),
            show_global: false,
            should_quit: false,
        }
    }

    pub fn visible_notes(&self) -> Vec<&Note> {
        let query = self.search_query.to_lowercase();
        self.notes
            .iter()
            .filter(|note| {
                if query.is_empty() {
                    return true;
                }
                if note.text.to_lowercase().contains(&query) {
                    return true;
                }
                note.tags.iter().any(|t| t.to_lowercase().contains(&query))
            })
            .collect()
    }

    fn clamp_selection(&mut self) {
        let count = self.visible_notes().len();
        if count == 0 {
            self.selected = 0;
        } else if self.selected >= count {
            self.selected = count - 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        let count = self.visible_notes().len();
        if count > 0 && self.selected < count - 1 {
            self.selected += 1;
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn start_add(&mut self) {
        self.mode = Mode::Adding;
        self.input_buffer.clear();
    }

    pub fn confirm_add(&mut self) -> Option<Note> {
        let text = self.input_buffer.trim();
        if text.is_empty() {
            return None;
        }

        let note = Note::new(text.to_string(), self.scope.clone());
        self.notes.push(note.clone());
        self.input_buffer.clear();
        self.mode = Mode::Normal;
        self.clamp_selection();
        Some(note)
    }

    pub fn cancel_add(&mut self) {
        self.input_buffer.clear();
        self.mode = Mode::Normal;
    }

    pub fn toggle_selected_done(&mut self) {
        let visible = self.visible_notes();
        if self.selected < visible.len() {
            let id = visible[self.selected].id.clone();
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.toggle_done();
            }
        }
    }

    pub fn start_delete_confirm(&mut self) {
        let visible = self.visible_notes();
        if self.selected < visible.len() {
            self.mode = Mode::ConfirmDelete;
        }
    }

    pub fn confirm_delete(&mut self) -> Option<String> {
        if self.mode != Mode::ConfirmDelete {
            return None;
        }
        let visible = self.visible_notes();
        if self.selected < visible.len() {
            let id = visible[self.selected].id.clone();
            self.notes.retain(|n| n.id != id);
            self.mode = Mode::Normal;
            self.clamp_selection();
            Some(id)
        } else {
            self.mode = Mode::Normal;
            None
        }
    }

    pub fn cancel_delete(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn start_search(&mut self) {
        self.mode = Mode::Searching;
    }

    pub fn apply_search(&mut self) {
        self.mode = Mode::Normal;
        self.clamp_selection();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.mode = Mode::Normal;
        self.clamp_selection();
    }

    pub fn toggle_global_view(&mut self) {
        self.show_global = !self.show_global;
        self.clamp_selection();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_app() -> App {
        let mut app = App::new("test-scope".to_string(), vec![]);
        app.notes.push(Note::new("first note".to_string(), "test-scope".to_string()));
        app.notes.push(Note::new("second note #tag".to_string(), "test-scope".to_string()));
        app.notes.push(Note::new("third note".to_string(), "test-scope".to_string()));
        app
    }

    #[test]
    fn test_selection_clamps() {
        let mut app = App::new("test-scope".to_string(), vec![]);
        app.move_selection_down();
        assert_eq!(app.selected, 0);
        app.move_selection_up();
        assert_eq!(app.selected, 0);

        let mut app = setup_app();
        app.move_selection_up();
        assert_eq!(app.selected, 0);
        
        app.move_selection_down();
        assert_eq!(app.selected, 1);
        app.move_selection_down();
        assert_eq!(app.selected, 2);
        app.move_selection_down();
        assert_eq!(app.selected, 2); // clamped
    }

    #[test]
    fn test_add_flow() {
        let mut app = App::new("test-scope".to_string(), vec![]);
        app.start_add();
        assert_eq!(app.mode, Mode::Adding);
        
        app.input_buffer.push_str("new note");
        let note = app.confirm_add();
        assert!(note.is_some());
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.notes.len(), 1);
        assert_eq!(app.notes[0].text, "new note");
        assert!(app.input_buffer.is_empty());

        app.start_add();
        app.input_buffer.push_str("   ");
        let none = app.confirm_add();
        assert!(none.is_none());
        assert_eq!(app.mode, Mode::Adding);
        assert_eq!(app.notes.len(), 1);

        app.cancel_add();
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn test_toggle_done() {
        let mut app = setup_app();
        assert!(!app.notes[0].done);
        app.toggle_selected_done();
        assert!(app.notes[0].done);
        
        app.move_selection_down();
        app.toggle_selected_done();
        assert!(app.notes[1].done);
    }

    #[test]
    fn test_delete_flow() {
        let mut app = setup_app();
        app.move_selection_down(); // Select second note
        let id_to_delete = app.notes[1].id.clone();
        
        app.start_delete_confirm();
        assert_eq!(app.mode, Mode::ConfirmDelete);
        
        let deleted_id = app.confirm_delete();
        assert_eq!(deleted_id, Some(id_to_delete.clone()));
        assert_eq!(app.notes.len(), 2);
        assert!(!app.notes.iter().any(|n| n.id == id_to_delete));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.selected, 1); // Clamp selection: was 1, length is 2, 1 is fine

        // Test cancel
        app.start_delete_confirm();
        app.cancel_delete();
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.notes.len(), 2);
    }

    #[test]
    fn test_search_filtering() {
        let mut app = setup_app();
        app.search_query = "second".to_string();
        let visible = app.visible_notes();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].text, "second note #tag");

        app.search_query = "TAG".to_string(); // case insensitive tag match
        let visible = app.visible_notes();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].text, "second note #tag");

        app.search_query = "note".to_string();
        let visible = app.visible_notes();
        assert_eq!(visible.len(), 3);
    }

    #[test]
    fn test_operations_with_filter() {
        let mut app = setup_app();
        app.search_query = "second".to_string();
        app.clamp_selection(); // Emulate apply_search
        
        // Only the second note is visible, selected index 0 corresponds to it.
        app.toggle_selected_done();
        assert!(app.notes[1].done);
        assert!(!app.notes[0].done);

        let id_to_delete = app.notes[1].id.clone();
        app.start_delete_confirm();
        let deleted = app.confirm_delete();
        assert_eq!(deleted, Some(id_to_delete.clone()));
        
        assert_eq!(app.notes.len(), 2);
        assert!(!app.notes.iter().any(|n| n.id == id_to_delete));
    }

    #[test]
    fn test_toggle_global_view() {
        let mut app = setup_app();
        assert!(!app.show_global);
        app.toggle_global_view();
        assert!(app.show_global);
    }
}
