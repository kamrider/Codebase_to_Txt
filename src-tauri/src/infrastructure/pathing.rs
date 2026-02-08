use std::fs;
use std::path::{Path, PathBuf};

use crate::infrastructure::errors::{
    coded, read_error, E_PATH_OUTSIDE_ROOT, E_ROOT_INVALID, E_ROOT_NOT_DIR, E_ROOT_REQUIRED,
};

pub fn canonicalize_dir(path: &str) -> Result<PathBuf, String> {
    let raw = path.trim();
    if raw.is_empty() {
        return Err(coded(E_ROOT_REQUIRED, "rootPath is required"));
    }
    let canonical =
        fs::canonicalize(raw).map_err(|e| coded(E_ROOT_INVALID, format!("Invalid rootPath: {e}")))?;
    if !canonical.is_dir() {
        return Err(coded(E_ROOT_NOT_DIR, "rootPath must be a directory"));
    }
    Ok(canonical)
}

pub fn canonicalize_existing(path: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|e| read_error("Failed to canonicalize path", e))
}

pub fn ensure_under_root(root: &Path, candidate: &Path) -> Result<PathBuf, String> {
    let canonical = canonicalize_existing(candidate)?;
    if canonical.starts_with(root) {
        return Ok(canonical);
    }
    Err(coded(E_PATH_OUTSIDE_ROOT, "Path is outside of rootPath"))
}

pub fn relative_unix_path(root: &Path, abs: &Path) -> Result<String, String> {
    let rel = abs
        .strip_prefix(root)
        .map_err(|_| coded(E_PATH_OUTSIDE_ROOT, "Path is outside of rootPath"))?;
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

    use crate::infrastructure::errors::{E_PATH_OUTSIDE_ROOT, E_ROOT_INVALID, E_ROOT_REQUIRED};

    use super::{canonicalize_dir, ensure_under_root};

    #[test]
    fn canonicalize_dir_rejects_empty_root_path() {
        let result = canonicalize_dir("   ");
        assert!(result.is_err());
        assert!(result.err().unwrap().contains(E_ROOT_REQUIRED));
    }

    #[test]
    fn canonicalize_dir_rejects_non_existing_root_path() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let result = canonicalize_dir(missing.to_string_lossy().as_ref());
        assert!(result.is_err());
        assert!(result.err().unwrap().contains(E_ROOT_INVALID));
    }

    #[test]
    fn ensure_under_root_rejects_outside_path() {
        let root = tempdir().unwrap();
        let outside_parent = tempdir().unwrap();
        let outside = outside_parent.path().join("outside.txt");
        fs::write(&outside, "x").unwrap();

        let result = ensure_under_root(root.path(), &outside);
        assert!(result.is_err());
        assert!(result.err().unwrap().contains(E_PATH_OUTSIDE_ROOT));
    }
}
