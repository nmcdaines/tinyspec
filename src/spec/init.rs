use std::fs;
use std::path::Path;

const TINYSPEC_REFINE_SKILL: &str = include_str!("../skills/tinyspec-refine.md");
const TINYSPEC_DO_SKILL: &str = include_str!("../skills/tinyspec-do.md");
const TINYSPEC_TASK_SKILL: &str = include_str!("../skills/tinyspec-task.md");
const TINYSPEC_ONESHOT_SKILL: &str = include_str!("../skills/tinyspec-oneshot.md");

fn remove_matching_entries(
    dir: &Path,
    label: &str,
    filter: impl Fn(&str, &Path) -> bool,
    remove: impl for<'a> Fn(&'a Path) -> std::io::Result<()>,
) {
    if dir.is_dir()
        && let Ok(entries) = fs::read_dir(dir)
    {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let path = entry.path();
            if filter(&name, &path) && remove(&path).is_ok() {
                println!("Removed legacy {label}/{name}");
            }
        }
    }
}

pub fn init(force: bool) -> Result<(), String> {
    let skills_dir = Path::new(".claude/skills");

    // Remove legacy .claude/commands/tinyspec*.md files and stale
    // .claude/skills/tinyspec-* dirs when --force is used
    if force {
        remove_matching_entries(
            Path::new(".claude/commands"),
            ".claude/commands",
            |name, _| name.starts_with("tinyspec") && name.ends_with(".md"),
            |path| fs::remove_file(path),
        );
        remove_matching_entries(
            skills_dir,
            ".claude/skills",
            |name, path| name.starts_with("tinyspec-") && path.is_dir(),
            |path| fs::remove_dir_all(path),
        );
    }
    let skills: &[(&str, &str)] = &[
        ("tinyspec-refine", TINYSPEC_REFINE_SKILL),
        ("tinyspec-do", TINYSPEC_DO_SKILL),
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
