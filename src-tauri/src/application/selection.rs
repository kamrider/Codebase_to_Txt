use std::path::PathBuf;
use std::{fs, path::Path};

use walkdir::WalkDir;

use crate::domain::rules::{Decision, RuleEngine};
use crate::infrastructure::pathing::{canonicalize_dir, relative_unix_path};
use crate::infrastructure::sorting::compare_entries;
use crate::models::{ExportConfig, ScanLimits};

#[derive(Debug, Clone)]
pub struct SelectedFile {
    pub abs_path: PathBuf,
    pub rel_path: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct SelectionRun {
    pub files: Vec<SelectedFile>,
    pub included_files: usize,
    pub excluded_files: usize,
    pub warnings: Vec<String>,
}

pub fn collect_selected_files(config: &ExportConfig, limits: &ScanLimits) -> Result<SelectionRun, String> {
    let root = canonicalize_dir(&config.root_path)?;
    let engine = RuleEngine::from_config(&root, config)?;

    let mut files = Vec::new();
    let mut included = 0usize;
    let mut excluded = 0usize;
    let mut warnings = engine.warnings().to_vec();
    let mut depth_warning_emitted = false;

    let walker = WalkDir::new(&root)
        .follow_links(false)
        .max_depth(limits.max_depth)
        .sort_by(|a, b| compare_entries(a.path(), a.file_type().is_dir(), b.path(), b.file_type().is_dir()));

    for entry in walker.into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path == root {
            continue;
        }

        let rel_path = relative_unix_path(&root, path)?;
        let is_dir = entry.file_type().is_dir();

        if is_dir && entry.depth() >= limits.max_depth && !depth_warning_emitted {
            if dir_has_descendants(path) {
                warnings.push(format!(
                    "Reached maxDepth limit ({}). Skipped deeper traversal.",
                    limits.max_depth
                ));
                depth_warning_emitted = true;
            }
        }

        let decision = engine.should_include(&rel_path, path, is_dir);
        if is_dir {
            if matches!(decision, Decision::Exclude) {
                continue;
            }
            continue;
        }

        match decision {
            Decision::Include => {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                files.push(SelectedFile {
                    abs_path: path.to_path_buf(),
                    rel_path,
                    size,
                });
                included += 1;
            }
            Decision::Exclude => {
                excluded += 1;
            }
        }

        if included + excluded >= limits.max_files {
            warnings.push(format!(
                "Reached maxFiles limit ({}). Remaining files were skipped.",
                limits.max_files
            ));
            break;
        }
    }

    files.sort_by(|a, b| {
        let a_lower = a.rel_path.to_lowercase();
        let b_lower = b.rel_path.to_lowercase();
        let primary = a_lower.cmp(&b_lower);
        if primary == std::cmp::Ordering::Equal {
            a.rel_path.cmp(&b.rel_path)
        } else {
            primary
        }
    });

    Ok(SelectionRun {
        files,
        included_files: included,
        excluded_files: excluded,
        warnings,
    })
}

fn dir_has_descendants(path: &Path) -> bool {
    fs::read_dir(path)
        .ok()
        .and_then(|mut iter| iter.next())
        .is_some()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use tempfile::tempdir;

    use crate::models::{
        ExportConfig, LargeFileStrategy, ManualSelectionState, OutputFormat, ScanLimits,
    };

    use super::collect_selected_files;

    #[test]
    fn manual_include_overrides_gitignore_but_not_hard_exclude() {
        let root = tempdir().unwrap();
        fs::write(root.path().join(".gitignore"), "ignored.txt\n").unwrap();
        fs::write(root.path().join("ignored.txt"), "hello").unwrap();
        fs::create_dir_all(root.path().join(".git")).unwrap();
        fs::write(root.path().join(".git").join("config"), "internal").unwrap();

        let mut manual = BTreeMap::new();
        manual.insert("ignored.txt".to_string(), ManualSelectionState::Include);
        manual.insert(".git/config".to_string(), ManualSelectionState::Include);

        let config = ExportConfig {
            root_path: root.path().to_string_lossy().to_string(),
            use_gitignore: true,
            include_globs: vec![],
            exclude_globs: vec![],
            include_extensions: vec![],
            exclude_extensions: vec![],
            structure_only: false,
            max_file_size_kb: 1024,
            large_file_strategy: LargeFileStrategy::Truncate,
            manual_selections: manual,
            output_format: OutputFormat::Txt,
        };

        let run = collect_selected_files(&config, &ScanLimits::default()).unwrap();
        assert!(run.files.iter().any(|item| item.rel_path == "ignored.txt"));
        assert!(!run.files.iter().any(|item| item.rel_path.starts_with(".git/")));
    }

    #[test]
    fn include_rules_override_exclude_and_gitignore_when_manual_is_inherit() {
        let root = tempdir().unwrap();
        fs::write(root.path().join(".gitignore"), "ignored.txt\n").unwrap();
        fs::write(root.path().join("ignored.txt"), "ignored").unwrap();
        fs::write(root.path().join("blocked.txt"), "blocked").unwrap();
        fs::write(root.path().join("allowed.txt"), "allowed").unwrap();

        let mut manual = BTreeMap::new();
        manual.insert("ignored.txt".to_string(), ManualSelectionState::Inherit);
        manual.insert("blocked.txt".to_string(), ManualSelectionState::Inherit);

        let config = ExportConfig {
            root_path: root.path().to_string_lossy().to_string(),
            use_gitignore: true,
            include_globs: vec!["*.txt".to_string()],
            exclude_globs: vec!["blocked.txt".to_string()],
            include_extensions: vec![],
            exclude_extensions: vec![],
            structure_only: false,
            max_file_size_kb: 1024,
            large_file_strategy: LargeFileStrategy::Truncate,
            manual_selections: manual,
            output_format: OutputFormat::Txt,
        };

        let run = collect_selected_files(&config, &ScanLimits::default()).unwrap();
        let included: Vec<&str> = run.files.iter().map(|item| item.rel_path.as_str()).collect();
        assert_eq!(included, vec!["allowed.txt", "blocked.txt", "ignored.txt"]);
        assert_eq!(run.excluded_files, 1);
    }

    #[test]
    fn emits_max_files_warning_when_limit_is_hit() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("a.txt"), "a").unwrap();
        fs::write(root.path().join("b.txt"), "b").unwrap();
        fs::write(root.path().join("c.txt"), "c").unwrap();

        let config = ExportConfig {
            root_path: root.path().to_string_lossy().to_string(),
            use_gitignore: false,
            include_globs: vec![],
            exclude_globs: vec![],
            include_extensions: vec![],
            exclude_extensions: vec![],
            structure_only: false,
            max_file_size_kb: 1024,
            large_file_strategy: LargeFileStrategy::Truncate,
            manual_selections: BTreeMap::new(),
            output_format: OutputFormat::Txt,
        };
        let limits = ScanLimits {
            max_files: 2,
            max_depth: 64,
        };

        let run = collect_selected_files(&config, &limits).unwrap();
        assert_eq!(run.included_files + run.excluded_files, 2);
        assert!(run
            .warnings
            .iter()
            .any(|warning| warning.contains("Reached maxFiles limit")));
    }

    #[test]
    fn emits_max_depth_warning_when_deeper_directories_exist() {
        let root = tempdir().unwrap();
        let level1 = root.path().join("level1");
        let level2 = level1.join("level2");
        fs::create_dir_all(&level2).unwrap();
        fs::write(level2.join("deep.txt"), "deep").unwrap();

        let config = ExportConfig {
            root_path: root.path().to_string_lossy().to_string(),
            use_gitignore: false,
            include_globs: vec![],
            exclude_globs: vec![],
            include_extensions: vec![],
            exclude_extensions: vec![],
            structure_only: false,
            max_file_size_kb: 1024,
            large_file_strategy: LargeFileStrategy::Truncate,
            manual_selections: BTreeMap::new(),
            output_format: OutputFormat::Txt,
        };
        let limits = ScanLimits {
            max_files: 100_000,
            max_depth: 1,
        };

        let run = collect_selected_files(&config, &limits).unwrap();
        assert_eq!(run.included_files, 0);
        assert!(run
            .warnings
            .iter()
            .any(|warning| warning.contains("Reached maxDepth limit")));
    }
}
