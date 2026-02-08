use crate::application::exporter::{
    evaluate_selection as evaluate_selection_impl, preview_export as preview_export_impl,
    run_export as run_export_impl,
};
use crate::models::{ExportConfig, ExportResult, PreviewMeta, ScanLimits, SelectionSummary};

use super::{validate_output_path, validate_root_path};

#[tauri::command]
pub fn evaluate_selection(config: ExportConfig) -> Result<SelectionSummary, String> {
    validate_root_path(&config.root_path)?;
    let limits = ScanLimits::default();
    evaluate_selection_impl(&config, &limits)
}

#[tauri::command]
pub fn preview_export(config: ExportConfig) -> Result<PreviewMeta, String> {
    validate_root_path(&config.root_path)?;
    let limits = ScanLimits::default();
    preview_export_impl(&config, &limits)
}

#[tauri::command]
pub fn run_export(config: ExportConfig, output_path: String) -> Result<ExportResult, String> {
    validate_root_path(&config.root_path)?;
    validate_output_path(&output_path)?;
    let limits = ScanLimits::default();
    run_export_impl(&config, &output_path, &limits)
}
