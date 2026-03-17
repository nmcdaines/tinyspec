use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub repositories: std::collections::BTreeMap<String, String>,
    /// Map of event name → list of shell commands to run.
    #[serde(default)]
    pub hooks: HashMap<String, Vec<String>>,
}

pub(crate) fn config_path() -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("TINYSPEC_HOME") {
        return Ok(PathBuf::from(dir).join("config.yaml"));
    }
    let home =
        std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    Ok(PathBuf::from(home).join(".tinyspec").join("config.yaml"))
}

pub(crate) fn load_config() -> Result<Config, String> {
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
        println!("Use `tinyspec config set <repo-name> <path>` to add a repository mapping.");
        return Ok(());
    }
    for (name, path) in &config.repositories {
        println!("{name}: {path}");
    }
    Ok(())
}

/// Load hooks from the project-level `.tinyspec.yaml` if it exists.
pub(crate) fn load_project_hooks() -> Result<HashMap<String, Vec<String>>, String> {
    // Walk up to find the project root (same heuristic as specs_dir)
    let mut dir = std::env::current_dir().map_err(|e| format!("Cannot get cwd: {e}"))?;
    loop {
        let candidate = dir.join(".tinyspec.yaml");
        if candidate.exists() {
            let content = fs::read_to_string(&candidate)
                .map_err(|e| format!("Failed to read .tinyspec.yaml: {e}"))?;
            if content.trim().is_empty() {
                return Ok(HashMap::new());
            }
            let cfg: Config = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse .tinyspec.yaml: {e}"))?;
            return Ok(cfg.hooks);
        }
        if dir.join(".specs").is_dir() || !dir.pop() {
            break;
        }
    }
    Ok(HashMap::new())
}

/// Load merged hooks: project-level hooks first, then user-level hooks appended.
pub(crate) fn load_merged_hooks() -> Result<HashMap<String, Vec<String>>, String> {
    let user_hooks = load_config()?.hooks;
    let project_hooks = load_project_hooks()?;

    // Merge: for each event, project hooks run first, then user hooks
    let mut merged: HashMap<String, Vec<String>> = project_hooks;
    for (event, cmds) in user_hooks {
        merged.entry(event).or_default().extend(cmds);
    }
    Ok(merged)
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
