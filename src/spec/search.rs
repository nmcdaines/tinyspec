use super::summary::{SpecStatus, load_spec_summary};
use super::{collect_spec_files, extract_spec_name, parse_front_matter, specs_dir};
use std::fs;

pub fn search(
    query: &str,
    group_filter: Option<&str>,
    status_filter: Option<&str>,
) -> Result<(), String> {
    let mut files = collect_spec_files()?;

    if files.is_empty() {
        println!("No specs found.");
        return Ok(());
    }

    // Sort for consistent output
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let specs_root = specs_dir();
    let query_lower = query.to_lowercase();

    let mut found_any = false;

    for path in &files {
        // Group filter
        if let Some(group) = group_filter {
            let parent = path.parent().unwrap_or(&specs_root);
            let file_group = if parent == specs_root {
                None
            } else {
                parent.file_name().and_then(|g| g.to_str())
            };
            if file_group != Some(group) {
                continue;
            }
        }

        // Status filter
        if let Some(status) = status_filter {
            if let Some(summary) = load_spec_summary(path) {
                let matches = match status {
                    "pending" => summary.status == SpecStatus::Pending,
                    "in-progress" => summary.status == SpecStatus::InProgress,
                    "completed" => summary.status == SpecStatus::Completed,
                    _ => {
                        return Err(format!(
                            "Invalid status filter '{status}'. Use: pending, in-progress, completed"
                        ));
                    }
                };
                if !matches {
                    continue;
                }
            }
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let spec_name = extract_spec_name(&filename).unwrap_or(&filename);

        // Collect matching lines (skip front matter delimiter lines)
        let mut matching_lines: Vec<(usize, &str)> = Vec::new();
        let mut in_front_matter = false;
        let mut front_matter_count = 0;

        for (i, line) in content.lines().enumerate() {
            if line.trim() == "---" {
                front_matter_count += 1;
                if front_matter_count <= 2 {
                    in_front_matter = front_matter_count == 1;
                    continue;
                }
            }
            if in_front_matter {
                continue;
            }

            if line.to_lowercase().contains(&query_lower) {
                matching_lines.push((i + 1, line));
            }
        }

        // Also check title (from front matter) separately
        let title = parse_front_matter(&content)
            .and_then(|fm| fm.title)
            .unwrap_or_default();

        if matching_lines.is_empty() && !title.to_lowercase().contains(&query_lower) {
            continue;
        }

        found_any = true;

        // Print spec header
        let display_name = {
            let parent = path.parent().unwrap_or(&specs_root);
            if parent != specs_root {
                let group = parent
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                format!("{group}/{spec_name}")
            } else {
                spec_name.to_string()
            }
        };

        println!("{display_name}  {title}");

        // Print matching lines as snippets
        for (line_num, line) in &matching_lines {
            let trimmed = line.trim();
            // Truncate long lines
            let snippet = if trimmed.len() > 120 {
                &trimmed[..120]
            } else {
                trimmed
            };
            println!("  line {line_num}: {snippet}");
        }

        println!();
    }

    if !found_any {
        println!("No specs matched '{query}'.");
    }

    Ok(())
}
