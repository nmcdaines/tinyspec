use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use super::{collect_spec_files, extract_spec_name, parse_front_matter, specs_dir};

#[derive(Debug, Clone)]
pub struct TaskNode {
    pub id: String,
    pub description: String,
    pub checked: bool,
    pub children: Vec<TaskNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecStatus {
    InProgress,
    Pending,
    Completed,
}

impl SpecStatus {
    fn sort_key(&self) -> u8 {
        match self {
            SpecStatus::InProgress => 0,
            SpecStatus::Pending => 1,
            SpecStatus::Completed => 2,
        }
    }
}

impl Ord for SpecStatus {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

impl PartialOrd for SpecStatus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct SpecSummary {
    pub name: String,
    pub title: String,
    pub group: Option<String>,
    pub timestamp: String, // "YYYY-MM-DD HH:MM"
    pub total: u32,
    pub checked: u32,
    pub status: SpecStatus,
    pub tasks: Vec<TaskNode>,
}

/// Extract a human-friendly timestamp from a spec filename.
/// `"2026-02-17-21-27-dashboard.md"` → `"2026-02-17 21:27"`
fn extract_timestamp(filename: &str) -> String {
    if filename.len() >= 16 {
        let raw = &filename[..16];
        format!("{} {}:{}", &raw[..10], &raw[11..13], &raw[14..16])
    } else {
        String::new()
    }
}

/// Parse the Implementation Plan section into a task tree.
fn parse_tasks(content: &str) -> Vec<TaskNode> {
    let mut in_plan = false;
    let mut tasks: Vec<TaskNode> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "# Implementation Plan" {
            in_plan = true;
            continue;
        }

        // Stop at next top-level heading
        if in_plan && trimmed.starts_with("# ") {
            break;
        }

        if !in_plan {
            continue;
        }

        let (is_checked, rest) = if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            (true, rest)
        } else if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            (false, rest)
        } else {
            continue;
        };

        // Parse "ID: description"
        let Some(colon_pos) = rest.find(':') else {
            continue;
        };
        let id = rest[..colon_pos].trim().to_string();
        let description = rest[colon_pos + 1..].trim().to_string();

        // Determine nesting by leading whitespace on the original line
        let indent = line.len() - line.trim_start().len();

        if indent == 0 {
            // Top-level task
            tasks.push(TaskNode {
                id,
                description,
                checked: is_checked,
                children: Vec::new(),
            });
        } else if let Some(parent) = tasks.last_mut() {
            // Subtask
            parent.children.push(TaskNode {
                id,
                description,
                checked: is_checked,
                children: Vec::new(),
            });
        }
    }

    tasks
}

/// Count total and checked tasks (including all nesting levels).
fn count_tasks(tasks: &[TaskNode]) -> (u32, u32) {
    let mut total = 0u32;
    let mut checked = 0u32;
    for task in tasks {
        total += 1;
        if task.checked {
            checked += 1;
        }
        for child in &task.children {
            total += 1;
            if child.checked {
                checked += 1;
            }
        }
    }
    (total, checked)
}

/// Load a single spec file into a SpecSummary.
pub fn load_spec_summary(path: &Path) -> Option<SpecSummary> {
    let filename = path.file_name()?.to_str()?;
    let name = extract_spec_name(filename)?.to_string();
    let timestamp = extract_timestamp(filename);
    let content = fs::read_to_string(path).ok()?;

    let title = parse_front_matter(&content)
        .and_then(|fm| fm.title)
        .unwrap_or_else(|| name.clone());

    let group = {
        let specs_root = specs_dir();
        let parent = path.parent()?;
        if parent != specs_root {
            parent
                .file_name()
                .and_then(|g| g.to_str())
                .map(String::from)
        } else {
            None
        }
    };

    let tasks = parse_tasks(&content);
    let (total, checked) = count_tasks(&tasks);

    let status = if total == 0 {
        SpecStatus::Pending
    } else if checked == total {
        SpecStatus::Completed
    } else if checked > 0 {
        SpecStatus::InProgress
    } else {
        SpecStatus::Pending
    };

    Some(SpecSummary {
        name,
        title,
        group,
        timestamp,
        total,
        checked,
        status,
        tasks,
    })
}

/// Load all specs and return them sorted by completion (incomplete first, then completed),
/// then by group name, then by timestamp within each group.
pub fn load_all_summaries() -> Result<Vec<SpecSummary>, String> {
    let files = collect_spec_files()?;
    let mut summaries: Vec<SpecSummary> = files
        .iter()
        .filter_map(|path| load_spec_summary(path))
        .collect();

    summaries.sort_by(|a, b| {
        let a_done = a.status == SpecStatus::Completed;
        let b_done = b.status == SpecStatus::Completed;
        a_done
            .cmp(&b_done) // incomplete (false) before completed (true)
            .then_with(|| a.group.cmp(&b.group))
            .then_with(|| {
                if a_done && b_done {
                    b.timestamp.cmp(&a.timestamp) // completed: newest first
                } else {
                    a.timestamp.cmp(&b.timestamp) // in-progress: oldest first
                }
            })
    });

    Ok(summaries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tasks_from_plan() {
        let content = "\
# Background

Some background.

# Implementation Plan

- [ ] A: First task
  - [x] A.1: Subtask one
  - [ ] A.2: Subtask two
- [x] B: Second task

# Test Plan
";
        let tasks = parse_tasks(content);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, "A");
        assert!(!tasks[0].checked);
        assert_eq!(tasks[0].children.len(), 2);
        assert!(tasks[0].children[0].checked);
        assert!(!tasks[0].children[1].checked);
        assert_eq!(tasks[1].id, "B");
        assert!(tasks[1].checked);
    }

    #[test]
    fn count_tasks_correctly() {
        let tasks = vec![
            TaskNode {
                id: "A".into(),
                description: "Task A".into(),
                checked: false,
                children: vec![
                    TaskNode {
                        id: "A.1".into(),
                        description: "Sub".into(),
                        checked: true,
                        children: vec![],
                    },
                    TaskNode {
                        id: "A.2".into(),
                        description: "Sub".into(),
                        checked: false,
                        children: vec![],
                    },
                ],
            },
            TaskNode {
                id: "B".into(),
                description: "Task B".into(),
                checked: true,
                children: vec![],
            },
        ];
        let (total, checked) = count_tasks(&tasks);
        assert_eq!(total, 4);
        assert_eq!(checked, 2);
    }

    #[test]
    fn status_sort_order() {
        assert!(SpecStatus::InProgress < SpecStatus::Pending);
        assert!(SpecStatus::Pending < SpecStatus::Completed);
    }

    #[test]
    fn extract_timestamp_from_filename() {
        assert_eq!(
            extract_timestamp("2026-02-17-21-27-dashboard.md"),
            "2026-02-17 21:27"
        );
        assert_eq!(
            extract_timestamp("2025-01-05-09-00-hello-world.md"),
            "2025-01-05 09:00"
        );
        assert_eq!(extract_timestamp("short.md"), "");
    }
}
