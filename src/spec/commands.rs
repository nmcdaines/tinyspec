use std::fs;
use std::io::{self, BufRead, Write};
use std::process::Command;

use chrono::Local;

use super::config::{config_path, load_config};
use super::format::format_file;
use super::templates::{collect_templates, find_template};
use super::{
    collect_spec_files, extract_spec_name, find_spec, parse_front_matter, parse_spec_input,
    specs_dir,
};

pub fn new_spec(input: &str, template_name: Option<&str>) -> Result<(), String> {
    let (group, name) = parse_spec_input(input)?;

    // Enforce global uniqueness — check if name already exists anywhere
    let existing = collect_spec_files().unwrap_or_default();
    for path in &existing {
        if let Some(filename) = path.file_name().and_then(|f| f.to_str())
            && extract_spec_name(filename) == Some(name)
        {
            return Err(format!(
                "A spec named '{name}' already exists: {}",
                path.display()
            ));
        }
    }

    let dir = match group {
        Some(g) => specs_dir().join(g),
        None => specs_dir(),
    };
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create {} directory: {e}", dir.display()))?;

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

    let date = Local::now().format("%Y-%m-%d").to_string();

    // Resolve template: explicit --template flag, or auto-detect "default"
    let template = match template_name {
        Some(name) => Some(find_template(name)?),
        None => {
            // Auto-apply "default" template if it exists
            collect_templates()
                .unwrap_or_default()
                .into_iter()
                .find(|t| t.name == "default")
        }
    };

    let content = match template {
        Some(t) => {
            let raw = fs::read_to_string(&t.path)
                .map_err(|e| format!("Failed to read template '{}': {e}", t.name))?;
            raw.replace("{{title}}", &title).replace("{{date}}", &date)
        }
        None => {
            format!(
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
            )
        }
    };

    fs::write(&path, &content).map_err(|e| format!("Failed to write spec file: {e}"))?;
    format_file(&path)?;
    println!("Created spec: {filename}");
    Ok(())
}

pub fn list() -> Result<(), String> {
    let mut files = collect_spec_files()?;

    if files.is_empty() {
        println!("No specs found.");
        return Ok(());
    }

    // Sort by filename (natural date ordering due to timestamp prefix)
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    // Group by parent directory
    let specs_root = specs_dir();
    let mut ungrouped = Vec::new();
    let mut groups: std::collections::BTreeMap<String, Vec<&std::path::PathBuf>> =
        std::collections::BTreeMap::new();

    for path in &files {
        let parent = path.parent().unwrap_or(&specs_root);
        if parent == specs_root {
            ungrouped.push(path);
        } else {
            let group_name = parent
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            groups.entry(group_name).or_default().push(path);
        }
    }

    let print_spec = |path: &std::path::Path| {
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let spec_name = extract_spec_name(&filename).unwrap_or(&filename);
        let content = fs::read_to_string(path).unwrap_or_default();
        let title = parse_front_matter(&content)
            .and_then(|fm| fm.title)
            .unwrap_or_else(|| "(no title)".into());
        println!("{spec_name:30} {title}");
    };

    // Print ungrouped specs first
    for path in &ungrouped {
        print_spec(path);
    }

    // Print each group with a header
    for (group_name, paths) in &groups {
        if !ungrouped.is_empty() || groups.len() > 1 {
            println!();
        }
        println!("{group_name}/");
        for path in paths {
            print_spec(path);
        }
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
            if let Some(after) = trimmed.strip_prefix("- [ ] ")
                && after.starts_with(&target)
            {
                *line = line.replacen("- [ ] ", "- [x] ", 1);
                found = true;
                break;
            }
        } else if let Some(after) = trimmed.strip_prefix("- [x] ")
            && after.starts_with(&target)
        {
            *line = line.replacen("- [x] ", "- [ ] ", 1);
            found = true;
            break;
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
    use super::summary::load_spec_summary;

    match name {
        Some(name) => {
            let path = find_spec(name)?;
            let summary =
                load_spec_summary(&path).ok_or_else(|| format!("Failed to load spec '{name}'"))?;
            println!(
                "{}: {}/{} tasks complete",
                summary.name, summary.checked, summary.total
            );
        }
        None => {
            let mut files = collect_spec_files()?;

            if files.is_empty() {
                println!("No specs found.");
                return Ok(());
            }

            files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

            for path in &files {
                if let Some(summary) = load_spec_summary(path) {
                    println!(
                        "{}: {}/{} tasks complete",
                        summary.name, summary.checked, summary.total
                    );
                }
            }
        }
    }
    Ok(())
}
