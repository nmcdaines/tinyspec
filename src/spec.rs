use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Local;
use clap_complete::engine::CompletionCandidate;
use serde::Deserialize;

const SPECS_DIR: &str = ".specs";
const TIMESTAMP_PREFIX_LEN: usize = 17; // "YYYY-MM-DD-HH-MM-"

fn specs_dir() -> PathBuf {
    PathBuf::from(SPECS_DIR)
}

/// Extract spec name from a filename like `2025-02-17-09-36-hello-world.md`
fn extract_spec_name(filename: &str) -> Option<&str> {
    if filename.len() > TIMESTAMP_PREFIX_LEN + 3 && filename.ends_with(".md") {
        Some(&filename[TIMESTAMP_PREFIX_LEN..filename.len() - 3])
    } else {
        None
    }
}

/// Find the spec file matching the given name (exact match on the name portion).
fn find_spec(name: &str) -> Result<PathBuf, String> {
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
        if let Some(spec_name) = extract_spec_name(&filename) {
            if spec_name == name {
                matches.push(entry.path());
            }
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
        .map(|name| CompletionCandidate::new(name))
        .collect()
}

// ---------------------------------------------------------------------------
// Front matter
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct FrontMatter {
    title: Option<String>,
}

fn parse_front_matter(content: &str) -> Option<FrontMatter> {
    let content = content.strip_prefix("---\n")?;
    let end = content.find("\n---")?;
    let yaml = &content[..end];
    serde_yaml::from_str(yaml).ok()
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn validate_kebab_case(name: &str) -> Result<(), String> {
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

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

pub fn new_spec(name: &str) -> Result<(), String> {
    validate_kebab_case(name)?;

    let dir = specs_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create .specs/ directory: {e}"))?;

    let timestamp = Local::now().format("%Y-%m-%d-%H-%M");
    let filename = format!("{timestamp}-{name}.md");
    let path = dir.join(&filename);

    if path.exists() {
        return Err(format!("Spec file already exists: {filename}"));
    }

    // Title-case the kebab-case name
    let title: String = name
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let content = format!(
        "\
---
tinySpec: v0
title: {title}
applications:
    -
---

# Background



# Proposal



# Implementation Plan



# Test Plan

"
    );

    fs::write(&path, &content).map_err(|e| format!("Failed to write spec file: {e}"))?;
    println!("Created spec: {filename}");
    Ok(())
}

pub fn list() -> Result<(), String> {
    let dir = specs_dir();
    if !dir.exists() {
        println!("No specs found.");
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read .specs/ directory: {e}"))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
        .collect();

    if entries.is_empty() {
        println!("No specs found.");
        return Ok(());
    }

    // Sort by filename (natural date ordering due to timestamp prefix)
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let filename = entry.file_name().to_string_lossy().to_string();
        let spec_name = extract_spec_name(&filename).unwrap_or(&filename);

        let content = fs::read_to_string(entry.path()).unwrap_or_default();
        let title = parse_front_matter(&content)
            .and_then(|fm| fm.title)
            .unwrap_or_else(|| "(no title)".into());

        println!("{spec_name:30} {title}");
    }

    Ok(())
}

pub fn view(name: &str) -> Result<(), String> {
    let path = find_spec(name)?;
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read spec: {e}"))?;
    print!("{content}");
    Ok(())
}

pub fn edit(name: &str) -> Result<(), String> {
    let path = find_spec(name)?;
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".into());

    Command::new(&editor)
        .arg(&path)
        .status()
        .map_err(|e| format!("Failed to open editor '{editor}': {e}"))?;

    Ok(())
}

pub fn delete(name: &str) -> Result<(), String> {
    let path = find_spec(name)?;
    let filename = path.file_name().unwrap().to_string_lossy();

    eprint!("Delete {filename}? [y/N] ");
    io::stderr().flush().ok();

    let mut input = String::new();
    io::stdin()
        .lock()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {e}"))?;

    if input.trim().eq_ignore_ascii_case("y") {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete spec: {e}"))?;
        println!("Deleted {filename}");
    } else {
        println!("Cancelled.");
    }

    Ok(())
}

pub fn check_task(name: &str, task_id: &str, check: bool) -> Result<(), String> {
    let path = find_spec(name)?;
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read spec: {e}"))?;

    let target = format!("{task_id}:");
    let mut found = false;
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    for line in &mut lines {
        let trimmed = line.trim();
        if check {
            if let Some(after) = trimmed.strip_prefix("- [ ] ") {
                if after.starts_with(&target) {
                    *line = line.replacen("- [ ] ", "- [x] ", 1);
                    found = true;
                    break;
                }
            }
        } else if let Some(after) = trimmed.strip_prefix("- [x] ") {
            if after.starts_with(&target) {
                *line = line.replacen("- [x] ", "- [ ] ", 1);
                found = true;
                break;
            }
        }
    }

    if !found {
        let state = if check { "unchecked" } else { "checked" };
        return Err(format!(
            "No {state} task '{task_id}' found in spec '{name}'"
        ));
    }

    // Preserve trailing newline
    let mut output = lines.join("\n");
    if content.ends_with('\n') {
        output.push('\n');
    }

    fs::write(&path, &output).map_err(|e| format!("Failed to write spec: {e}"))?;

    let action = if check { "Checked" } else { "Unchecked" };
    println!("{action} task {task_id}");
    Ok(())
}

pub fn status(name: Option<&str>) -> Result<(), String> {
    match name {
        Some(name) => {
            let path = find_spec(name)?;
            let content =
                fs::read_to_string(&path).map_err(|e| format!("Failed to read spec: {e}"))?;
            print_status(name, &content);
        }
        None => {
            let dir = specs_dir();
            if !dir.exists() {
                println!("No specs found.");
                return Ok(());
            }

            let mut entries: Vec<_> = fs::read_dir(&dir)
                .map_err(|e| format!("Failed to read .specs/ directory: {e}"))?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
                .collect();

            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let filename = entry.file_name().to_string_lossy().to_string();
                let spec_name = extract_spec_name(&filename).unwrap_or(&filename);
                let content = fs::read_to_string(entry.path()).unwrap_or_default();
                print_status(spec_name, &content);
            }
        }
    }
    Ok(())
}

fn print_status(name: &str, content: &str) {
    let mut total = 0u32;
    let mut checked = 0u32;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [ ] ") {
            total += 1;
        } else if trimmed.starts_with("- [x] ") {
            total += 1;
            checked += 1;
        }
    }

    println!("{name}: {checked}/{total} tasks complete");
}

// ---------------------------------------------------------------------------
// Init — Claude Code skills + shell completion instructions
// ---------------------------------------------------------------------------

pub fn init() -> Result<(), String> {
    let commands_dir = Path::new(".claude/commands");
    fs::create_dir_all(commands_dir)
        .map_err(|e| format!("Failed to create .claude/commands/ directory: {e}"))?;

    let skills: &[(&str, &str)] = &[
        ("spec-refine.md", SPEC_REFINE_SKILL),
        ("spec-work.md", SPEC_WORK_SKILL),
        ("spec-task.md", SPEC_TASK_SKILL),
    ];

    for (filename, content) in skills {
        let path = commands_dir.join(filename);
        if path.exists() {
            println!("Skipped {filename} (already exists)");
        } else {
            fs::write(&path, content).map_err(|e| format!("Failed to write {filename}: {e}"))?;
            println!("Created {filename}");
        }
    }

    // Shell completion instructions
    println!();
    println!("Shell completion setup:");

    let shell = std::env::var("SHELL").unwrap_or_default();
    if shell.contains("zsh") {
        println!("  Add this to your ~/.zshrc:");
        println!("  source <(COMPLETE=zsh tinyspec)");
    } else if shell.contains("fish") {
        println!("  Add this to your fish config:");
        println!("  COMPLETE=fish tinyspec | source");
    } else {
        println!("  Add this to your ~/.bashrc:");
        println!("  source <(COMPLETE=bash tinyspec)");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Claude Code skill prompts
// ---------------------------------------------------------------------------

const SPEC_REFINE_SKILL: &str = r#"Read the tinyspec specification at `.specs/$ARGUMENTS.md` (resolve the name by matching the suffix after the timestamp prefix, e.g., `hello-world` matches `2025-02-17-09-36-hello-world.md`).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to collaborate with the user to refine this spec:

1. Read and understand the spec's Background and Proposal sections.
2. Ask clarifying questions about ambiguous requirements or missing context.
3. Suggest improvements to the Background and Proposal sections.
4. Once the user is satisfied with the problem definition, scaffold or update the **Implementation Plan**:
   - Break the work into logical task groups (A, B, C, ...)
   - Each group gets subtasks (A.1, A.2, ...)
   - Use markdown checkboxes: `- [ ] A: Task description`
5. If the user wants tests, scaffold the **Test Plan** using Given/When/Then syntax with task IDs (T.1, T.2, ...).
6. Wait for user approval before writing any changes to the spec file.

Use `tinyspec view <spec-name>` to read the current spec and directly edit the file when making approved changes. Keep the front matter and existing structure intact.
"#;

const SPEC_WORK_SKILL: &str = r#"Read the tinyspec specification at `.specs/$ARGUMENTS.md` (resolve the name by matching the suffix after the timestamp prefix).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to work through the spec's Implementation Plan:

1. Read the full spec to understand the context (Background, Proposal).
2. Run `tinyspec status <spec-name>` to see current progress.
3. Find the next unchecked task in the Implementation Plan (top-level tasks in order: A, B, C, ...).
4. For each top-level task group:
   a. Implement all subtasks within the group.
   b. After completing each subtask, mark it done with `tinyspec check <spec-name> <task-id>`.
   c. After completing the top-level task and all its subtasks, mark it done too.
   d. Commit your progress with a descriptive commit message referencing the spec and task group.
5. Move on to the next task group and repeat.

If you encounter ambiguity or a task that requires user input, stop and ask before proceeding. Always verify your work compiles/runs before marking tasks complete.
"#;

const SPEC_TASK_SKILL: &str = r#"The arguments contain a spec name and a task ID separated by a space: `$ARGUMENTS`
Parse the first word as the spec name and the second word as the task ID.

Read the tinyspec specification at `.specs/<spec-name>.md` (resolve by matching the suffix after the timestamp prefix).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to complete a specific task:

1. Read the full spec to understand the context (Background, Proposal, Implementation Plan).
2. Locate the specified task in the Implementation Plan.
3. Implement just that task.
4. Mark it complete with `tinyspec check <spec-name> <task-id>`.
5. If the task has subtasks, complete and check each subtask as well.

If the task depends on uncompleted prior tasks, warn the user and ask how to proceed. Always verify your work compiles/runs before marking the task complete.
"#;
