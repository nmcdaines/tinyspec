use std::fs;
use std::path::Path;

use super::config::load_config;
use super::summary::{detect_dependency_cycles, load_all_summaries, parse_tasks_from_content};
use super::{collect_spec_files, find_spec, parse_front_matter};

#[derive(Debug)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug)]
pub struct LintIssue {
    pub severity: Severity,
    pub message: String,
    pub line: Option<usize>,
}

impl LintIssue {
    fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            line: None,
        }
    }

    fn error_at(message: impl Into<String>, line: usize) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            line: Some(line),
        }
    }

    fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            line: None,
        }
    }
}

const REQUIRED_SECTIONS: &[&str] = &["# Background", "# Proposal", "# Implementation Plan"];

pub fn lint_file(path: &Path) -> Vec<LintIssue> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return vec![LintIssue::error(format!("Failed to read file: {e}"))];
        }
    };

    let mut issues = Vec::new();

    // Check required sections
    for section in REQUIRED_SECTIONS {
        if !content.contains(section) {
            issues.push(LintIssue::error(format!(
                "Missing required section '{section}'"
            )));
        }
    }

    // Check for empty sections
    let mut current_heading_line: Option<(usize, &str)> = None;
    let mut section_has_content = false;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            // Finish previous section check
            if let Some((heading_line, heading)) = current_heading_line
                && !section_has_content {
                    issues.push(LintIssue::error_at(
                        format!("Section '{heading}' is empty"),
                        heading_line + 1,
                    ));
                }
            current_heading_line = Some((i, trimmed));
            section_has_content = false;
        } else if current_heading_line.is_some() && !trimmed.is_empty() {
            section_has_content = true;
        }
    }
    // Check last section
    if let Some((heading_line, heading)) = current_heading_line
        && !section_has_content {
            issues.push(LintIssue::error_at(
                format!("Section '{heading}' is empty"),
                heading_line + 1,
            ));
        }

    // Check task IDs are sequential
    let tasks = parse_tasks_from_content(&content);
    if tasks.is_empty() {
        issues.push(LintIssue::warning(
            "Spec has no tasks in Implementation Plan",
        ));
    } else {
        // Validate top-level tasks are sequential letters (A, B, C, ...)
        for (idx, task) in tasks.iter().enumerate() {
            let expected = char::from(b'A' + idx as u8).to_string();
            if task.id != expected {
                issues.push(LintIssue::error(format!(
                    "Non-sequential task ID: expected '{expected}', found '{}'",
                    task.id
                )));
            }

            // Validate subtask IDs
            for (sub_idx, child) in task.children.iter().enumerate() {
                let expected_sub = format!("{}.{}", task.id, sub_idx + 1);
                if child.id != expected_sub {
                    issues.push(LintIssue::error(format!(
                        "Non-sequential subtask ID: expected '{expected_sub}', found '{}'",
                        child.id
                    )));
                }
            }
        }
    }

    // Check applications are configured
    let apps: Vec<String> = parse_front_matter(&content)
        .map(|fm| {
            fm.applications
                .into_iter()
                .filter(|a| !a.is_empty())
                .collect()
        })
        .unwrap_or_default();

    if !apps.is_empty()
        && let Ok(config) = load_config() {
            for app in &apps {
                if !config.repositories.contains_key(app.as_str()) {
                    issues.push(LintIssue::warning(format!(
                        "Application '{app}' is not configured (run: tinyspec config set {app} <path>)"
                    )));
                }
            }
        }

    issues
}

pub fn lint(spec_name: Option<&str>, all: bool) -> Result<(), String> {
    let files = if all || spec_name.is_none() {
        collect_spec_files()?
    } else {
        vec![find_spec(spec_name.unwrap())?]
    };

    if files.is_empty() {
        println!("No specs found.");
        return Ok(());
    }

    let mut has_errors = false;
    let mut any_issues = false;

    // Collect all known spec names for dependency validation
    let all_files = collect_spec_files().unwrap_or_default();
    let all_spec_names: std::collections::HashSet<String> = all_files
        .iter()
        .filter_map(|p| {
            p.file_name()
                .and_then(|f| f.to_str())
                .and_then(|f| super::extract_spec_name(f))
                .map(String::from)
        })
        .collect();

    for path in &files {
        let mut issues = lint_file(path);

        // Check depends_on references
        let content = std::fs::read_to_string(path).unwrap_or_default();
        if let Some(fm) = parse_front_matter(&content) {
            for dep in &fm.depends_on {
                if !all_spec_names.contains(dep) {
                    issues.push(LintIssue::warning(format!(
                        "depends_on references unknown spec '{dep}'"
                    )));
                }
            }
        }

        if issues.is_empty() {
            continue;
        }

        any_issues = true;
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let spec_name_str = super::extract_spec_name(&filename).unwrap_or(&filename);

        println!("{spec_name_str}:");
        for issue in &issues {
            let level = match issue.severity {
                Severity::Error => {
                    has_errors = true;
                    "error"
                }
                Severity::Warning => "warning",
            };
            if let Some(line) = issue.line {
                println!("  [{level}] line {line}: {}", issue.message);
            } else {
                println!("  [{level}] {}", issue.message);
            }
        }
    }

    // Check for circular dependencies across all specs
    if let Ok(summaries) = load_all_summaries()
        && let Err(cycle) = detect_dependency_cycles(&summaries) {
            any_issues = true;
            has_errors = true;
            println!("(dependency cycle):");
            println!(
                "  [error] Circular dependency detected among specs: {}",
                cycle.join(", ")
            );
        }

    if !any_issues {
        println!("All specs are clean.");
    }

    if has_errors {
        Err("Lint errors found".into())
    } else {
        Ok(())
    }
}
