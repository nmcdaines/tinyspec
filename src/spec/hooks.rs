use std::collections::HashMap;
use std::process::Command;

use super::config::load_merged_hooks;

/// All lifecycle events that can trigger hooks.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    OnTaskCheck,
    OnTaskUncheck,
    OnSpecComplete,
    OnSpecStart,
    OnSpecCreate,
}

impl Event {
    pub fn as_str(&self) -> &'static str {
        match self {
            Event::OnTaskCheck => "on_task_check",
            Event::OnTaskUncheck => "on_task_uncheck",
            Event::OnSpecComplete => "on_spec_complete",
            Event::OnSpecStart => "on_spec_start",
            Event::OnSpecCreate => "on_spec_create",
        }
    }

    pub fn from_str(s: &str) -> Option<Event> {
        match s {
            "on_task_check" => Some(Event::OnTaskCheck),
            "on_task_uncheck" => Some(Event::OnTaskUncheck),
            "on_spec_complete" => Some(Event::OnSpecComplete),
            "on_spec_start" => Some(Event::OnSpecStart),
            "on_spec_create" => Some(Event::OnSpecCreate),
            _ => None,
        }
    }

    /// All valid event names, for use in help text and test command.
    pub fn all_names() -> &'static [&'static str] {
        &[
            "on_task_check",
            "on_task_uncheck",
            "on_spec_complete",
            "on_spec_start",
            "on_spec_create",
        ]
    }
}

/// Context passed to hook commands via environment variables.
pub struct HookContext {
    pub event: Event,
    pub spec_name: String,
    pub spec_title: String,
    pub spec_group: String,
    pub task_id: String,
    pub spec_path: String,
}

impl HookContext {
    pub fn to_env_vars(&self) -> HashMap<&'static str, String> {
        let mut vars = HashMap::new();
        vars.insert("TINYSPEC_EVENT", self.event.as_str().to_string());
        vars.insert("TINYSPEC_SPEC", self.spec_name.clone());
        vars.insert("TINYSPEC_SPEC_TITLE", self.spec_title.clone());
        vars.insert("TINYSPEC_SPEC_GROUP", self.spec_group.clone());
        vars.insert("TINYSPEC_TASK_ID", self.task_id.clone());
        vars.insert("TINYSPEC_SPEC_PATH", self.spec_path.clone());
        vars
    }
}

/// Execute all configured hooks for the given event.
/// Hooks that exit non-zero print a warning but do not block the calling command.
pub fn run_hooks(context: &HookContext) {
    let hooks = match load_merged_hooks() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Warning: failed to load hook config: {e}");
            return;
        }
    };

    let event_key = context.event.as_str();
    let Some(commands) = hooks.get(event_key) else {
        return;
    };

    let env_vars = context.to_env_vars();

    for cmd in commands {
        let status = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .envs(&env_vars)
            .status();

        match status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                eprintln!(
                    "Warning: hook command exited with status {}: {cmd}",
                    s.code().unwrap_or(-1)
                );
            }
            Err(e) => {
                eprintln!("Warning: failed to run hook command '{cmd}': {e}");
            }
        }
    }
}

/// Fire a named event with dummy context data (for `tinyspec hooks test`).
pub fn test_hook(event_name: &str) -> Result<(), String> {
    let event = Event::from_str(event_name).ok_or_else(|| {
        format!(
            "Unknown event '{event_name}'.\nValid events: {}",
            Event::all_names().join(", ")
        )
    })?;

    let hooks = load_merged_hooks()?;
    let commands = hooks.get(event_name).cloned().unwrap_or_default();

    if commands.is_empty() {
        println!("No hooks configured for '{event_name}'.");
        return Ok(());
    }

    let context = HookContext {
        event,
        spec_name: "test-spec".into(),
        spec_title: "Test Spec".into(),
        spec_group: "".into(),
        task_id: "A.1".into(),
        spec_path: "/path/to/.specs/test-spec.md".into(),
    };

    let env_vars = context.to_env_vars();

    println!("Testing hooks for event '{event_name}':");
    for cmd in &commands {
        println!("  Running: {cmd}");
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .envs(&env_vars)
            .output();

        match output {
            Ok(out) => {
                let code = out.status.code().unwrap_or(-1);
                if !out.stdout.is_empty() {
                    print!("  stdout: {}", String::from_utf8_lossy(&out.stdout));
                }
                if !out.stderr.is_empty() {
                    eprint!("  stderr: {}", String::from_utf8_lossy(&out.stderr));
                }
                println!("  exit code: {code}");
            }
            Err(e) => {
                eprintln!("  error: {e}");
            }
        }
    }

    Ok(())
}
