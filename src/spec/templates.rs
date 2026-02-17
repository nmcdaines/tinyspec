use std::fmt;
use std::fs;
use std::path::PathBuf;

use super::specs_dir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateSource {
    Repo,
    User,
}

impl fmt::Display for TemplateSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateSource::Repo => write!(f, "repo"),
            TemplateSource::User => write!(f, "user"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub path: PathBuf,
    pub source: TemplateSource,
}

/// Repo-level templates directory: `.specs/templates/`
pub fn repo_templates_dir() -> PathBuf {
    specs_dir().join("templates")
}

/// User-level templates directory: `~/.config/tinyspec/templates/`
pub fn user_templates_dir() -> Result<PathBuf, String> {
    let home =
        std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("tinyspec")
        .join("templates"))
}

/// Extract template name from a filename (strip `.md` extension).
fn template_name(filename: &str) -> Option<&str> {
    filename.strip_suffix(".md")
}

/// Scan a directory for `.md` template files.
fn scan_templates(dir: &PathBuf, source: TemplateSource) -> Vec<TemplateInfo> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                let filename = path.file_name()?.to_str()?;
                let name = template_name(filename)?.to_string();
                Some(TemplateInfo {
                    name,
                    path,
                    source: source.clone(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Collect all available templates from both repo and user directories.
/// Repo-level templates take precedence over user-level on name conflicts.
pub fn collect_templates() -> Result<Vec<TemplateInfo>, String> {
    let mut templates = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    // Repo templates first (higher precedence)
    for t in scan_templates(&repo_templates_dir(), TemplateSource::Repo) {
        seen_names.insert(t.name.clone());
        templates.push(t);
    }

    // User templates (skip if name already seen from repo)
    let user_dir = user_templates_dir()?;
    for t in scan_templates(&user_dir, TemplateSource::User) {
        if !seen_names.contains(&t.name) {
            templates.push(t);
        }
    }

    templates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(templates)
}

/// Find a specific template by name.
pub fn find_template(name: &str) -> Result<TemplateInfo, String> {
    let templates = collect_templates()?;
    templates
        .into_iter()
        .find(|t| t.name == name)
        .ok_or_else(|| format!("No template found matching '{name}'"))
}

/// List all available templates, showing name and source.
pub fn list_templates() -> Result<(), String> {
    let templates = collect_templates()?;

    if templates.is_empty() {
        println!("No templates found.");
        println!();
        println!("Create templates as Markdown files in:");
        println!("  .specs/templates/       (repo-level)");
        println!("  ~/.config/tinyspec/templates/  (user-level)");
        return Ok(());
    }

    for t in &templates {
        println!("{:30} ({})", t.name, t.source);
    }

    Ok(())
}
