mod export;
mod scan;

pub use export::{evaluate_selection, preview_export, run_export};
pub use scan::{scan_children, scan_tree};

fn validate_root_path(root_path: &str) -> Result<(), String> {
    if root_path.trim().is_empty() {
        return Err("rootPath is required".to_string());
    }
    Ok(())
}

fn validate_output_path(output_path: &str) -> Result<(), String> {
    if output_path.trim().is_empty() {
        return Err("outputPath is required".to_string());
    }
    Ok(())
}
