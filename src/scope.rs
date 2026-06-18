use std::path::Path;

pub fn detect_scope(cwd: &Path) -> String {
    for ancestor in cwd.ancestors() {
        if ancestor.join(".git").is_dir() {
            if let Some(name) = ancestor.file_name().and_then(|n| n.to_str()) {
                return sanitize_scope_name(name);
            }
        }
    }

    if let Some(name) = cwd.file_name().and_then(|n| n.to_str()) {
        return sanitize_scope_name(name);
    }

    "_global".to_string()
}

fn sanitize_scope_name(name: &str) -> String {
    name.replace(
        |c| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'),
        "_",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_scope_with_git() {
        let dir = tempdir().unwrap();
        let top_dir = dir.path().join("my-repo");
        fs::create_dir(&top_dir).unwrap();
        fs::create_dir(top_dir.join(".git")).unwrap();
        let sub_dir = top_dir.join("src").join("nested");
        fs::create_dir_all(&sub_dir).unwrap();

        assert_eq!(detect_scope(&sub_dir), "my-repo");
    }

    #[test]
    fn test_detect_scope_no_git() {
        let dir = tempdir().unwrap();
        let project_dir = dir.path().join("my-project");
        fs::create_dir(&project_dir).unwrap();
        let sub_dir = project_dir.join("src");
        fs::create_dir_all(&sub_dir).unwrap();

        assert_eq!(detect_scope(&sub_dir), "src");
    }

    #[test]
    fn test_sanitize_scope_name() {
        assert_eq!(sanitize_scope_name("normal_name"), "normal_name");
        assert_eq!(sanitize_scope_name("my:weird/repo*name?"), "my_weird_repo_name_");
        assert_eq!(sanitize_scope_name("a\\b|c<d>e\"f"), "a_b_c_d_e_f");
    }

    #[test]
    fn test_detect_scope_fallback() {
        let root = Path::new("/");
        // Assuming root has no .git, or if it does, this might be flaky, but standard / has no .git.
        // It has no file_name.
        assert_eq!(detect_scope(root), "_global");
    }
}
