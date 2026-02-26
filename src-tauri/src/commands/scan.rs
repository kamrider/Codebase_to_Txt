use crate::application::scanner::{scan_children as scan_children_impl, scan_root};
use crate::models::{ExportConfig, ScanLimits, TreeNode};

use super::validate_root_path;

#[tauri::command]
pub fn scan_tree(config: ExportConfig) -> Result<TreeNode, String> {
    validate_root_path(&config.root_path)?;
    let limits = ScanLimits::default();
    scan_root(&config, &limits)
}

#[tauri::command]
pub fn scan_children(
    config: ExportConfig,
    dir_path: String,
) -> Result<Vec<TreeNode>, String> {
    validate_root_path(&config.root_path)?;
    let limits = ScanLimits::default();
    let batch = scan_children_impl(&config, &dir_path, &limits)?;
    Ok(batch.nodes)
}
