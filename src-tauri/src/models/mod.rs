use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportConfig {
    pub root_path: String,
    pub use_gitignore: bool,
    pub include_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub include_extensions: Vec<String>,
    pub exclude_extensions: Vec<String>,
    pub max_file_size_kb: u64,
    pub large_file_strategy: LargeFileStrategy,
    pub manual_selections: BTreeMap<String, ManualSelectionState>,
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LargeFileStrategy {
    Truncate,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManualSelectionState {
    Include,
    Exclude,
    Inherit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Txt,
    Md,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeNode {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub children_count: Option<usize>,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionSummary {
    pub included_files: usize,
    pub excluded_files: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewMeta {
    pub included_files: usize,
    pub estimated_bytes: u64,
    pub estimated_tokens: Option<u64>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub output_path: String,
    pub exported_files: usize,
    pub skipped_files: usize,
    pub total_bytes_written: u64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScanLimits {
    pub max_files: usize,
    pub max_depth: usize,
}

impl Default for ScanLimits {
    fn default() -> Self {
        Self {
            max_files: 100_000,
            max_depth: 64,
        }
    }
}
