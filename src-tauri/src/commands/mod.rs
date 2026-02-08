mod export;
mod scan;

use crate::infrastructure::errors::{coded, E_OUTPUT_REQUIRED, E_ROOT_REQUIRED};

pub use export::{evaluate_selection, preview_export, run_export};
pub use scan::{scan_children, scan_tree};

fn validate_root_path(root_path: &str) -> Result<(), String> {
    if root_path.trim().is_empty() {
        return Err(coded(E_ROOT_REQUIRED, "rootPath is required"));
    }
    Ok(())
}

fn validate_output_path(output_path: &str) -> Result<(), String> {
    if output_path.trim().is_empty() {
        return Err(coded(E_OUTPUT_REQUIRED, "outputPath is required"));
    }
    Ok(())
}
