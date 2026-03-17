use std::fs;
use std::path::PathBuf;

use super::{collect_spec_files, find_spec, specs_dir};

const ARCHIVE_DIR: &str = "archive";

/// Returns the `.specs/archive/` path.
pub(crate) fn archive_dir() -> PathBuf {
    specs_dir().join(ARCHIVE_DIR)
}

pub fn archive_spec(name: &str) -> Result<(), String> {
    let path = find_spec(name)?;

    let specs_root = specs_dir();
    let archive_root = archive_dir();

    // Preserve group subdirectory structure inside archive/
    let parent = path.parent().unwrap_or(&specs_root);
    let dest_dir = if parent == specs_root {
        archive_root.clone()
    } else {
        let group = parent.file_name().unwrap_or_default();
        archive_root.join(group)
    };

    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create archive directory: {e}"))?;

    let filename = path.file_name().unwrap_or_default();
    let dest = dest_dir.join(filename);

    fs::rename(&path, &dest).map_err(|e| format!("Failed to archive spec: {e}"))?;

    println!("Archived: {}", dest.display());
    Ok(())
}

pub fn unarchive_spec(name: &str) -> Result<(), String> {
    // Search within the archive directory
    let archive_root = archive_dir();
    if !archive_root.exists() {
        return Err(format!("No archived spec found matching '{name}'"));
    }

    let archived_path = find_archived_spec(name)?;

    // Determine destination: mirror the archive sub-path back into .specs/
    let specs_root = specs_dir();
    let archived_parent = archived_path
        .parent()
        .unwrap_or(&archive_root);

    let dest_dir = if archived_parent == archive_root {
        specs_root.clone()
    } else {
        // Preserve group inside archive (e.g. archive/improvements/ → improvements/)
        let group = archived_parent.file_name().unwrap_or_default();
        specs_root.join(group)
    };

    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create destination directory: {e}"))?;

    let filename = archived_path.file_name().unwrap_or_default();
    let dest = dest_dir.join(filename);

    fs::rename(&archived_path, &dest).map_err(|e| format!("Failed to unarchive spec: {e}"))?;

    println!("Unarchived: {}", dest.display());
    Ok(())
}

pub fn archive_all_completed() -> Result<(), String> {
    use super::summary::{SpecStatus, load_spec_summary};

    let files = collect_spec_files()?;
    let mut count = 0;

    for path in &files {
        if let Some(summary) = load_spec_summary(path) {
            if summary.status == SpecStatus::Completed {
                let name = summary.name.clone();
                archive_spec(&name)?;
                count += 1;
            }
        }
    }

    if count == 0 {
        println!("No completed specs to archive.");
    } else {
        println!("Archived {count} completed spec(s).");
    }

    Ok(())
}

/// Find a spec file within the archive directory by name.
pub(crate) fn find_archived_spec(name: &str) -> Result<PathBuf, String> {
    let archive_root = archive_dir();
    if !archive_root.exists() {
        return Err(format!("No archived spec found matching '{name}'"));
    }

    let mut matches = Vec::new();

    // Walk archive root and one level of subdirectories
    if let Ok(entries) = fs::read_dir(&archive_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if is_spec_match(&sub_path, name) {
                            matches.push(sub_path);
                        }
                    }
                }
            } else if is_spec_match(&path, name) {
                matches.push(path);
            }
        }
    }

    match matches.len() {
        0 => Err(format!("No archived spec found matching '{name}'")),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => {
            matches.sort();
            Ok(matches.into_iter().last().unwrap())
        }
    }
}

fn is_spec_match(path: &std::path::Path, name: &str) -> bool {
    path.extension().is_some_and(|ext| ext == "md")
        && path
            .file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| super::extract_spec_name(f))
            == Some(name)
}

/// Collect spec files including the archive directory.
pub(crate) fn collect_spec_files_with_archived() -> Result<Vec<PathBuf>, String> {
    let mut files = collect_spec_files()?;

    let archive_root = archive_dir();
    if archive_root.exists() {
        if let Ok(entries) = fs::read_dir(&archive_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
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
        }
    }

    Ok(files)
}
