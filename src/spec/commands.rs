use std::fs;
use std::io::{self, BufRead, Write};
use std::process::Command;

use serde::Serialize;

use chrono::Local;

use super::config::{config_path, load_config};
use super::format::format_file;
use super::hooks::{Event, HookContext, run_hooks};
use super::summary::{SpecStatus, load_spec_summary};
use super::templates::{collect_templates, find_template, substitute_variables};
use super::{
    SPECS_DIR, TIMESTAMP_PREFIX_LEN, collect_spec_files, discover_git_root, extract_spec_name,
    find_spec, parse_front_matter, parse_spec_input, specs_dir,
};

pub fn new_spec(input: &str, template_name: Option<&str>) -> Result<(), String> {
    new_spec_impl(input, template_name, false)
}

pub fn new_spec_with_hooks(input: &str, template_name: Option<&str>) -> Result<(), String> {
    new_spec_impl(input, template_name, true)
}

fn new_spec_impl(input: &str, template_name: Option<&str>, fire_hooks: bool) -> Result<(), String> {
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

    // If .specs/ doesn't exist yet, create it at the git repo root (if in a git repo)
    let base = if specs_dir().exists() {
        specs_dir()
    } else {
        match discover_git_root() {
            Some(root) => root.join(SPECS_DIR),
            None => specs_dir(),
        }
    };
    let dir = match group {
        Some(g) => base.join(g),
        None => base,
    };
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create {} directory: {e}", dir.display()))?;

    // Find a unique timestamp prefix, incrementing by 1 minute on conflict
    let existing_prefixes: Vec<String> = existing
        .iter()
        .filter_map(|p| {
            p.file_name()
                .and_then(|f| f.to_str())
                .filter(|f| f.len() >= TIMESTAMP_PREFIX_LEN)
                .map(|f| f[..TIMESTAMP_PREFIX_LEN].to_string())
        })
        .collect();

    let mut ts = Local::now();
    loop {
        let prefix = format!("{}-", ts.format("%Y-%m-%d-%H-%M"));
        if !existing_prefixes.contains(&prefix) {
            break;
        }
        ts += chrono::Duration::minutes(1);
    }

    let timestamp = ts.format("%Y-%m-%d-%H-%M");
    let filename = format!("{timestamp}-{name}.md");
    let path = dir.join(&filename);

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

    let vars =
        std::collections::HashMap::from([("title", title.as_str()), ("date", date.as_str())]);

    let content = match template {
        Some(t) => {
            let raw = fs::read_to_string(&t.path)
                .map_err(|e| format!("Failed to read template '{}': {e}", t.name))?;
            substitute_variables(&raw, &vars)
        }
        None => {
            format!(
                "\
---
tinySpec: v0
title: {title}
# priority: high        # high | medium | low (default: medium)
# tags: []              # arbitrary string labels for filtering
# depends_on: []        # spec names that must complete first
applications:
    -
---

# Background



# Proposal

<!-- Add a Mermaid diagram here when the proposal involves interacting components, a state machine, data schema, or dependency graph. Example:
```mermaid
flowchart LR
    A[Input] --> B[Process] --> C[Output]
```
-->

# Implementation Plan

- [ ] A:

# Test Plan

- [ ] T.1:
- [ ] T.2:
"
            )
        }
    };

    fs::write(&path, &content).map_err(|e| format!("Failed to write spec file: {e}"))?;
    format_file(&path)?;
    println!("Created spec: {filename}");

    if fire_hooks {
        let fm = parse_front_matter(&content);
        let spec_group = match group {
            Some(g) => g.to_string(),
            None => String::new(),
        };
        run_hooks(&HookContext {
            event: Event::OnSpecCreate,
            spec_name: name.to_string(),
            spec_title: fm.and_then(|f| f.title).unwrap_or_else(|| title.clone()),
            spec_group,
            task_id: String::new(),
            spec_path: path.to_string_lossy().to_string(),
        });
    }

    Ok(())
}

pub fn list(json: bool, include_archived: bool, tag: Option<&str>) -> Result<(), String> {
    use super::archive::collect_spec_files_with_archived;
    use super::summary::load_spec_summary;

    let mut files = if include_archived {
        collect_spec_files_with_archived()?
    } else {
        collect_spec_files()?
    };

    if files.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("No specs found.");
        }
        return Ok(());
    }

    // Sort by filename (natural date ordering due to timestamp prefix)
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    if json {
        let mut summaries: Vec<_> = files.iter().filter_map(|p| load_spec_summary(p)).collect();
        if let Some(tag_filter) = tag {
            summaries.retain(|s| s.tags.iter().any(|t| t == tag_filter));
        }
        let out = serde_json::to_string_pretty(&summaries)
            .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
        println!("{out}");
        return Ok(());
    }

    // Group by parent directory
    let specs_root = specs_dir();
    let mut ungrouped = Vec::new();
    let mut groups: std::collections::BTreeMap<String, Vec<&std::path::PathBuf>> =
        std::collections::BTreeMap::new();

    for path in &files {
        // Apply tag filter
        if let Some(tag_filter) = tag {
            let content = fs::read_to_string(path).unwrap_or_default();
            let fm = parse_front_matter(&content);
            let has_tag = fm
                .map(|f| f.tags.iter().any(|t| t == tag_filter))
                .unwrap_or(false);
            if !has_tag {
                continue;
            }
        }

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
        let fm = parse_front_matter(&content);
        let title = fm
            .as_ref()
            .and_then(|f| f.title.clone())
            .unwrap_or_else(|| "(no title)".into());
        let priority = fm.as_ref().and_then(|f| f.priority).unwrap_or_default();
        println!("[{}] {spec_name:30} {title}", priority.label());
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

pub fn view(name: &str, json: bool) -> Result<(), String> {
    use super::summary::load_spec_summary;

    let path = find_spec(name)?;
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read spec: {e}"))?;

    if json {
        #[derive(Serialize)]
        struct ViewJson {
            name: String,
            title: Option<String>,
            applications: Vec<String>,
            body: String,
            tasks: Vec<super::summary::TaskNode>,
        }

        let fm = parse_front_matter(&content);
        let title = fm.as_ref().and_then(|f| f.title.clone());
        let applications = fm
            .map(|f| {
                f.applications
                    .into_iter()
                    .filter(|a| !a.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        let summary = load_spec_summary(&path);
        let tasks = summary.map(|s| s.tasks).unwrap_or_default();

        let view_json = ViewJson {
            name: name.to_string(),
            title,
            applications,
            body: content.clone(),
            tasks,
        };

        let out = serde_json::to_string_pretty(&view_json)
            .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
        println!("{out}");
        return Ok(());
    }

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
    check_task_impl(name, task_id, check, true)
}

pub fn check_task_no_hooks(name: &str, task_id: &str, check: bool) -> Result<(), String> {
    check_task_impl(name, task_id, check, false)
}

fn check_task_impl(name: &str, task_id: &str, check: bool, fire_hooks: bool) -> Result<(), String> {
    let path = find_spec(name)?;
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read spec: {e}"))?;

    // Capture status before change (for transition detection)
    let status_before = load_spec_summary(&path).map(|s| s.status);

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

    if fire_hooks {
        let status_after = load_spec_summary(&path).map(|s| s.status);
        let fm = parse_front_matter(&content);
        let spec_title = fm.and_then(|f| f.title).unwrap_or_else(|| name.to_string());
        let spec_group = path
            .parent()
            .and_then(|p| {
                let specs_root = specs_dir();
                if p != specs_root {
                    p.file_name().and_then(|g| g.to_str()).map(String::from)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let spec_path_str = path.to_string_lossy().to_string();

        let task_event = if check {
            Event::OnTaskCheck
        } else {
            Event::OnTaskUncheck
        };
        run_hooks(&HookContext {
            event: task_event,
            spec_name: name.to_string(),
            spec_title: spec_title.clone(),
            spec_group: spec_group.clone(),
            task_id: task_id.to_string(),
            spec_path: spec_path_str.clone(),
        });

        // Fire spec-level transition hooks
        if check
            && let (Some(before), Some(after)) = (status_before, status_after) {
                if before == SpecStatus::Pending && after == SpecStatus::InProgress {
                    run_hooks(&HookContext {
                        event: Event::OnSpecStart,
                        spec_name: name.to_string(),
                        spec_title: spec_title.clone(),
                        spec_group: spec_group.clone(),
                        task_id: task_id.to_string(),
                        spec_path: spec_path_str.clone(),
                    });
                } else if after == SpecStatus::Completed {
                    run_hooks(&HookContext {
                        event: Event::OnSpecComplete,
                        spec_name: name.to_string(),
                        spec_title,
                        spec_group,
                        task_id: task_id.to_string(),
                        spec_path: spec_path_str,
                    });
                }
            }
    }

    Ok(())
}

pub fn status(
    name: Option<&str>,
    json: bool,
    include_archived: bool,
    skip_tests: bool,
    tag: Option<&str>,
) -> Result<(), String> {
    use super::archive::collect_spec_files_with_archived;
    use super::summary::{load_all_summaries, load_spec_summary};

    let format_status = |summary: &super::summary::SpecSummary| -> String {
        let blocked = if summary.blocked { " BLOCKED" } else { "" };
        let priority = format!("[{}]", summary.priority.label());
        if skip_tests || summary.total_tests == 0 {
            format!(
                "{priority} {}: {}/{} tasks complete{blocked}",
                summary.name, summary.checked, summary.total
            )
        } else {
            format!(
                "{priority} {}: {}/{} impl, {}/{} tests{blocked}",
                summary.name,
                summary.checked,
                summary.total,
                summary.checked_tests,
                summary.total_tests
            )
        }
    };

    match name {
        Some(name) => {
            let path = find_spec(name)?;
            let mut summary =
                load_spec_summary(&path).ok_or_else(|| format!("Failed to load spec '{name}'"))?;

            // Resolve blocked status by checking deps
            if !summary.depends_on.is_empty() {
                let all = load_all_summaries()?;
                let completed: std::collections::HashSet<&str> = all
                    .iter()
                    .filter(|s| s.status == super::summary::SpecStatus::Completed)
                    .map(|s| s.name.as_str())
                    .collect();
                summary.blocked = summary
                    .depends_on
                    .iter()
                    .any(|dep| !completed.contains(dep.as_str()));
            }

            if json {
                let out = serde_json::to_string_pretty(&summary)
                    .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
                println!("{out}");
            } else {
                println!("{}", format_status(&summary));
            }
        }
        None => {
            let files = if include_archived {
                collect_spec_files_with_archived()?
            } else {
                collect_spec_files()?
            };

            if files.is_empty() {
                if json {
                    println!("[]");
                } else {
                    println!("No specs found.");
                }
                return Ok(());
            }

            // Use load_all_summaries to get blocked status resolved
            let mut summaries = load_all_summaries()?;

            // Apply tag filter
            if let Some(tag_filter) = tag {
                summaries.retain(|s| s.tags.iter().any(|t| t == tag_filter));
            }

            if json {
                let out = serde_json::to_string_pretty(&summaries)
                    .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
                println!("{out}");
            } else {
                for summary in &summaries {
                    println!("{}", format_status(summary));
                }
            }
        }
    }
    Ok(())
}

/// Skill-backed command: suggests Mermaid diagram additions for a spec.
///
/// This command validates the spec exists and prints guidance directing the
/// user to the `/tinyspec:diagram` Claude skill, which does the actual work
/// (reads the spec, proposes diagrams, writes accepted ones).
pub fn diagram(name: &str) -> Result<(), String> {
    // Validate the spec exists
    let path = find_spec(name)?;
    let filename = path.file_name().unwrap().to_string_lossy();

    println!("Spec: {filename}");
    println!();
    println!("tinyspec diagram is a skill-backed command.");
    println!("Run it through Claude Code with:");
    println!();
    println!("  /tinyspec:diagram {name}");
    println!();
    println!("Claude will analyze the spec's prose, identify sections that benefit");
    println!("from visualization, and propose Mermaid blocks with rationale for each.");

    Ok(())
}
