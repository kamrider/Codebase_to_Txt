use std::fs;
use std::path::{Path, PathBuf};

pub fn canonicalize_dir(path: &str) -> Result<PathBuf, String> {
    let raw = path.trim();
    if raw.is_empty() {
        return Err("rootPath is required".to_string());
    }
    let canonical = fs::canonicalize(raw).map_err(|e| format!("Invalid rootPath: {e}"))?;
    if !canonical.is_dir() {
        return Err("rootPath must be a directory".to_string());
    }
    Ok(canonical)
}

pub fn canonicalize_existing(path: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|e| format!("Failed to canonicalize path: {e}"))
}

pub fn ensure_under_root(root: &Path, candidate: &Path) -> Result<PathBuf, String> {
    let canonical = canonicalize_existing(candidate)?;
    if canonical.starts_with(root) {
        return Ok(canonical);
    }
    Err("Path is outside of rootPath".to_string())
}

pub fn relative_unix_path(root: &Path, abs: &Path) -> Result<String, String> {
    let rel = abs
        .strip_prefix(root)
        .map_err(|_| "Failed to compute relative path".to_string())?;
    let text = rel
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string();
    Ok(text)
}

pub fn file_name_or_fallback(path: &Path, fallback: &str) -> String {
    path.file_name()
        .map(|v| v.to_string_lossy().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{canonicalize_dir, ensure_under_root};

    #[test]
    fn canonicalize_dir_rejects_empty_root_path() {
        let result = canonicalize_dir("   ");
        assert!(result.is_err());
        assert!(result.err().unwrap().contains("rootPath is required"));
    }

    #[test]
    fn canonicalize_dir_rejects_non_existing_root_path() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let result = canonicalize_dir(missing.to_string_lossy().as_ref());
        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Invalid rootPath"));
    }

    #[test]
    fn ensure_under_root_rejects_outside_path() {
        let root = tempdir().unwrap();
        let outside_parent = tempdir().unwrap();
        let outside = outside_parent.path().join("outside.txt");
        fs::write(&outside, "x").unwrap();

        let result = ensure_under_root(root.path(), &outside);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Path is outside of rootPath");
    }
}
