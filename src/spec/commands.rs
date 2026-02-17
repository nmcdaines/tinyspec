use std::fs;
use std::io::{self, BufRead, Write};
use std::process::Command;

use chrono::Local;

use super::config::{config_path, load_config};
use super::format::format_file;
use super::{extract_spec_name, find_spec, parse_front_matter, specs_dir, validate_kebab_case};

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
