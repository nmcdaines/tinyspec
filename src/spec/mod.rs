mod commands;
mod config;
mod format;
mod init;

// Re-export public API (keeps `spec::function_name` working from main.rs)
pub use commands::{check_task, delete, edit, list, new_spec, status, view};
pub use config::{config_list, config_remove, config_set};
pub use format::{format_all_specs, format_spec};
pub use init::init;

use std::fs;
use std::path::PathBuf;

use clap_complete::engine::CompletionCandidate;
use serde::Deserialize;

const SPECS_DIR: &str = ".specs";
const TIMESTAMP_PREFIX_LEN: usize = 17; // "YYYY-MM-DD-HH-MM-"

pub(crate) fn specs_dir() -> PathBuf {
    PathBuf::from(SPECS_DIR)
}

/// Extract spec name from a filename like `2025-02-17-09-36-hello-world.md`
pub(crate) fn extract_spec_name(filename: &str) -> Option<&str> {
    if filename.len() > TIMESTAMP_PREFIX_LEN + 3 && filename.ends_with(".md") {
        Some(&filename[TIMESTAMP_PREFIX_LEN..filename.len() - 3])
    } else {
        None
    }
}

/// Find the spec file matching the given name (exact match on the name portion).
pub(crate) fn find_spec(name: &str) -> Result<PathBuf, String> {
    let dir = specs_dir();
    if !dir.exists() {
        return Err("No .specs/ directory found".into());
    }

    let entries =
        fs::read_dir(&dir).map_err(|e| format!("Failed to read .specs/ directory: {e}"))?;

    let mut matches = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
        let filename = entry.file_name().to_string_lossy().to_string();
        if let Some(spec_name) = extract_spec_name(&filename)
            && spec_name == name
        {
            matches.push(entry.path());
        }
    }

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
    let dir = specs_dir();
    let current = current.to_string_lossy();

    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let filename = e.file_name().to_string_lossy().to_string();
            extract_spec_name(&filename).map(|name| name.to_string())
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
