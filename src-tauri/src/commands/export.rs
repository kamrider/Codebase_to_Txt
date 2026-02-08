use crate::models::{ExportConfig, ExportResult, PreviewMeta, SelectionSummary};

use super::{validate_output_path, validate_root_path};

#[tauri::command]
pub fn evaluate_selection(config: ExportConfig) -> Result<SelectionSummary, String> {
    validate_root_path(&config.root_path)?;

    let warnings = vec![
        "Placeholder: selection engine not wired to real scanner yet.".to_string(),
        "Next step will merge .gitignore and custom rules.".to_string(),
    ];

    Ok(SelectionSummary {
        included_files: 0,
        excluded_files: 0,
        warnings,
    })
}

#[tauri::command]
pub fn preview_export(config: ExportConfig) -> Result<PreviewMeta, String> {
    validate_root_path(&config.root_path)?;

    let warnings = vec![
        "Placeholder: preview counts are mocked.".to_string(),
        "Token estimator is planned for v1.1.".to_string(),
    ];

    Ok(PreviewMeta {
        included_files: 0,
        estimated_bytes: 0,
        estimated_tokens: None,
        warnings,
    })
}

#[tauri::command]
pub fn run_export(config: ExportConfig, output_path: String) -> Result<ExportResult, String> {
    validate_root_path(&config.root_path)?;
    validate_output_path(&output_path)?;

    let notes = vec![
        "Placeholder: export writer not connected yet.".to_string(),
        "Streaming writer will be added in next implementation phase.".to_string(),
    ];

    Ok(ExportResult {
        output_path,
        exported_files: 0,
        skipped_files: 0,
        total_bytes_written: 0,
        notes,
    })
}
