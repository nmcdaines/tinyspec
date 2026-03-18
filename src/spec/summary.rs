use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use serde::Serialize;

use super::{Priority, collect_spec_files, extract_spec_name, parse_front_matter, specs_dir};

#[derive(Debug, Clone, Serialize)]
pub struct TaskNode {
    pub id: String,
    pub description: String,
    pub checked: bool,
    pub children: Vec<TaskNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct SpecSummary {
    pub name: String,
    pub title: String,
    pub group: Option<String>,
    pub timestamp: String, // "YYYY-MM-DD HH:MM"
    pub total: u32,
    pub checked: u32,
    pub total_tests: u32,
    pub checked_tests: u32,
    pub status: SpecStatus,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub depends_on: Vec<String>,
    pub blocked: bool,
    pub tasks: Vec<TaskNode>,
    pub test_tasks: Vec<TaskNode>,
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

/// Parse a specific headed section (e.g. `# Implementation Plan` or `# Test Plan`)
/// into a task tree. Stops at the next top-level `#` heading.
fn parse_section_tasks(content: &str, section_heading: &str) -> Vec<TaskNode> {
    let mut in_section = false;
    let mut tasks: Vec<TaskNode> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == section_heading {
            in_section = true;
            continue;
        }

        // Stop at next top-level heading
        if in_section && trimmed.starts_with("# ") {
            break;
        }

        if !in_section {
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
            tasks.push(TaskNode {
                id,
                description,
                checked: is_checked,
                children: Vec::new(),
            });
        } else if let Some(parent) = tasks.last_mut() {
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

/// Parse the `# Implementation Plan` section into a task tree.
pub fn parse_tasks_from_content(content: &str) -> Vec<TaskNode> {
    parse_section_tasks(content, "# Implementation Plan")
}

/// Parse the `# Test Plan` section into a task tree.
pub fn parse_test_tasks_from_content(content: &str) -> Vec<TaskNode> {
    parse_section_tasks(content, "# Test Plan")
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

    let fm = parse_front_matter(&content);
    let title = fm
        .as_ref()
        .and_then(|f| f.title.clone())
        .unwrap_or_else(|| name.clone());
    let priority = fm.as_ref().and_then(|f| f.priority).unwrap_or_default();
    let tags = fm.as_ref().map(|f| f.tags.clone()).unwrap_or_default();
    let depends_on = fm
        .as_ref()
        .map(|f| f.depends_on.clone())
        .unwrap_or_default();

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

    let tasks = parse_tasks_from_content(&content);
    let (total, checked) = count_tasks(&tasks);

    let test_tasks = parse_test_tasks_from_content(&content);
    let (total_tests, checked_tests) = count_tasks(&test_tasks);

    let status = if total == 0 && total_tests == 0 {
        SpecStatus::Pending
    } else if checked == total && checked_tests == total_tests {
        SpecStatus::Completed
    } else if checked > 0 || checked_tests > 0 {
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
        total_tests,
        checked_tests,
        status,
        priority,
        tags,
        depends_on,
        blocked: false, // resolved later by load_all_summaries
        tasks,
        test_tasks,
    })
}

/// Perform a topological sort of spec names based on `depends_on`.
/// Returns `Err` with the cycle participants if a cycle is detected.
pub fn detect_dependency_cycles(summaries: &[SpecSummary]) -> Result<Vec<String>, Vec<String>> {
    use std::collections::{HashMap, VecDeque};

    // Build adjacency map: spec -> depends_on specs
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for s in summaries {
        in_degree.entry(&s.name).or_insert(0);
        for dep in &s.depends_on {
            // Only count deps that reference known specs
            if summaries.iter().any(|other| other.name == *dep) {
                *in_degree.entry(&s.name).or_insert(0) += 1;
                dependents.entry(dep.as_str()).or_default().push(&s.name);
            }
        }
    }

    // Kahn's algorithm
    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|&(_, &deg)| deg == 0)
        .map(|(&name, _)| name)
        .collect();

    let mut sorted = Vec::new();
    while let Some(node) = queue.pop_front() {
        sorted.push(node.to_string());
        if let Some(deps) = dependents.get(node) {
            for &dep in deps {
                if let Some(deg) = in_degree.get_mut(dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }
    }

    if sorted.len() == in_degree.len() {
        Ok(sorted)
    } else {
        // Specs not in sorted are in cycles
        let cycle: Vec<String> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg > 0)
            .map(|(&name, _)| name.to_string())
            .collect();
        Err(cycle)
    }
}

/// Resolve `blocked` field for all summaries based on `depends_on` references.
fn resolve_blocked(summaries: &mut [SpecSummary]) {
    // Collect completed spec names
    let completed: std::collections::HashSet<String> = summaries
        .iter()
        .filter(|s| s.status == SpecStatus::Completed)
        .map(|s| s.name.clone())
        .collect();

    for summary in summaries.iter_mut() {
        if !summary.depends_on.is_empty() {
            summary.blocked = summary
                .depends_on
                .iter()
                .any(|dep| !completed.contains(dep));
        }
    }
}

/// Load all specs and return them sorted by completion (incomplete first, then completed),
/// then by priority within status group, then by group name, then by timestamp.
pub fn load_all_summaries() -> Result<Vec<SpecSummary>, String> {
    let files = collect_spec_files()?;
    let mut summaries: Vec<SpecSummary> = files
        .iter()
        .filter_map(|path| load_spec_summary(path))
        .collect();

    resolve_blocked(&mut summaries);

    summaries.sort_by(|a, b| {
        let a_done = a.status == SpecStatus::Completed;
        let b_done = b.status == SpecStatus::Completed;
        a_done
            .cmp(&b_done) // incomplete (false) before completed (true)
            .then_with(|| a.priority.cmp(&b.priority)) // High < Medium < Low
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
        let tasks = parse_tasks_from_content(content);
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
    fn parse_tasks_with_emoji_group_ids() {
        let content = "\
# Implementation Plan

- [ ] 🧪: Testing tasks
  - [x] 🧪.1: Write unit tests
  - [ ] 🧪.2: Write integration tests
- [ ] 🚀: Deployment tasks
  - [ ] 🚀.1: Deploy to staging

# Test Plan
";
        let tasks = parse_tasks_from_content(content);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, "🧪");
        assert!(!tasks[0].checked);
        assert_eq!(tasks[0].children.len(), 2);
        assert_eq!(tasks[0].children[0].id, "🧪.1");
        assert!(tasks[0].children[0].checked);
        assert_eq!(tasks[0].children[1].id, "🧪.2");
        assert!(!tasks[0].children[1].checked);
        assert_eq!(tasks[1].id, "🚀");
        assert_eq!(tasks[1].children.len(), 1);
        assert_eq!(tasks[1].children[0].id, "🚀.1");
    }

    #[test]
    fn parse_test_tasks_from_plan() {
        let content = "\
# Implementation Plan

- [x] A: Impl task

# Test Plan

- [ ] T.1: First test
- [x] T.2: Second test
  - [ ] T.2.1: Sub-test
";
        let impl_tasks = parse_tasks_from_content(content);
        let test_tasks = parse_test_tasks_from_content(content);
        assert_eq!(impl_tasks.len(), 1);
        assert_eq!(impl_tasks[0].id, "A");
        assert_eq!(test_tasks.len(), 2);
        assert_eq!(test_tasks[0].id, "T.1");
        assert!(!test_tasks[0].checked);
        assert_eq!(test_tasks[1].id, "T.2");
        assert!(test_tasks[1].checked);
        assert_eq!(test_tasks[1].children.len(), 1);
        assert_eq!(test_tasks[1].children[0].id, "T.2.1");
    }

    #[test]
    fn completed_requires_all_test_tasks_checked() {
        // Spec with impl done but test task pending → InProgress
        let content = "\
# Implementation Plan

- [x] A: Done

# Test Plan

- [ ] T.1: Not done yet
";
        let tasks = parse_tasks_from_content(content);
        let test_tasks = parse_test_tasks_from_content(content);
        let (total, checked) = count_tasks(&tasks);
        let (total_tests, checked_tests) = count_tasks(&test_tasks);
        let status = if total == 0 && total_tests == 0 {
            SpecStatus::Pending
        } else if checked == total && checked_tests == total_tests {
            SpecStatus::Completed
        } else if checked > 0 || checked_tests > 0 {
            SpecStatus::InProgress
        } else {
            SpecStatus::Pending
        };
        assert_eq!(status, SpecStatus::InProgress);
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
