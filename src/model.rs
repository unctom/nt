/// Note and Tag parsing model.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Represents a single note.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Note {
    /// Unique identifier for the note.
    pub id: String,
    /// The original text content of the note.
    pub text: String,
    /// Extracted tags.
    pub tags: Vec<String>,
    /// The scope this note belongs to.
    pub scope: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Whether the note is marked as done.
    pub done: bool,
}

impl Note {
    /// Creates a new Note with a generated ULID, parsing tags from the text.
    pub fn new(text: String, scope: String) -> Self {
        let tags = extract_tags(&text);
        Self {
            id: Ulid::new().to_string(),
            text,
            tags,
            scope,
            created_at: Utc::now(),
            done: false,
        }
    }

    /// Toggles the done state of the note.
    pub fn toggle_done(&mut self) {
        self.done = !self.done;
    }
}

/// Extracts tags from a given text string.
/// Tags are alphanumeric strings prefixed with a `#`.
pub fn extract_tags(text: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut current_tag = String::new();
    let mut in_tag = false;

    for c in text.chars() {
        if in_tag {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                current_tag.push(c);
            } else {
                if !current_tag.is_empty() {
                    let tag = current_tag.to_lowercase();
                    if !tags.contains(&tag) {
                        tags.push(tag);
                    }
                }
                current_tag.clear();
                in_tag = c == '#';
            }
        } else if c == '#' {
            in_tag = true;
        }
    }

    if in_tag && !current_tag.is_empty() {
        let tag = current_tag.to_lowercase();
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tags_no_tags() {
        assert_eq!(extract_tags("hello world"), Vec::<String>::new());
    }

    #[test]
    fn test_extract_tags_one_tag() {
        assert_eq!(extract_tags("fix bug #urgent"), vec!["urgent".to_string()]);
    }

    #[test]
    fn test_extract_tags_multiple_tags() {
        assert_eq!(
            extract_tags("fix bug #urgent #now"),
            vec!["urgent".to_string(), "now".to_string()]
        );
    }

    #[test]
    fn test_extract_tags_deduplicates() {
        assert_eq!(extract_tags("#bug stuff #bug"), vec!["bug".to_string()]);
    }

    #[test]
    fn test_extract_tags_hyphens_underscores() {
        assert_eq!(
            extract_tags("#high-priority #needs_review"),
            vec!["high-priority".to_string(), "needs_review".to_string()]
        );
    }

    #[test]
    fn test_extract_tags_bare_hash() {
        assert_eq!(extract_tags("this is # a test"), Vec::<String>::new());
    }

    #[test]
    fn test_extract_tags_case_insensitive() {
        assert_eq!(extract_tags("#URGENT"), vec!["urgent".to_string()]);
    }

    #[test]
    fn test_note_new() {
        let note = Note::new("fix bug #urgent".to_string(), "test-scope".to_string());
        assert_eq!(note.tags, vec!["urgent".to_string()]);
        assert_eq!(note.text, "fix bug #urgent");
        assert_eq!(note.scope, "test-scope");
        assert!(!note.done);
    }

    #[test]
    fn test_note_toggle_done() {
        let mut note = Note::new("test".to_string(), "scope".to_string());
        assert!(!note.done);
        note.toggle_done();
        assert!(note.done);
        note.toggle_done();
        assert!(!note.done);
    }

    #[test]
    fn test_note_serialization() {
        let note = Note::new("test #tag".to_string(), "scope".to_string());
        let json = serde_json::to_string(&note).unwrap();
        let deserialized: Note = serde_json::from_str(&json).unwrap();
        assert_eq!(note, deserialized);
    }
}
