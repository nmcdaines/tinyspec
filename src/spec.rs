use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Local;
use clap_complete::engine::CompletionCandidate;
use pulldown_cmark::{Options, Parser};
use pulldown_cmark_to_cmark::cmark_with_options;
use serde::{Deserialize, Serialize};

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
    #[serde(default)]
    applications: Vec<String>,
}

fn parse_front_matter(content: &str) -> Option<FrontMatter> {
    let content = content.strip_prefix("---\n")?;
    let end = content.find("\n---")?;
    let yaml = &content[..end];
    serde_yaml::from_str(yaml).ok()
}

// ---------------------------------------------------------------------------
// Config file (~/.tinyspec/config.yaml)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub repositories: std::collections::BTreeMap<String, String>,
}

fn config_path() -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("TINYSPEC_HOME") {
        return Ok(PathBuf::from(dir).join("config.yaml"));
    }
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    Ok(PathBuf::from(home).join(".tinyspec").join("config.yaml"))
}

fn load_config() -> Result<Config, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {e}"))?;
    if content.trim().is_empty() {
        return Ok(Config::default());
    }
    serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse config: {e}"))
}

fn save_config(config: &Config) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {e}"))?;
    }
    let yaml =
        serde_yaml::to_string(config).map_err(|e| format!("Failed to serialize config: {e}"))?;
    fs::write(&path, yaml).map_err(|e| format!("Failed to write config: {e}"))?;
    Ok(())
}

pub fn config_set(name: &str, path: &str) -> Result<(), String> {
    let mut config = load_config()?;
    config
        .repositories
        .insert(name.to_string(), path.to_string());
    save_config(&config)?;
    println!("Set {name} = {path}");
    Ok(())
}

pub fn config_list() -> Result<(), String> {
    let config = load_config()?;
    if config.repositories.is_empty() {
        println!("No repositories configured.");
        println!(
            "Use `tinyspec config set <repo-name> <path>` to add a repository mapping."
        );
        return Ok(());
    }
    for (name, path) in &config.repositories {
        println!("{name}: {path}");
    }
    Ok(())
}

pub fn config_remove(name: &str) -> Result<(), String> {
    let mut config = load_config()?;
    if config.repositories.remove(name).is_none() {
        return Err(format!("Repository '{name}' not found in config"));
    }
    save_config(&config)?;
    println!("Removed {name}");
    Ok(())
}

// ---------------------------------------------------------------------------
// Markdown formatting
// ---------------------------------------------------------------------------

/// Split YAML front matter from the Markdown body.
/// Returns (front_matter_block_including_delimiters, body).
fn split_front_matter(content: &str) -> (Option<&str>, &str) {
    if let Some(rest) = content.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n") {
            let split = "---\n".len() + end + "\n---\n".len();
            return (Some(&content[..split]), &content[split..]);
        }
    }
    (None, content)
}

/// Format a Markdown string by parsing it through pulldown-cmark and rendering
/// it back to normalised Markdown. YAML front matter is preserved verbatim.
pub fn format_markdown(content: &str) -> Result<String, String> {
    let (front_matter, body) = split_front_matter(content);

    let opts = Options::ENABLE_TASKLISTS
        | Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;

    let parser = Parser::new_ext(body, opts);

    let mut cmark_opts = pulldown_cmark_to_cmark::Options::default();
    cmark_opts.newlines_after_headline = 2;
    cmark_opts.newlines_after_paragraph = 2;
    cmark_opts.newlines_after_codeblock = 2;
    cmark_opts.newlines_after_table = 2;
    cmark_opts.newlines_after_rule = 2;
    cmark_opts.newlines_after_list = 2;
    cmark_opts.newlines_after_blockquote = 2;
    cmark_opts.newlines_after_rest = 1;
    cmark_opts.code_block_token_count = 3;
    cmark_opts.list_token = '-';

    let mut formatted_body = String::with_capacity(body.len());
    cmark_with_options(parser, &mut formatted_body, cmark_opts)
        .map_err(|e| format!("Failed to format markdown: {e}"))?;

    let mut result = String::with_capacity(content.len());
    if let Some(fm) = front_matter {
        result.push_str(fm);
        // Ensure blank line between front matter and body
        if !formatted_body.starts_with('\n') {
            result.push('\n');
        }
    }
    result.push_str(&formatted_body);

    // Ensure trailing newline
    if !result.ends_with('\n') {
        result.push('\n');
    }

    Ok(result)
}

/// Format a spec file at the given path in place (no output).
fn format_file(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read spec: {e}"))?;
    let formatted = format_markdown(&content)?;
    fs::write(path, &formatted).map_err(|e| format!("Failed to write spec: {e}"))?;
    Ok(())
}

/// Format a single spec file in place.
pub fn format_spec(name: &str) -> Result<(), String> {
    let path = find_spec(name)?;
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read spec: {e}"))?;
    let formatted = format_markdown(&content)?;
    fs::write(&path, &formatted).map_err(|e| format!("Failed to write spec: {e}"))?;
    println!("Formatted {}", path.file_name().unwrap().to_string_lossy());
    Ok(())
}

/// Format all spec files in the `.specs/` directory.
pub fn format_all_specs() -> Result<(), String> {
    let dir = specs_dir();
    if !dir.exists() {
        println!("No specs found.");
        return Ok(());
    }

    let entries: Vec<_> = fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read .specs/ directory: {e}"))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
        .collect();

    if entries.is_empty() {
        println!("No specs found.");
        return Ok(());
    }

    for entry in &entries {
        let content = fs::read_to_string(entry.path())
            .map_err(|e| format!("Failed to read spec: {e}"))?;
        let formatted = format_markdown(&content)?;
        fs::write(entry.path(), &formatted)
            .map_err(|e| format!("Failed to write spec: {e}"))?;
        println!("Formatted {}", entry.file_name().to_string_lossy());
    }

    Ok(())
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
    format_file(&path)?;
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

    // Parse frontmatter to check for application references
    let apps: Vec<String> = parse_front_matter(&content)
        .map(|fm| {
            fm.applications
                .into_iter()
                .filter(|a| !a.is_empty())
                .collect()
        })
        .unwrap_or_default();

    if apps.is_empty() {
        print!("{content}");
        return Ok(());
    }

    // Resolve application names to folder paths via config
    let config_path = config_path()?;
    if !config_path.exists() {
        return Err(format!(
            "Spec references applications {:?} but no config file found.\n\
             Create one with: tinyspec config set <repo-name> <path>",
            apps
        ));
    }

    let config = load_config()?;
    let mut missing: Vec<&str> = Vec::new();
    let mut replacements: Vec<(&str, &str)> = Vec::new();

    for app in &apps {
        match config.repositories.get(app.as_str()) {
            Some(folder) => replacements.push((app.as_str(), folder.as_str())),
            None => missing.push(app.as_str()),
        }
    }

    if !missing.is_empty() {
        return Err(format!(
            "Spec references applications not found in config: {}\n\
             Add them with: tinyspec config set <repo-name> <path>",
            missing.join(", ")
        ));
    }

    // Perform find-and-replace of application names with folder paths
    let mut output = content;
    for (app_name, folder_path) in replacements {
        output = output.replace(app_name, folder_path);
    }

    print!("{output}");
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
    format_file(&path)?;

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
        ("tinyspec:refine.md", TINYSPEC_REFINE_SKILL),
        ("tinyspec:work.md", TINYSPEC_WORK_SKILL),
        ("tinyspec:task.md", TINYSPEC_TASK_SKILL),
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

const TINYSPEC_REFINE_SKILL: &str = r#"IMPORTANT: `tinyspec` is a native binary CLI tool (installed via cargo/crates.io), NOT an npm package. Run it directly as `tinyspec <command>`. Never use npm, npx, or node to run it.

Read the tinyspec specification at `.specs/$ARGUMENTS.md` (resolve the name by matching the suffix after the timestamp prefix, e.g., `hello-world` matches `2025-02-17-09-36-hello-world.md`).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to collaborate with the user to refine this spec:

1. Read and understand the spec's Background and Proposal sections.
2. Ask clarifying questions about ambiguous requirements or missing context. Use the `AskUserQuestion` tool to present structured, selectable options rather than asking inline.
3. Suggest improvements to the Background and Proposal sections.
4. Once the user is satisfied with the problem definition, scaffold or update the **Implementation Plan**:
   - Break the work into logical task groups (A, B, C, ...)
   - Each group gets subtasks (A.1, A.2, ...)
   - Use markdown checkboxes: `- [ ] A: Task description`
5. If the user wants tests, scaffold the **Test Plan** using Given/When/Then syntax with task IDs (T.1, T.2, ...).
6. Wait for user approval before writing any changes to the spec file.

Use `tinyspec view <spec-name>` to read the current spec and directly edit the file when making approved changes. Keep the front matter and existing structure intact.

After editing a spec file directly, run `tinyspec format <spec-name>` to normalize the Markdown formatting.
"#;

const TINYSPEC_WORK_SKILL: &str = r#"IMPORTANT: `tinyspec` is a native binary CLI tool (installed via cargo/crates.io), NOT an npm package. Run it directly as `tinyspec <command>`. Never use npm, npx, or node to run it.

Read the tinyspec specification at `.specs/$ARGUMENTS.md` (resolve the name by matching the suffix after the timestamp prefix).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to work through the spec's Implementation Plan:

1. Read the full spec using `tinyspec view <spec-name>` to understand the context (Background, Proposal). This command resolves application references to folder paths automatically.
   - If `tinyspec view` fails with a config error, inform the user that they need to configure repository paths with `tinyspec config set <repo-name> <path>` and stop.
   - If the spec references multiple applications, use the `AskUserQuestion` tool to ask the user which repositories to focus on before proceeding.
2. Run `tinyspec status <spec-name>` to see current progress.
3. Find the next unchecked task in the Implementation Plan (top-level tasks in order: A, B, C, ...).
4. For each top-level task group:
   a. Implement all subtasks within the group.
   b. After completing each subtask, mark it done with `tinyspec check <spec-name> <task-id>`.
   c. After completing the top-level task and all its subtasks, mark it done too.
   d. Commit your progress with a descriptive commit message referencing the spec and task group.
5. Move on to the next task group and repeat.

If you encounter ambiguity or a task that requires user input, use the `AskUserQuestion` tool to present structured, selectable options rather than asking inline. Always verify your work compiles/runs before marking tasks complete.
"#;

const TINYSPEC_TASK_SKILL: &str = r#"IMPORTANT: `tinyspec` is a native binary CLI tool (installed via cargo/crates.io), NOT an npm package. Run it directly as `tinyspec <command>`. Never use npm, npx, or node to run it.

The arguments contain a spec name and a task ID separated by a space: `$ARGUMENTS`
Parse the first word as the spec name and the second word as the task ID.

Read the tinyspec specification at `.specs/<spec-name>.md` (resolve by matching the suffix after the timestamp prefix).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to complete a specific task:

1. Read the full spec using `tinyspec view <spec-name>` to understand the context (Background, Proposal, Implementation Plan). This command resolves application references to folder paths automatically.
   - If `tinyspec view` fails with a config error, inform the user that they need to configure repository paths with `tinyspec config set <repo-name> <path>` and stop.
   - If the spec references multiple applications, use the `AskUserQuestion` tool to ask the user which repositories to focus on before proceeding.
2. Locate the specified task in the Implementation Plan.
3. Implement just that task.
4. Mark it complete with `tinyspec check <spec-name> <task-id>`.
5. If the task has subtasks, complete and check each subtask as well.

If the task depends on uncompleted prior tasks, use the `AskUserQuestion` tool to warn the user and ask how to proceed. Always verify your work compiles/runs before marking the task complete.
"#;
