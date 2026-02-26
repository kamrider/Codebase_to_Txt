use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::Match;
use walkdir::WalkDir;

use crate::infrastructure::errors::{coded, E_RULE_INVALID_GLOB};
use crate::models::{ExportConfig, ManualSelectionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Include,
    Exclude,
}

pub struct RuleEngine {
    include_globs: Option<GlobSet>,
    exclude_globs: Option<GlobSet>,
    include_ext: HashSet<String>,
    exclude_ext: HashSet<String>,
    manual: BTreeMap<String, ManualSelectionState>,
    gitignore: Option<Gitignore>,
    use_gitignore: bool,
    warnings: Vec<String>,
}

impl RuleEngine {
    pub fn from_config(root: &Path, config: &ExportConfig) -> Result<Self, String> {
        let include_globs = compile_globset(&config.include_globs)?;
        let exclude_globs = compile_globset(&config.exclude_globs)?;
        let include_ext = normalize_extensions(&config.include_extensions);
        let exclude_ext = normalize_extensions(&config.exclude_extensions);
        let manual = normalize_manual_selections(&config.manual_selections);
        let (gitignore, warnings) = if config.use_gitignore {
            build_gitignore_matcher(root)
        } else {
            (None, vec![])
        };

        Ok(Self {
            include_globs,
            exclude_globs,
            include_ext,
            exclude_ext,
            manual,
            gitignore,
            use_gitignore: config.use_gitignore,
            warnings,
        })
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub fn should_include(&self, rel_path: &str, abs_path: &Path, is_dir: bool) -> Decision {
        if is_hard_excluded(rel_path) {
            return Decision::Exclude;
        }

        if let Some(manual_state) = self.manual_state_for(rel_path) {
            match manual_state {
                ManualSelectionState::Include => return Decision::Include,
                ManualSelectionState::Exclude => return Decision::Exclude,
                ManualSelectionState::Inherit => {}
            }
        }

        let include_glob_match = self.matches_include_glob(rel_path);
        if matches!(include_glob_match, Some(false)) {
            return Decision::Exclude;
        }

        let include_ext_match = if is_dir {
            None
        } else {
            self.matches_include_extension(rel_path)
        };
        if matches!(include_ext_match, Some(false)) {
            return Decision::Exclude;
        }

        let include_rule_matched =
            matches!(include_glob_match, Some(true)) || matches!(include_ext_match, Some(true));
        if include_rule_matched {
            return Decision::Include;
        }

        if !is_dir && self.matches_exclude_extension(rel_path) {
            return Decision::Exclude;
        }

        if self.matches_exclude_glob(rel_path) {
            return Decision::Exclude;
        }

        if self.use_gitignore {
            if let Some(gi) = &self.gitignore {
                if matches!(gi.matched_path_or_any_parents(abs_path, is_dir), Match::Ignore(_)) {
                    return Decision::Exclude;
                }
            }
        }

        Decision::Include
    }

    fn matches_include_glob(&self, rel_path: &str) -> Option<bool> {
        self.include_globs.as_ref().map(|set| set.is_match(rel_path))
    }

    fn matches_exclude_glob(&self, rel_path: &str) -> bool {
        if let Some(set) = &self.exclude_globs {
            return set.is_match(rel_path);
        }
        false
    }

    fn matches_include_extension(&self, rel_path: &str) -> Option<bool> {
        if self.include_ext.is_empty() {
            return None;
        }
        let lower = rel_path.to_lowercase();
        Some(self.include_ext.iter().any(|ext| lower.ends_with(ext)))
    }

    fn matches_exclude_extension(&self, rel_path: &str) -> bool {
        if self.exclude_ext.is_empty() {
            return false;
        }
        let lower = rel_path.to_lowercase();
        self.exclude_ext.iter().any(|ext| lower.ends_with(ext))
    }

    fn manual_state_for(&self, rel_path: &str) -> Option<ManualSelectionState> {
        let key = normalize_key(rel_path);
        if let Some(state) = self.manual.get(&key) {
            return Some(state.clone());
        }

        let mut best: Option<(usize, ManualSelectionState)> = None;
        for (manual_key, state) in &self.manual {
            if key == *manual_key
                || (key.starts_with(manual_key) && key.chars().nth(manual_key.len()) == Some('/'))
            {
                let score = manual_key.len();
                if let Some((best_score, _)) = &best {
                    if score <= *best_score {
                        continue;
                    }
                }
                best = Some((score, state.clone()));
            }
        }
        best.map(|(_, state)| state)
    }
}

pub fn is_hard_excluded(rel_path: &str) -> bool {
    let normalized = normalize_key(rel_path);
    normalized == ".git" || normalized.starts_with(".git/")
}

fn normalize_manual_selections(
    source: &BTreeMap<String, ManualSelectionState>,
) -> BTreeMap<String, ManualSelectionState> {
    source
        .iter()
        .map(|(k, v)| (normalize_key(k), v.clone()))
        .collect()
}

fn normalize_extensions(items: &[String]) -> HashSet<String> {
    items
        .iter()
        .map(|v| v.trim().to_lowercase())
        .filter(|v| !v.is_empty())
        .map(|v| if v.starts_with('.') { v } else { format!(".{v}") })
        .collect()
}

fn compile_globset(patterns: &[String]) -> Result<Option<GlobSet>, String> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern)
            .map_err(|e| coded(E_RULE_INVALID_GLOB, format!("Invalid glob '{pattern}': {e}")))?;
        builder.add(glob);
    }
    let set = builder
        .build()
        .map_err(|e| coded(E_RULE_INVALID_GLOB, format!("Failed to build glob matcher: {e}")))?;
    Ok(Some(set))
}

fn build_gitignore_matcher(root: &Path) -> (Option<Gitignore>, Vec<String>) {
    let mut builder = GitignoreBuilder::new(root);
    let mut warnings = Vec::new();
    let mut has_patterns = false;

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name().to_string_lossy() != ".gitignore" {
            continue;
        }
        has_patterns = true;
        if let Some(error) = builder.add(entry.path()) {
            warnings.push(format!("Partial .gitignore parse error: {error}"));
        }
    }

    if !has_patterns {
        return (None, warnings);
    }

    match builder.build() {
        Ok(matcher) => (Some(matcher), warnings),
        Err(error) => {
            warnings.push(format!("Failed to build .gitignore matcher: {error}"));
            (None, warnings)
        }
    }
}

fn normalize_key(input: &str) -> String {
    input
        .trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_matches('/')
        .to_string()
}
