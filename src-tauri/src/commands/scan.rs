use crate::models::TreeNode;

use super::validate_root_path;

#[tauri::command]
pub fn scan_tree(root_path: String) -> Result<TreeNode, String> {
    validate_root_path(&root_path)?;

    let normalized_path = root_path.trim().replace('\\', "/");
    let directory_name = normalized_path
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("workspace")
        .to_string();

    Ok(TreeNode {
        path: normalized_path,
        name: directory_name,
        is_dir: true,
        children_count: 0,
        children: vec![],
    })
}
