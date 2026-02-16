use crate::application::scanner::{scan_children as scan_children_impl, scan_root};
use crate::models::{ScanLimits, TreeNode};

use super::validate_root_path;

#[tauri::command]
pub fn scan_tree(root_path: String, use_gitignore: bool) -> Result<TreeNode, String> {
    validate_root_path(&root_path)?;
    let limits = ScanLimits::default();
    scan_root(&root_path, use_gitignore, &limits)
}

#[tauri::command]
pub fn scan_children(
    root_path: String,
    dir_path: String,
    use_gitignore: bool,
) -> Result<Vec<TreeNode>, String> {
    validate_root_path(&root_path)?;
    let limits = ScanLimits::default();
    let batch = scan_children_impl(&root_path, &dir_path, use_gitignore, &limits)?;
    Ok(batch.nodes)
}
