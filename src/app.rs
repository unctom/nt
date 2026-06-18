/// Application state and business logic.
use crate::model::Note;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Represents the current mode of the application UI.
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Adding,
    Editing,
    Searching,
    ConfirmDelete,
}

/// The main application state containing notes and UI context.
pub struct App {
    /// List of notes currently loaded.
    pub notes: Vec<Note>,
    /// The active scope.
    pub scope: String,
    /// Selected item index in the visible list.
    pub selected: usize,
    /// Current input mode.
    pub mode: Mode,
    /// Buffer for adding a new note.
    pub input_buffer: String,
    /// Buffer for search query.
    pub search_query: String,
    /// Set of multi-selected note IDs for bulk actions.
    pub multi_selected: std::collections::HashSet<String>,
    /// Whether global view is active.
    pub show_global: bool,
    /// Whether the application should terminate.
    pub should_quit: bool,
}

impl App {
    /// Constructs a new App state.
    pub fn new(scope: String, notes: Vec<Note>) -> Self {
        Self {
            notes,
            scope,
            selected: 0,
            mode: Mode::Normal,
            input_buffer: String::new(),
            search_query: String::new(),
            multi_selected: std::collections::HashSet::new(),
            show_global: false,
            should_quit: false,
        }
    }

    /// Returns a vector of references to notes matching the current search filter.
    pub fn visible_notes(&self) -> Vec<&Note> {
        if self.search_query.trim().is_empty() {
            return self.notes.iter().collect();
        }

        let matcher = SkimMatcherV2::default();
        let query = self.search_query.to_lowercase();

        let mut matched: Vec<(&Note, i64)> = self
            .notes
            .iter()
            .filter_map(|note| {
                // Check if tags match first
                let tag_match = note.tags.iter().any(|t| matcher.fuzzy_match(&t.to_lowercase(), &query).is_some());
                if tag_match {
                    return Some((note, 100)); // Arbitrary high score for tag matches
                }
                
                // Then check text
                matcher.fuzzy_match(&note.text.to_lowercase(), &query).map(|score| (note, score))
            })
            .collect();

        // Sort by score descending (stable sort to keep original order for same score)
        matched.sort_by_key(|b| std::cmp::Reverse(b.1));

        matched.into_iter().map(|(n, _)| n).collect()
    }

    fn clamp_selection(&mut self) {
        let count = self.visible_notes().len();
        if count == 0 {
            self.selected = 0;
        } else if self.selected >= count {
            self.selected = count - 1;
        }
    }

    /// Moves the list selection down by one, clamping at the bottom.
    pub fn move_selection_down(&mut self) {
        let count = self.visible_notes().len();
        if count > 0 && self.selected < count - 1 {
            self.selected += 1;
        }
    }

    /// Moves the list selection up by one, clamping at the top.
    pub fn move_selection_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Transitions to the Adding mode.
    pub fn start_add(&mut self) {
        self.mode = Mode::Adding;
        self.input_buffer.clear();
    }

    /// Confirms addition of a new note if the buffer is not empty.
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

    /// Cancels the add operation.
    pub fn cancel_add(&mut self) {
        self.input_buffer.clear();
        self.mode = Mode::Normal;
    }

    /// Transitions to the Editing mode.
    pub fn start_edit(&mut self) {
        let visible = self.visible_notes();
        if self.selected < visible.len() {
            self.input_buffer = visible[self.selected].text.clone();
            self.mode = Mode::Editing;
        }
    }

    /// Confirms the edit operation.
    pub fn confirm_edit(&mut self) -> Option<Note> {
        let text = self.input_buffer.trim();
        if text.is_empty() {
            return None;
        }

        let visible = self.visible_notes();
        if self.selected < visible.len() {
            let id = visible[self.selected].id.clone();
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.text = text.to_string();
                note.tags = crate::model::extract_tags(text);
                
                let cloned = note.clone();
                self.input_buffer.clear();
                self.mode = Mode::Normal;
                self.clamp_selection();
                return Some(cloned);
            }
        }
        None
    }

    /// Cancels the edit operation.
    pub fn cancel_edit(&mut self) {
        self.input_buffer.clear();
        self.mode = Mode::Normal;
    }

    /// Toggles the 'done' state of the selected notes.
    pub fn toggle_selected_done(&mut self) {
        if !self.multi_selected.is_empty() {
            for note in self.notes.iter_mut() {
                if self.multi_selected.contains(&note.id) {
                    note.toggle_done();
                }
            }
            self.multi_selected.clear();
            return;
        }

        let visible = self.visible_notes();
        if self.selected < visible.len() {
            let id = visible[self.selected].id.clone();
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.toggle_done();
            }
        }
    }

    /// Toggles multi-selection for the current cursor note.
    pub fn toggle_multi_select(&mut self) {
        let visible = self.visible_notes();
        if self.selected < visible.len() {
            let id = visible[self.selected].id.clone();
            if self.multi_selected.contains(&id) {
                self.multi_selected.remove(&id);
            } else {
                self.multi_selected.insert(id);
            }
        }
    }

    /// Prepares to delete the selected note.
    pub fn start_delete_confirm(&mut self) {
        let visible = self.visible_notes();
        if self.selected < visible.len() {
            self.mode = Mode::ConfirmDelete;
        }
    }

    /// Confirms deletion of the selected note(s).
    pub fn confirm_delete(&mut self) -> Option<String> {
        if self.mode != Mode::ConfirmDelete {
            return None;
        }
        
        if !self.multi_selected.is_empty() {
            self.notes.retain(|n| !self.multi_selected.contains(&n.id));
            self.multi_selected.clear();
            self.mode = Mode::Normal;
            self.clamp_selection();
            // We just return one of the deleted IDs to indicate success.
            // A more complete implementation would return a Vec<String>.
            return Some("bulk".to_string());
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

    /// Cancels the delete operation.
    pub fn cancel_delete(&mut self) {
        self.mode = Mode::Normal;
    }

    /// Switches to the searching mode.
    pub fn start_search(&mut self) {
        self.mode = Mode::Searching;
    }

    /// Finalizes the search mode.
    pub fn apply_search(&mut self) {
        self.mode = Mode::Normal;
        self.clamp_selection();
    }

    /// Clears the current search query and resets the view.
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.mode = Mode::Normal;
        self.clamp_selection();
    }

    /// Toggles between the current scope and global view.
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
        app.notes.push(Note::new(
            "first note".to_string(),
            "test-scope".to_string(),
        ));
        app.notes.push(Note::new(
            "second note #tag".to_string(),
            "test-scope".to_string(),
        ));
        app.notes.push(Note::new(
            "third note".to_string(),
            "test-scope".to_string(),
        ));
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

    #[test]
    fn test_edit_flow() {
        let mut app = setup_app();
        app.selected = 1; // "second note #tag"
        
        app.start_edit();
        assert_eq!(app.mode, Mode::Editing);
        assert_eq!(app.input_buffer, "second note #tag");

        app.input_buffer = "edited note #new".to_string();
        let note = app.confirm_edit();
        
        assert!(note.is_some());
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.notes[1].text, "edited note #new");
        assert_eq!(app.notes[1].tags, vec!["new".to_string()]);
    }
}
