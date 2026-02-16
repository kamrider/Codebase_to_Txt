use std::fs;
use std::path::{Path, PathBuf};

use ignore::gitignore::Gitignore;
use ignore::Match;

use crate::infrastructure::errors::read_error;
use crate::infrastructure::sorting::compare_entries;
use crate::models::{ScanLimits, TreeNode};

#[derive(Debug, Clone)]
pub struct ScanBatch {
    pub nodes: Vec<TreeNode>,
    pub warnings: Vec<String>,
}

pub fn scan_single_level(
    root: &Path,
    dir: &Path,
    limits: &ScanLimits,
    gitignore: Option<&Gitignore>,
) -> Result<ScanBatch, String> {
    let mut entries: Vec<(PathBuf, bool)> = Vec::new();
    let mut warnings = Vec::new();

    let reader = fs::read_dir(dir).map_err(|e| read_error("Failed to read directory", e))?;
    for item in reader {
        let item = item.map_err(|e| read_error("Failed to read directory entry", e))?;
        let file_type = item
            .file_type()
            .map_err(|e| read_error("Failed to read file type", e))?;
        entries.push((item.path(), file_type.is_dir()));
        if entries.len() >= limits.max_files {
            warnings.push(format!(
                "Reached maxFiles limit ({}). Remaining entries were skipped.",
                limits.max_files
            ));
            break;
        }
    }

    entries.sort_by(|(a, a_dir), (b, b_dir)| compare_entries(a, *a_dir, b, *b_dir));

    let mut nodes = Vec::with_capacity(entries.len());
    for (entry_path, is_dir) in entries {
        let ignored_by_gitignore = matches!(
            gitignore.map(|matcher| matcher.matched_path_or_any_parents(&entry_path, is_dir)),
            Some(Match::Ignore(_))
        );
        let rel = entry_path
            .strip_prefix(root)
            .map_err(|_| read_error("Failed to derive relative path", "path not under root"))?;
        let rel_text = rel.to_string_lossy().replace('\\', "/");
        let name = entry_path
            .file_name()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_else(|| rel_text.clone());

        nodes.push(TreeNode {
            path: rel_text,
            name,
            is_dir,
            children_count: if is_dir { None } else { Some(0) },
            ignored_by_gitignore,
            children: vec![],
        });
    }

    Ok(ScanBatch { nodes, warnings })
}
