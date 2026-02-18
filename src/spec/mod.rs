mod commands;
mod config;
pub(crate) mod dashboard;
mod format;
mod init;
pub(crate) mod summary;
pub(crate) mod templates;

// Re-export public API (keeps `spec::function_name` working from main.rs)
pub use commands::{check_task, delete, edit, list, new_spec, status, view};
pub use config::{config_list, config_remove, config_set};
pub use format::{format_all_specs, format_spec};
pub use init::init;
pub use templates::list_templates;

use std::fs;
use std::path::PathBuf;

use clap_complete::engine::CompletionCandidate;
use serde::Deserialize;

const SPECS_DIR: &str = ".specs";
const TIMESTAMP_PREFIX_LEN: usize = 17; // "YYYY-MM-DD-HH-MM-"

/// Walk up from the current directory looking for a `.specs/` directory.
fn discover_specs_dir() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join(SPECS_DIR);
        if candidate.is_dir() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Walk up from the current directory looking for a `.git` directory (git repo root).
pub(crate) fn discover_git_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join(".git").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub(crate) fn specs_dir() -> PathBuf {
    discover_specs_dir().unwrap_or_else(|| PathBuf::from(SPECS_DIR))
}

/// Extract spec name from a filename like `2025-02-17-09-36-hello-world.md`
pub(crate) fn extract_spec_name(filename: &str) -> Option<&str> {
    if filename.len() > TIMESTAMP_PREFIX_LEN + 3 && filename.ends_with(".md") {
        Some(&filename[TIMESTAMP_PREFIX_LEN..filename.len() - 3])
    } else {
        None
    }
}

/// Collect all spec .md file paths from `.specs/` and its immediate subdirectories.
pub(crate) fn collect_spec_files() -> Result<Vec<PathBuf>, String> {
    let dir = specs_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let entries =
        fs::read_dir(&dir).map_err(|e| format!("Failed to read .specs/ directory: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
        let path = entry.path();
        if path.is_dir() {
            // Skip the templates directory — it holds spec templates, not specs
            if path.file_name().is_some_and(|n| n == "templates") {
                continue;
            }
            // One level of subdirectories
            if let Ok(sub_entries) = fs::read_dir(&path) {
                for sub_entry in sub_entries.flatten() {
                    let sub_path = sub_entry.path();
                    if sub_path.extension().is_some_and(|ext| ext == "md") {
                        files.push(sub_path);
                    }
                }
            }
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }

    Ok(files)
}

/// Find the spec file matching the given name (exact match on the name portion).
/// Searches `.specs/` and its immediate subdirectories.
pub(crate) fn find_spec(name: &str) -> Result<PathBuf, String> {
    let dir = specs_dir();
    if !dir.exists() {
        return Err("No .specs/ directory found".into());
    }

    let files = collect_spec_files()?;
    let mut matches: Vec<PathBuf> = files
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|f| f.to_str())
                .and_then(|f| extract_spec_name(f))
                == Some(name)
        })
        .collect();

    match matches.len() {
        0 => Err(format!("No spec found matching '{name}'")),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => {
            // Multiple files with same name but different timestamps — use the most recent
            matches.sort();
            Ok(matches.into_iter().last().unwrap())
        }
    }
}

/// Provide spec name completions for shell tab completion.
pub fn complete_spec_names(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let current = current.to_string_lossy();

    let Ok(files) = collect_spec_files() else {
        return Vec::new();
    };

    files
        .iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|f| f.to_str())
                .and_then(|f| extract_spec_name(f))
                .map(|name| name.to_string())
        })
        .filter(|name| name.starts_with(current.as_ref()))
        .map(CompletionCandidate::new)
        .collect()
}

// ---------------------------------------------------------------------------
// Front matter
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub(crate) struct FrontMatter {
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) applications: Vec<String>,
}

pub(crate) fn parse_front_matter(content: &str) -> Option<FrontMatter> {
    let content = content.strip_prefix("---\n")?;
    let end = content.find("\n---")?;
    let yaml = &content[..end];
    serde_yaml::from_str(yaml).ok()
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

pub(crate) fn validate_kebab_case(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Spec name cannot be empty".into());
    }

    let valid = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--");

    if !valid {
        return Err(format!(
            "Invalid spec name '{name}'. Names must be kebab-case \
             (lowercase letters, numbers, and single hyphens). Example: my-feature"
        ));
    }

    Ok(())
}

/// Parse a spec input that may include a group prefix (e.g. `v1/feature`).
/// Returns (group, name) where group is None for ungrouped specs.
/// Only single-level grouping is supported.
pub(crate) fn parse_spec_input(input: &str) -> Result<(Option<&str>, &str), String> {
    if let Some((group, name)) = input.split_once('/') {
        if name.contains('/') {
            return Err(
                "Only single-level grouping is supported (e.g. v1/feature, not v1/sub/feature)"
                    .into(),
            );
        }
        validate_kebab_case(group).map_err(|_| {
            format!(
                "Invalid group name '{group}'. Group names must be kebab-case \
                 (lowercase letters, numbers, and single hyphens)."
            )
        })?;
        validate_kebab_case(name)?;
        Ok((Some(group), name))
    } else {
        validate_kebab_case(input)?;
        Ok((None, input))
    }
}
