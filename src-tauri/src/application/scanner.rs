use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use walkdir::WalkDir;

use crate::infrastructure::fs_scan::{scan_single_level, ScanBatch};
use crate::infrastructure::errors::{coded, E_DIRPATH_NOT_DIR, E_PATH_OUTSIDE_ROOT};
use crate::infrastructure::pathing::{canonicalize_dir, ensure_under_root, file_name_or_fallback};
use crate::models::{ScanLimits, TreeNode};

pub fn scan_root(root_path: &str, use_gitignore: bool, limits: &ScanLimits) -> Result<TreeNode, String> {
    let root = canonicalize_dir(root_path)?;
    let gitignore = if use_gitignore {
        build_gitignore_matcher(&root)
    } else {
        None
    };
    let children = scan_single_level(&root, &root, limits, gitignore.as_ref())?;
    let _scan_warnings = &children.warnings;
    let root_node = TreeNode {
        path: ".".to_string(),
        name: file_name_or_fallback(&root, "workspace"),
        is_dir: true,
        children_count: Some(children.nodes.len()),
        ignored_by_gitignore: false,
        children: children.nodes,
    };
    Ok(root_node)
}

pub fn scan_children(
    root_path: &str,
    dir_path: &str,
    use_gitignore: bool,
    limits: &ScanLimits,
) -> Result<ScanBatch, String> {
    let root = canonicalize_dir(root_path)?;
    let dir_abs = resolve_dir_under_root(&root, dir_path)?;
    let gitignore = if use_gitignore {
        build_gitignore_matcher(&root)
    } else {
        None
    };

    let depth = depth_from_root(&root, &dir_abs)?;
    if depth >= limits.max_depth {
        return Ok(ScanBatch {
            nodes: vec![],
            warnings: vec![format!(
                "Reached maxDepth limit ({}). Skipped deeper traversal.",
                limits.max_depth
            )],
        });
    }

    scan_single_level(&root, &dir_abs, limits, gitignore.as_ref())
}

fn resolve_dir_under_root(root: &Path, dir_path: &str) -> Result<PathBuf, String> {
    let trimmed = dir_path.trim();
    if trimmed.is_empty() || trimmed == "." {
        return Ok(root.to_path_buf());
    }

    let candidate = Path::new(trimmed);
    let canonical = if candidate.is_absolute() {
        ensure_under_root(root, candidate)?
    } else {
        let joined = root.join(candidate);
        ensure_under_root(root, &joined)?
    };

    if !canonical.is_dir() {
        return Err(coded(E_DIRPATH_NOT_DIR, "dirPath must be a directory"));
    }

    Ok(canonical)
}

fn depth_from_root(root: &Path, target: &Path) -> Result<usize, String> {
    let rel = target
        .strip_prefix(root)
        .map_err(|_| coded(E_PATH_OUTSIDE_ROOT, "Path is outside of rootPath"))?;
    Ok(rel.components().count())
}

fn build_gitignore_matcher(root: &Path) -> Option<Gitignore> {
    let mut builder = GitignoreBuilder::new(root);
    let mut has_patterns = false;

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name().to_string_lossy() != ".gitignore" {
            continue;
        }
        has_patterns = true;
        let _ = builder.add(entry.path());
    }

    if !has_patterns {
        return None;
    }

    builder.build().ok()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use crate::infrastructure::errors::{E_DIRPATH_NOT_DIR, E_PATH_OUTSIDE_ROOT};
    use crate::models::ScanLimits;

    use super::{scan_children, scan_root};

    #[test]
    fn scan_root_returns_first_level_and_keeps_directories_lazy() {
        let root = tempdir().unwrap();
        let nested_dir = root.path().join("a_dir");
        fs::create_dir_all(&nested_dir).unwrap();
        fs::write(nested_dir.join("deep.txt"), "nested").unwrap();
        fs::write(root.path().join("root.txt"), "root").unwrap();

        let limits = ScanLimits::default();
        let tree = scan_root(root.path().to_string_lossy().as_ref(), false, &limits).unwrap();

        assert_eq!(tree.path, ".");
        assert_eq!(tree.children.len(), 2);
        let dir_node = tree.children.iter().find(|node| node.is_dir).unwrap();
        assert_eq!(dir_node.children_count, None);
        assert!(dir_node.children.is_empty());
    }

    #[test]
    fn scan_children_rejects_paths_outside_root() {
        let root = tempdir().unwrap();
        let outside = tempdir().unwrap();

        let limits = ScanLimits::default();
        let result = scan_children(
            root.path().to_string_lossy().as_ref(),
            outside.path().to_string_lossy().as_ref(),
            false,
            &limits,
        );

        assert!(result.is_err());
        assert!(result.err().unwrap().contains(E_PATH_OUTSIDE_ROOT));
    }

    #[test]
    fn scan_children_rejects_file_path() {
        let root = tempdir().unwrap();
        let file_path = root.path().join("file.txt");
        fs::write(&file_path, "content").unwrap();

        let limits = ScanLimits::default();
        let result = scan_children(
            root.path().to_string_lossy().as_ref(),
            "file.txt",
            false,
            &limits,
        );

        assert!(result.is_err());
        assert!(result.err().unwrap().contains(E_DIRPATH_NOT_DIR));
    }

    #[test]
    fn scan_root_marks_gitignored_entries_when_enabled() {
        let root = tempdir().unwrap();
        fs::write(root.path().join(".gitignore"), "ignored.txt\nignored_dir/\n").unwrap();
        fs::write(root.path().join("ignored.txt"), "x").unwrap();
        fs::write(root.path().join("normal.txt"), "y").unwrap();
        fs::create_dir_all(root.path().join("ignored_dir")).unwrap();

        let limits = ScanLimits::default();
        let tree = scan_root(root.path().to_string_lossy().as_ref(), true, &limits).unwrap();

        let ignored_file = tree.children.iter().find(|node| node.path == "ignored.txt").unwrap();
        let normal_file = tree.children.iter().find(|node| node.path == "normal.txt").unwrap();
        let ignored_dir = tree.children.iter().find(|node| node.path == "ignored_dir").unwrap();

        assert!(ignored_file.ignored_by_gitignore);
        assert!(ignored_dir.ignored_by_gitignore);
        assert!(!normal_file.ignored_by_gitignore);
    }
}
