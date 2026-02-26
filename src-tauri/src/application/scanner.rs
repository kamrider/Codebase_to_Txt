use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use walkdir::WalkDir;

use crate::domain::rules::{Decision, RuleEngine};
use crate::infrastructure::fs_scan::{scan_single_level, ScanBatch};
use crate::infrastructure::errors::{coded, E_DIRPATH_NOT_DIR, E_PATH_OUTSIDE_ROOT};
use crate::infrastructure::pathing::{canonicalize_dir, ensure_under_root, file_name_or_fallback};
use crate::models::{ExportConfig, ScanLimits, TreeNode};

pub fn scan_root(config: &ExportConfig, limits: &ScanLimits) -> Result<TreeNode, String> {
    let root = canonicalize_dir(&config.root_path)?;
    let engine = RuleEngine::from_config(&root, config)?;
    let gitignore = build_gitignore_matcher(&root, config.use_gitignore);
    let mut children = scan_single_level(&root, &root, limits, gitignore.as_ref())?;
    apply_rule_decisions(&root, &engine, &mut children);
    let _scan_warnings = &children.warnings;
    let root_node = TreeNode {
        path: ".".to_string(),
        name: file_name_or_fallback(&root, "workspace"),
        is_dir: true,
        children_count: Some(children.nodes.len()),
        included_by_rules: true,
        ignored_by_gitignore: false,
        children: children.nodes,
    };
    Ok(root_node)
}

pub fn scan_children(
    config: &ExportConfig,
    dir_path: &str,
    limits: &ScanLimits,
) -> Result<ScanBatch, String> {
    let root = canonicalize_dir(&config.root_path)?;
    let engine = RuleEngine::from_config(&root, config)?;
    let dir_abs = resolve_dir_under_root(&root, dir_path)?;
    let gitignore = build_gitignore_matcher(&root, config.use_gitignore);

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

    let mut batch = scan_single_level(&root, &dir_abs, limits, gitignore.as_ref())?;
    apply_rule_decisions(&root, &engine, &mut batch);
    Ok(batch)
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

fn apply_rule_decisions(root: &Path, engine: &RuleEngine, batch: &mut ScanBatch) {
    for node in &mut batch.nodes {
        let abs_path = root.join(&node.path);
        let decision = engine.should_include(&node.path, &abs_path, node.is_dir);
        node.included_by_rules = matches!(decision, Decision::Include);
    }
}

fn build_gitignore_matcher(root: &Path, enabled: bool) -> Option<Gitignore> {
    if !enabled {
        return None;
    }

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
    use std::collections::BTreeMap;
    use std::fs;

    use tempfile::tempdir;

    use crate::infrastructure::errors::{E_DIRPATH_NOT_DIR, E_PATH_OUTSIDE_ROOT};
    use crate::models::{
        ExportConfig, LargeFileStrategy, ManualSelectionState, OutputFormat, ScanLimits,
    };

    use super::{scan_children, scan_root};

    fn test_config(root_path: &str) -> ExportConfig {
        ExportConfig {
            root_path: root_path.to_string(),
            use_gitignore: true,
            include_globs: vec![],
            exclude_globs: vec![],
            include_extensions: vec![],
            exclude_extensions: vec![],
            structure_only: false,
            max_file_size_kb: 256,
            large_file_strategy: LargeFileStrategy::Truncate,
            manual_selections: BTreeMap::new(),
            output_format: OutputFormat::Txt,
        }
    }

    #[test]
    fn scan_root_returns_first_level_and_keeps_directories_lazy() {
        let root = tempdir().unwrap();
        let nested_dir = root.path().join("a_dir");
        fs::create_dir_all(&nested_dir).unwrap();
        fs::write(nested_dir.join("deep.txt"), "nested").unwrap();
        fs::write(root.path().join("root.txt"), "root").unwrap();

        let config = test_config(root.path().to_string_lossy().as_ref());
        let limits = ScanLimits::default();
        let tree = scan_root(&config, &limits).unwrap();

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

        let config = test_config(root.path().to_string_lossy().as_ref());
        let limits = ScanLimits::default();
        let result = scan_children(&config, outside.path().to_string_lossy().as_ref(), &limits);

        assert!(result.is_err());
        assert!(result.err().unwrap().contains(E_PATH_OUTSIDE_ROOT));
    }

    #[test]
    fn scan_children_rejects_file_path() {
        let root = tempdir().unwrap();
        let file_path = root.path().join("file.txt");
        fs::write(&file_path, "content").unwrap();

        let config = test_config(root.path().to_string_lossy().as_ref());
        let limits = ScanLimits::default();
        let result = scan_children(&config, "file.txt", &limits);

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

        let mut config = test_config(root.path().to_string_lossy().as_ref());
        config.use_gitignore = true;
        let limits = ScanLimits::default();
        let tree = scan_root(&config, &limits).unwrap();

        let ignored_file = tree.children.iter().find(|node| node.path == "ignored.txt").unwrap();
        let normal_file = tree.children.iter().find(|node| node.path == "normal.txt").unwrap();
        let ignored_dir = tree.children.iter().find(|node| node.path == "ignored_dir").unwrap();

        assert!(ignored_file.ignored_by_gitignore);
        assert!(ignored_dir.ignored_by_gitignore);
        assert!(!normal_file.ignored_by_gitignore);
    }

    #[test]
    fn scan_root_hides_gitignored_entries_when_disabled() {
        let root = tempdir().unwrap();
        fs::write(root.path().join(".gitignore"), "ignored.txt\n").unwrap();
        fs::write(root.path().join("ignored.txt"), "x").unwrap();
        fs::write(root.path().join("normal.txt"), "y").unwrap();

        let mut config = test_config(root.path().to_string_lossy().as_ref());
        config.use_gitignore = false;
        let limits = ScanLimits::default();
        let tree = scan_root(&config, &limits).unwrap();

        let ignored_file = tree.children.iter().find(|node| node.path == "ignored.txt").unwrap();
        let normal_file = tree.children.iter().find(|node| node.path == "normal.txt").unwrap();

        assert!(!ignored_file.ignored_by_gitignore);
        assert!(!normal_file.ignored_by_gitignore);
    }

    #[test]
    fn scan_root_include_rules_override_exclude_and_gitignore() {
        let root = tempdir().unwrap();
        fs::write(root.path().join(".gitignore"), "kept.ts\n").unwrap();
        fs::write(root.path().join("kept.ts"), "x").unwrap();

        let mut config = test_config(root.path().to_string_lossy().as_ref());
        config.use_gitignore = true;
        config.include_extensions = vec![".ts".to_string()];
        config.exclude_extensions = vec![".ts".to_string()];
        let limits = ScanLimits::default();
        let tree = scan_root(&config, &limits).unwrap();

        let node = tree.children.iter().find(|item| item.path == "kept.ts").unwrap();
        assert!(node.ignored_by_gitignore);
        assert!(node.included_by_rules);
    }

    #[test]
    fn scan_root_manual_selection_still_has_highest_priority() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("file.ts"), "x").unwrap();

        let mut config = test_config(root.path().to_string_lossy().as_ref());
        config.include_extensions = vec![".ts".to_string()];
        config.exclude_extensions = vec![".ts".to_string()];
        config
            .manual_selections
            .insert("file.ts".to_string(), ManualSelectionState::Exclude);
        let limits = ScanLimits::default();
        let tree = scan_root(&config, &limits).unwrap();

        let node = tree.children.iter().find(|item| item.path == "file.ts").unwrap();
        assert!(!node.included_by_rules);
    }
}
