use std::fs;
use std::path::Path;

const TINYSPEC_REFINE_SKILL: &str = include_str!("../skills/tinyspec-refine.md");
const TINYSPEC_WORK_SKILL: &str = include_str!("../skills/tinyspec-work.md");
const TINYSPEC_TASK_SKILL: &str = include_str!("../skills/tinyspec-task.md");
const TINYSPEC_ONESHOT_SKILL: &str = include_str!("../skills/tinyspec-oneshot.md");

pub fn init(force: bool) -> Result<(), String> {
    // Remove legacy .claude/commands/tinyspec*.md files when --force is used
    if force {
        let commands_dir = Path::new(".claude/commands");
        if commands_dir.is_dir()
            && let Ok(entries) = fs::read_dir(commands_dir)
        {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with("tinyspec")
                    && name.ends_with(".md")
                    && fs::remove_file(entry.path()).is_ok()
                {
                    println!("Removed legacy .claude/commands/{name}");
                }
            }
        }
    }

    let skills_dir = Path::new(".claude/skills");
    let skills: &[(&str, &str)] = &[
        ("tinyspec-refine", TINYSPEC_REFINE_SKILL),
        ("tinyspec-work", TINYSPEC_WORK_SKILL),
        ("tinyspec-task", TINYSPEC_TASK_SKILL),
        ("tinyspec-oneshot", TINYSPEC_ONESHOT_SKILL),
    ];

    for (skill_name, content) in skills {
        let dir = skills_dir.join(skill_name);
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create .claude/skills/{skill_name}/ directory: {e}"))?;
        let path = dir.join("SKILL.md");
        if !force && path.exists() {
            println!("Skipped {skill_name}/SKILL.md (already exists)");
        } else {
            fs::write(&path, content)
                .map_err(|e| format!("Failed to write {skill_name}/SKILL.md: {e}"))?;
            println!("Created {skill_name}/SKILL.md");
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
