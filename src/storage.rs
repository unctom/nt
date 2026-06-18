use crate::model::Note;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nt")
        .join("notes")
}

pub fn load_notes(base: &Path, scope: &str) -> Vec<Note> {
    let file_path = base.join(format!("{}.json", scope));
    if !file_path.exists() {
        return Vec::new();
    }

    match fs::read_to_string(&file_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(notes) => notes,
            Err(e) => {
                eprintln!("Warning: Failed to parse notes for scope '{}': {}", scope, e);
                Vec::new()
            }
        },
        Err(e) => {
            eprintln!("Warning: Failed to read notes for scope '{}': {}", scope, e);
            Vec::new()
        }
    }
}

pub fn save_notes(base: &Path, scope: &str, notes: &[Note]) -> io::Result<()> {
    if !base.exists() {
        fs::create_dir_all(base)?;
    }

    let json = serde_json::to_string_pretty(notes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut temp_file = tempfile::NamedTempFile::new_in(base)?;
    temp_file.write_all(json.as_bytes())?;
    
    let file_path = base.join(format!("{}.json", scope));
    temp_file.persist(&file_path).map_err(|e| e.error)?;

    Ok(())
}

pub fn list_scopes(base: &Path) -> Vec<String> {
    let mut scopes = Vec::new();
    if !base.exists() {
        return scopes;
    }

    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    scopes.push(stem.to_string());
                }
            }
        }
    }

    scopes
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_nonexistent_scope() {
        let dir = tempdir().unwrap();
        let notes = load_notes(dir.path(), "missing");
        assert!(notes.is_empty());
    }

    #[test]
    fn test_save_and_load_round_trip() {
        let dir = tempdir().unwrap();
        let scope = "test_scope";
        let note = Note::new("my note".to_string(), scope.to_string());
        let notes = vec![note.clone()];

        save_notes(dir.path(), scope, &notes).unwrap();
        let loaded = load_notes(dir.path(), scope);
        assert_eq!(loaded, notes);
    }

    #[test]
    fn test_save_creates_base_dir() {
        let dir = tempdir().unwrap();
        let base = dir.path().join("some").join("nested").join("dir");
        let scope = "test_scope";
        let notes = vec![Note::new("test".to_string(), scope.to_string())];

        save_notes(&base, scope, &notes).unwrap();
        assert!(base.exists());
        let loaded = load_notes(&base, scope);
        assert_eq!(loaded.len(), 1);
    }

    #[test]
    fn test_load_corrupted_json() {
        let dir = tempdir().unwrap();
        let scope = "corrupt";
        let file_path = dir.path().join(format!("{}.json", scope));
        fs::write(&file_path, "this is not valid json").unwrap();

        let notes = load_notes(dir.path(), scope);
        assert!(notes.is_empty());
    }

    #[test]
    fn test_list_scopes_empty() {
        let dir = tempdir().unwrap();
        let scopes = list_scopes(dir.path());
        assert!(scopes.is_empty());
    }

    #[test]
    fn test_list_scopes_multiple() {
        let dir = tempdir().unwrap();
        save_notes(dir.path(), "scope1", &[]).unwrap();
        save_notes(dir.path(), "scope2", &[]).unwrap();
        
        let mut scopes = list_scopes(dir.path());
        scopes.sort();
        assert_eq!(scopes, vec!["scope1".to_string(), "scope2".to_string()]);
    }

    #[test]
    fn test_delete_to_empty() {
        let dir = tempdir().unwrap();
        let scope = "test_scope";
        let notes = vec![Note::new("test".to_string(), scope.to_string())];

        save_notes(dir.path(), scope, &notes).unwrap();
        assert_eq!(load_notes(dir.path(), scope).len(), 1);

        save_notes(dir.path(), scope, &[]).unwrap();
        assert!(load_notes(dir.path(), scope).is_empty());
    }

    #[test]
    fn test_no_leftover_tmp_files() {
        let dir = tempdir().unwrap();
        let scope = "test_scope";
        let notes = vec![Note::new("test".to_string(), scope.to_string())];

        save_notes(dir.path(), scope, &notes).unwrap();

        for entry in fs::read_dir(dir.path()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            assert_ne!(path.extension().and_then(|s| s.to_str()), Some("tmp"));
        }
    }
}
