use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper: create a tinyspec command that runs in the given directory.
fn tinyspec(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("tinyspec").unwrap();
    cmd.current_dir(dir.path());
    cmd
}

/// Helper: create a `.specs/` directory with a sample spec file.
fn create_sample_spec(dir: &TempDir, filename: &str, content: &str) {
    let specs = dir.path().join(".specs");
    fs::create_dir_all(&specs).unwrap();
    fs::write(specs.join(filename), content).unwrap();
}

/// Helper: create a spec file inside a group subdirectory.
fn create_grouped_spec(dir: &TempDir, group: &str, filename: &str, content: &str) {
    let group_dir = dir.path().join(".specs").join(group);
    fs::create_dir_all(&group_dir).unwrap();
    fs::write(group_dir.join(filename), content).unwrap();
}

fn sample_spec_content() -> String {
    "\
---
tinySpec: v0
title: Hello World
applications:
    - my-app
---

# Background

Some background.

# Proposal

Some proposal.

# Implementation Plan

- [ ] A: Do this
    - [ ] A.1: Do this subtask
    - [ ] A.2: Do this other subtask

- [ ] B: Do that
    - [ ] B.1: Subtask one
    - [ ] B.2: Subtask two
    - [ ] B.3: Subtask three

# Test Plan

"
    .to_string()
}

// ─── T.1: Create a new spec with valid name ─────────────────────────────────

#[test]
fn t1_new_creates_spec_with_valid_name() {
    let dir = TempDir::new().unwrap();

    tinyspec(&dir)
        .args(["new", "my-feature"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created spec:"))
        .stdout(predicate::str::contains("my-feature.md"));

    // Verify file exists in .specs/
    let specs = dir.path().join(".specs");
    let entries: Vec<_> = fs::read_dir(&specs)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 1);

    let filename = entries[0].file_name().to_string_lossy().to_string();
    assert!(filename.ends_with("-my-feature.md"));

    // Verify content
    let content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(content.contains("tinySpec: v0"));
    assert!(content.contains("title: My Feature"));
    assert!(content.contains("# Background"));
    assert!(content.contains("# Proposal"));
    assert!(content.contains("# Implementation Plan"));
    assert!(content.contains("# Test Plan"));
}

// ─── T.2: Reject invalid spec names ─────────────────────────────────────────

#[test]
fn t2_new_rejects_invalid_names() {
    let dir = TempDir::new().unwrap();

    // Uppercase
    tinyspec(&dir)
        .args(["new", "MyFeature"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("kebab-case"));

    // Spaces
    tinyspec(&dir)
        .args(["new", "my feature"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("kebab-case"));

    // Leading hyphen (use -- to pass through clap)
    tinyspec(&dir)
        .args(["new", "--", "-bad"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("kebab-case"));

    // Double hyphens
    tinyspec(&dir)
        .args(["new", "bad--name"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("kebab-case"));
}

// ─── T.3: List all specs ────────────────────────────────────────────────────

#[test]
fn t3_list_all_specs() {
    let dir = TempDir::new().unwrap();

    create_sample_spec(
        &dir,
        "2025-01-01-10-00-alpha.md",
        "---\ntinySpec: v0\ntitle: Alpha Spec\napplications:\n    -\n---\n\n# Background\n",
    );
    create_sample_spec(
        &dir,
        "2025-02-01-10-00-beta.md",
        "---\ntinySpec: v0\ntitle: Beta Spec\napplications:\n    -\n---\n\n# Background\n",
    );

    tinyspec(&dir)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("Alpha Spec"))
        .stdout(predicate::str::contains("beta"))
        .stdout(predicate::str::contains("Beta Spec"));
}

// ─── T.4: View an existing spec ─────────────────────────────────────────────

#[test]
fn t4_view_existing_spec() {
    let dir = TempDir::new().unwrap();
    let content = sample_spec_content();
    create_sample_spec(&dir, "2025-02-17-09-36-hello-world.md", &content);

    // Set up config so applications can be resolved
    let config_dir = dir.path().join(".tinyspec-config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.yaml"),
        "repositories:\n  my-app: /path/to/my-app\n",
    )
    .unwrap();

    tinyspec(&dir)
        .env("TINYSPEC_HOME", config_dir.to_str().unwrap())
        .args(["view", "hello-world"])
        .assert()
        .success()
        .stdout(predicate::str::contains("title: Hello World"))
        .stdout(predicate::str::contains("# Background"));
}

// ─── T.5: View a non-existent spec ──────────────────────────────────────────

#[test]
fn t5_view_nonexistent_spec() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(".specs")).unwrap();

    tinyspec(&dir)
        .args(["view", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No spec found matching"));
}

// ─── T.6: Delete a spec ─────────────────────────────────────────────────────

#[test]
fn t6_delete_spec() {
    let dir = TempDir::new().unwrap();
    create_sample_spec(
        &dir,
        "2025-02-17-09-36-hello-world.md",
        &sample_spec_content(),
    );

    // Confirm deletion by piping "y" to stdin
    tinyspec(&dir)
        .args(["delete", "hello-world"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted"));

    assert!(
        !dir.path()
            .join(".specs/2025-02-17-09-36-hello-world.md")
            .exists()
    );
}

// ─── T.7: Check a task ──────────────────────────────────────────────────────

#[test]
fn t7_check_task() {
    let dir = TempDir::new().unwrap();
    create_sample_spec(
        &dir,
        "2025-02-17-09-36-hello-world.md",
        &sample_spec_content(),
    );

    tinyspec(&dir)
        .args(["check", "hello-world", "A"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Checked task A"));

    let content =
        fs::read_to_string(dir.path().join(".specs/2025-02-17-09-36-hello-world.md")).unwrap();
    assert!(content.contains("- [x] A: Do this"));
    // Other tasks untouched
    assert!(content.contains("- [ ] A.1: Do this subtask"));
    assert!(content.contains("- [ ] B: Do that"));
}

// ─── T.8: Uncheck a task ────────────────────────────────────────────────────

#[test]
fn t8_uncheck_task() {
    let dir = TempDir::new().unwrap();
    let content = sample_spec_content().replace("- [ ] B: Do that", "- [x] B: Do that");
    create_sample_spec(&dir, "2025-02-17-09-36-hello-world.md", &content);

    tinyspec(&dir)
        .args(["uncheck", "hello-world", "B"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Unchecked task B"));

    let content =
        fs::read_to_string(dir.path().join(".specs/2025-02-17-09-36-hello-world.md")).unwrap();
    assert!(content.contains("- [ ] B: Do that"));
}

// ─── T.9: Check with invalid task ID ────────────────────────────────────────

#[test]
fn t9_check_invalid_task_id() {
    let dir = TempDir::new().unwrap();
    create_sample_spec(
        &dir,
        "2025-02-17-09-36-hello-world.md",
        &sample_spec_content(),
    );

    tinyspec(&dir)
        .args(["check", "hello-world", "Z"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No unchecked task 'Z' found"));
}

// ─── T.10: Show status for a spec ───────────────────────────────────────────

#[test]
fn t10_status_for_spec() {
    let dir = TempDir::new().unwrap();
    // Create a spec with 7 tasks, 3 checked
    let content = "\
---
tinySpec: v0
title: Status Test
applications:
    -
---

# Implementation Plan

- [x] A: Task one
- [x] B: Task two
- [x] C: Task three
- [ ] D: Task four
- [ ] E: Task five
- [ ] F: Task six
- [ ] G: Task seven

# Test Plan

";
    create_sample_spec(&dir, "2025-02-17-09-36-hello-world.md", content);

    tinyspec(&dir)
        .args(["status", "hello-world"])
        .assert()
        .success()
        .stdout(predicate::str::contains("3/7 tasks complete"));
}

// ─── T.11: Init creates skill files ─────────────────────────────────────────

#[test]
fn t11_init_creates_skill_files() {
    let dir = TempDir::new().unwrap();

    tinyspec(&dir)
        .args(["init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created tinyspec-refine/SKILL.md"))
        .stdout(predicate::str::contains("Created tinyspec-work/SKILL.md"))
        .stdout(predicate::str::contains("Created tinyspec-task/SKILL.md"));

    let skills_dir = dir.path().join(".claude/skills");
    assert!(skills_dir.join("tinyspec-refine/SKILL.md").exists());
    assert!(skills_dir.join("tinyspec-work/SKILL.md").exists());
    assert!(skills_dir.join("tinyspec-task/SKILL.md").exists());

    // Verify skill files have content
    let refine = fs::read_to_string(skills_dir.join("tinyspec-refine/SKILL.md")).unwrap();
    assert!(refine.contains("$ARGUMENTS"));
}

// ─── T.12: Init does not overwrite existing skill files ─────────────────────

#[test]
fn t12_init_no_overwrite() {
    let dir = TempDir::new().unwrap();
    let skills_dir = dir.path().join(".claude/skills/tinyspec-refine");
    fs::create_dir_all(&skills_dir).unwrap();
    fs::write(skills_dir.join("SKILL.md"), "custom content").unwrap();

    tinyspec(&dir)
        .args(["init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Skipped tinyspec-refine/SKILL.md"))
        .stdout(predicate::str::contains("Created tinyspec-work/SKILL.md"))
        .stdout(predicate::str::contains("Created tinyspec-task/SKILL.md"));

    // Verify custom content preserved
    let content = fs::read_to_string(skills_dir.join("SKILL.md")).unwrap();
    assert_eq!(content, "custom content");
}

// ─── T.13: Tab completion suggests spec names ───────────────────────────────

#[test]
fn t13_tab_completion_suggests_spec_names() {
    let dir = TempDir::new().unwrap();
    let specs = dir.path().join(".specs");
    fs::create_dir_all(&specs).unwrap();
    fs::write(
        specs.join("2025-02-17-09-36-hello-world.md"),
        "---\ntinySpec: v0\ntitle: Hello World\n---\n",
    )
    .unwrap();
    fs::write(
        specs.join("2025-03-01-14-00-auth-flow.md"),
        "---\ntinySpec: v0\ntitle: Auth Flow\n---\n",
    )
    .unwrap();

    // Trigger the bash dynamic completion mechanism by setting the internal
    // env vars that clap_complete uses when the shell's completion function
    // calls the binary.
    let mut cmd = Command::cargo_bin("tinyspec").unwrap();
    cmd.current_dir(dir.path());
    cmd.env("COMPLETE", "bash");
    cmd.env("_CLAP_COMPLETE_INDEX", "2");
    cmd.env("_CLAP_COMPLETE_COMP_TYPE", "9");
    cmd.env("_CLAP_COMPLETE_SPACE", "true");
    cmd.args(["--", "tinyspec", "view", ""]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("hello-world") && stdout.contains("auth-flow"),
        "Expected completions for hello-world and auth-flow, got: {stdout}"
    );
}

// ─── T.14: Init prints shell completion instructions ────────────────────────

#[test]
fn t14_init_prints_shell_completion_instructions() {
    let dir = TempDir::new().unwrap();

    tinyspec(&dir)
        .env("SHELL", "/bin/zsh")
        .args(["init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("source <(COMPLETE=zsh tinyspec)"));
}

// ─── T.15: Format normalizes inconsistent Markdown ──────────────────────────

#[test]
fn t15_format_normalizes_markdown() {
    let dir = TempDir::new().unwrap();

    // Spec with inconsistent spacing (extra blank lines, missing blank lines)
    let messy = "\
---
tinySpec: v0
title: Messy Spec
applications:
    - app
---


# Background



Some background text.



# Proposal
No blank line after heading.


# Implementation Plan

- [ ] A: First task

# Test Plan

";
    create_sample_spec(&dir, "2025-03-01-10-00-messy.md", messy);

    tinyspec(&dir)
        .args(["format", "messy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Formatted"));

    let formatted =
        fs::read_to_string(dir.path().join(".specs/2025-03-01-10-00-messy.md")).unwrap();

    // Headings should be present
    assert!(formatted.contains("# Background"));
    assert!(formatted.contains("# Proposal"));
    assert!(formatted.contains("# Implementation Plan"));
    assert!(formatted.contains("# Test Plan"));
    // Content preserved
    assert!(formatted.contains("Some background text."));
    assert!(formatted.contains("No blank line after heading."));
    assert!(formatted.contains("- [ ] A: First task"));

    // Formatting is idempotent: running again produces the same output
    tinyspec(&dir).args(["format", "messy"]).assert().success();
    let second = fs::read_to_string(dir.path().join(".specs/2025-03-01-10-00-messy.md")).unwrap();
    assert_eq!(formatted, second, "Formatter is not idempotent");
}

// ─── T.16: Format preserves YAML front matter ──────────────────────────────

#[test]
fn t16_format_preserves_front_matter() {
    let dir = TempDir::new().unwrap();

    let content = "\
---
tinySpec: v0
title: Front Matter Test
applications:
    - my-app
---

# Background

Some text.

# Test Plan

";
    create_sample_spec(&dir, "2025-03-01-10-00-fm-test.md", content);

    tinyspec(&dir)
        .args(["format", "fm-test"])
        .assert()
        .success();

    let formatted =
        fs::read_to_string(dir.path().join(".specs/2025-03-01-10-00-fm-test.md")).unwrap();

    // Front matter must be preserved exactly
    assert!(formatted.starts_with(
        "---\ntinySpec: v0\ntitle: Front Matter Test\napplications:\n    - my-app\n---\n"
    ));
}

// ─── T.17: Format --all formats all specs ───────────────────────────────────

#[test]
fn t17_format_all_specs() {
    let dir = TempDir::new().unwrap();

    create_sample_spec(
        &dir,
        "2025-01-01-10-00-alpha.md",
        "---\ntinySpec: v0\ntitle: Alpha\napplications:\n    -\n---\n\n# Background\n\nAlpha text.\n",
    );
    create_sample_spec(
        &dir,
        "2025-02-01-10-00-beta.md",
        "---\ntinySpec: v0\ntitle: Beta\napplications:\n    -\n---\n\n# Background\n\nBeta text.\n",
    );

    tinyspec(&dir)
        .args(["format", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Formatted 2025-01-01-10-00-alpha.md",
        ))
        .stdout(predicate::str::contains(
            "Formatted 2025-02-01-10-00-beta.md",
        ));
}

// ─── T.18: New spec is auto-formatted ───────────────────────────────────────

#[test]
fn t18_new_spec_is_auto_formatted() {
    let dir = TempDir::new().unwrap();

    tinyspec(&dir).args(["new", "auto-fmt"]).assert().success();

    let specs = dir.path().join(".specs");
    let entry = fs::read_dir(&specs)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap();

    let content = fs::read_to_string(entry.path()).unwrap();

    // Running format again should produce identical output (already formatted)
    tinyspec(&dir)
        .args(["format", "auto-fmt"])
        .assert()
        .success();

    let after_format = fs::read_to_string(entry.path()).unwrap();
    assert_eq!(content, after_format, "New spec was not already formatted");
}

// ─── T.19: Config set creates config file and adds mapping ──────────────────

#[test]
fn t19_config_set_creates_config() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".tinyspec-config");

    tinyspec(&dir)
        .env("TINYSPEC_HOME", config_dir.to_str().unwrap())
        .args(["config", "set", "tinyspec", "/path/to/tinyspec"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Set tinyspec = /path/to/tinyspec"));

    let config = fs::read_to_string(config_dir.join("config.yaml")).unwrap();
    assert!(config.contains("tinyspec"));
    assert!(config.contains("/path/to/tinyspec"));
}

// ─── T.20: Config list displays all mappings ────────────────────────────────

#[test]
fn t20_config_list_displays_mappings() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".tinyspec-config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.yaml"),
        "repositories:\n  alpha: /path/alpha\n  beta: /path/beta\n",
    )
    .unwrap();

    tinyspec(&dir)
        .env("TINYSPEC_HOME", config_dir.to_str().unwrap())
        .args(["config", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha: /path/alpha"))
        .stdout(predicate::str::contains("beta: /path/beta"));
}

// ─── T.21: Config remove deletes a mapping ──────────────────────────────────

#[test]
fn t21_config_remove_deletes_mapping() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".tinyspec-config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.yaml"),
        "repositories:\n  alpha: /path/alpha\n  beta: /path/beta\n",
    )
    .unwrap();

    tinyspec(&dir)
        .env("TINYSPEC_HOME", config_dir.to_str().unwrap())
        .args(["config", "remove", "alpha"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed alpha"));

    // Verify alpha is gone but beta remains
    let config = fs::read_to_string(config_dir.join("config.yaml")).unwrap();
    assert!(!config.contains("alpha"));
    assert!(config.contains("beta"));
}

// ─── T.22: Config set updates existing mapping ──────────────────────────────

#[test]
fn t22_config_set_updates_existing() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".tinyspec-config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.yaml"),
        "repositories:\n  myrepo: /old/path\n",
    )
    .unwrap();

    tinyspec(&dir)
        .env("TINYSPEC_HOME", config_dir.to_str().unwrap())
        .args(["config", "set", "myrepo", "/new/path"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Set myrepo = /new/path"));

    let config = fs::read_to_string(config_dir.join("config.yaml")).unwrap();
    assert!(config.contains("/new/path"));
    assert!(!config.contains("/old/path"));
}

// ─── T.23: View resolves application names when config exists ───────────────

#[test]
fn t23_view_resolves_applications() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".tinyspec-config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.yaml"),
        "repositories:\n  my-app: /resolved/my-app\n",
    )
    .unwrap();

    create_sample_spec(
        &dir,
        "2025-02-17-09-36-hello-world.md",
        &sample_spec_content(),
    );

    let output = tinyspec(&dir)
        .env("TINYSPEC_HOME", config_dir.to_str().unwrap())
        .args(["view", "hello-world"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Application name should be replaced with the resolved path
    assert!(
        stdout.contains("/resolved/my-app"),
        "Expected resolved path in output, got: {stdout}"
    );
}

// ─── T.24: View errors when config missing and applications specified ───────

#[test]
fn t24_view_errors_when_config_missing() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".nonexistent-config");

    create_sample_spec(
        &dir,
        "2025-02-17-09-36-hello-world.md",
        &sample_spec_content(),
    );

    tinyspec(&dir)
        .env("TINYSPEC_HOME", config_dir.to_str().unwrap())
        .args(["view", "hello-world"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no config file found"))
        .stderr(predicate::str::contains("tinyspec config set"));
}

// ─── T.25: View errors when application name not in config ──────────────────

#[test]
fn t25_view_errors_when_app_unmapped() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".tinyspec-config");
    fs::create_dir_all(&config_dir).unwrap();
    // Config exists but doesn't have "my-app"
    fs::write(
        config_dir.join("config.yaml"),
        "repositories:\n  other-repo: /path/other\n",
    )
    .unwrap();

    create_sample_spec(
        &dir,
        "2025-02-17-09-36-hello-world.md",
        &sample_spec_content(),
    );

    tinyspec(&dir)
        .env("TINYSPEC_HOME", config_dir.to_str().unwrap())
        .args(["view", "hello-world"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("my-app"))
        .stderr(predicate::str::contains("tinyspec config set"));
}

// ─── T.26: View works normally with no applications ─────────────────────────

#[test]
fn t26_view_no_applications_works() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".nonexistent-config");

    let content = "\
---
tinySpec: v0
title: No Apps
applications:
    -
---

# Background

Some text.
";
    create_sample_spec(&dir, "2025-02-17-09-36-no-apps.md", content);

    tinyspec(&dir)
        .env("TINYSPEC_HOME", config_dir.to_str().unwrap())
        .args(["view", "no-apps"])
        .assert()
        .success()
        .stdout(predicate::str::contains("title: No Apps"))
        .stdout(predicate::str::contains("Some text."));
}

// ─── T.27: Init generates updated slash command files ───────────────────────

#[test]
fn t27_init_generates_updated_skills() {
    let dir = TempDir::new().unwrap();

    tinyspec(&dir).args(["init"]).assert().success();

    let skills_dir = dir.path().join(".claude/skills");

    let work = fs::read_to_string(skills_dir.join("tinyspec-work/SKILL.md")).unwrap();
    assert!(
        work.contains("tinyspec view"),
        "tinyspec-work/SKILL.md should reference `tinyspec view`"
    );
    assert!(
        work.contains("multiple applications"),
        "tinyspec-work/SKILL.md should mention multiple applications"
    );

    let task = fs::read_to_string(skills_dir.join("tinyspec-task/SKILL.md")).unwrap();
    assert!(
        task.contains("tinyspec view"),
        "tinyspec-task/SKILL.md should reference `tinyspec view`"
    );
    assert!(
        task.contains("multiple applications"),
        "tinyspec-task/SKILL.md should mention multiple applications"
    );
}

// ─── T.28: Init --force removes legacy command files ─────────────────────────

#[test]
fn t28_init_force_removes_legacy_commands() {
    let dir = TempDir::new().unwrap();
    let commands_dir = dir.path().join(".claude/commands");
    fs::create_dir_all(&commands_dir).unwrap();
    fs::write(commands_dir.join("tinyspec:refine.md"), "old").unwrap();
    fs::write(commands_dir.join("tinyspec:work.md"), "old").unwrap();
    fs::write(commands_dir.join("tinyspec:task.md"), "old").unwrap();

    tinyspec(&dir)
        .args(["init", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Removed legacy .claude/commands/tinyspec:refine.md",
        ))
        .stdout(predicate::str::contains(
            "Removed legacy .claude/commands/tinyspec:work.md",
        ))
        .stdout(predicate::str::contains(
            "Removed legacy .claude/commands/tinyspec:task.md",
        ))
        .stdout(predicate::str::contains("Created tinyspec-refine/SKILL.md"))
        .stdout(predicate::str::contains("Created tinyspec-work/SKILL.md"))
        .stdout(predicate::str::contains("Created tinyspec-task/SKILL.md"));

    // Legacy files should be gone
    assert!(!commands_dir.join("tinyspec:refine.md").exists());
    assert!(!commands_dir.join("tinyspec:work.md").exists());
    assert!(!commands_dir.join("tinyspec:task.md").exists());

    // New skill files should exist
    let skills_dir = dir.path().join(".claude/skills");
    assert!(skills_dir.join("tinyspec-refine/SKILL.md").exists());
    assert!(skills_dir.join("tinyspec-work/SKILL.md").exists());
    assert!(skills_dir.join("tinyspec-task/SKILL.md").exists());
}

// ─── T.29: Create a grouped spec ────────────────────────────────────────────

#[test]
fn t29_new_grouped_spec() {
    let dir = TempDir::new().unwrap();

    tinyspec(&dir)
        .args(["new", "v1/my-feature"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created spec:"))
        .stdout(predicate::str::contains("my-feature.md"));

    // Verify file exists in .specs/v1/
    let group_dir = dir.path().join(".specs/v1");
    assert!(group_dir.exists(), ".specs/v1/ directory should exist");

    let entries: Vec<_> = fs::read_dir(&group_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 1);

    let filename = entries[0].file_name().to_string_lossy().to_string();
    assert!(filename.ends_with("-my-feature.md"));

    // Verify content
    let content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(content.contains("title: My Feature"));
}

// ─── T.30: Find/view a grouped spec by name alone ───────────────────────────

#[test]
fn t30_view_grouped_spec_by_name() {
    let dir = TempDir::new().unwrap();
    let content = "\
---
tinySpec: v0
title: Grouped Feature
applications:
    -
---

# Background

Grouped spec content.
";
    create_grouped_spec(&dir, "v1", "2025-03-01-10-00-grouped-feature.md", content);

    tinyspec(&dir)
        .args(["view", "grouped-feature"])
        .assert()
        .success()
        .stdout(predicate::str::contains("title: Grouped Feature"))
        .stdout(predicate::str::contains("Grouped spec content."));
}

// ─── T.31: List output with grouped and ungrouped specs ─────────────────────

#[test]
fn t31_list_with_groups() {
    let dir = TempDir::new().unwrap();

    create_sample_spec(
        &dir,
        "2025-01-01-10-00-alpha.md",
        "---\ntinySpec: v0\ntitle: Alpha\napplications:\n    -\n---\n\n# Background\n",
    );
    create_grouped_spec(
        &dir,
        "v1",
        "2025-02-01-10-00-beta.md",
        "---\ntinySpec: v0\ntitle: Beta\napplications:\n    -\n---\n\n# Background\n",
    );
    create_grouped_spec(
        &dir,
        "v1",
        "2025-03-01-10-00-gamma.md",
        "---\ntinySpec: v0\ntitle: Gamma\napplications:\n    -\n---\n\n# Background\n",
    );

    let output = tinyspec(&dir).args(["list"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Ungrouped spec should appear
    assert!(stdout.contains("alpha"), "Should list ungrouped spec");
    assert!(stdout.contains("Alpha"), "Should show ungrouped title");

    // Group header should appear
    assert!(stdout.contains("v1/"), "Should show group header");

    // Grouped specs should appear
    assert!(stdout.contains("beta"), "Should list grouped spec beta");
    assert!(stdout.contains("gamma"), "Should list grouped spec gamma");
}

// ─── T.32: Duplicate names across groups are rejected ───────────────────────

#[test]
fn t32_reject_duplicate_names_across_groups() {
    let dir = TempDir::new().unwrap();

    // Create an ungrouped spec named "my-feature"
    create_sample_spec(
        &dir,
        "2025-01-01-10-00-my-feature.md",
        "---\ntinySpec: v0\ntitle: My Feature\napplications:\n    -\n---\n",
    );

    // Try to create a grouped spec with the same name
    tinyspec(&dir)
        .args(["new", "v1/my-feature"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

// ─── T.33: Check/uncheck on a grouped spec ──────────────────────────────────

#[test]
fn t33_check_grouped_spec() {
    let dir = TempDir::new().unwrap();
    let content = "\
---
tinySpec: v0
title: Grouped Tasks
applications:
    -
---

# Implementation Plan

- [ ] A: First task
- [ ] B: Second task

# Test Plan

";
    create_grouped_spec(&dir, "v2", "2025-03-01-10-00-grouped-tasks.md", content);

    // Check a task
    tinyspec(&dir)
        .args(["check", "grouped-tasks", "A"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Checked task A"));

    let updated = fs::read_to_string(
        dir.path()
            .join(".specs/v2/2025-03-01-10-00-grouped-tasks.md"),
    )
    .unwrap();
    assert!(updated.contains("- [x] A: First task"));
    assert!(updated.contains("- [ ] B: Second task"));

    // Uncheck the task
    tinyspec(&dir)
        .args(["uncheck", "grouped-tasks", "A"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Unchecked task A"));

    let updated = fs::read_to_string(
        dir.path()
            .join(".specs/v2/2025-03-01-10-00-grouped-tasks.md"),
    )
    .unwrap();
    assert!(updated.contains("- [ ] A: First task"));
}

// ── Dashboard ──────────────────────────────────────────────────────────────

#[test]
fn t34_dashboard_exits_with_error_without_tty() {
    let dir = TempDir::new().unwrap();
    create_sample_spec(&dir, "2025-02-17-09-00-my-spec.md", &sample_spec_content());

    // When run without a TTY (as in tests), the dashboard should exit with an error
    tinyspec(&dir)
        .arg("dashboard")
        .assert()
        .failure()
        .stderr(predicate::str::contains("interactive terminal"));
}

// ─── T.35: Templates command with no templates ───────────────────────────────

#[test]
fn t35_templates_command_no_templates() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(".specs")).unwrap();

    tinyspec(&dir)
        .args(["templates"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No templates found."));
}

// ─── T.36: Templates command lists repo templates ────────────────────────────

#[test]
fn t36_templates_command_lists_repo_templates() {
    let dir = TempDir::new().unwrap();
    let templates_dir = dir.path().join(".specs/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("default.md"),
        "---\ntinySpec: v0\ntitle: {{title}}\napplications:\n    -\n---\n\n# Background\n",
    )
    .unwrap();
    fs::write(
        templates_dir.join("rust-lib.md"),
        "---\ntinySpec: v0\ntitle: {{title}}\napplications:\n    -\n---\n\n# Background\n",
    )
    .unwrap();

    tinyspec(&dir)
        .args(["templates"])
        .assert()
        .success()
        .stdout(predicate::str::contains("default"))
        .stdout(predicate::str::contains("rust-lib"))
        .stdout(predicate::str::contains("(repo)"));
}

// ─── T.37: Templates directory is excluded from spec listing ─────────────────

#[test]
fn t37_templates_excluded_from_spec_listing() {
    let dir = TempDir::new().unwrap();

    // Create a regular spec
    create_sample_spec(
        &dir,
        "2025-01-01-10-00-alpha.md",
        "---\ntinySpec: v0\ntitle: Alpha\napplications:\n    -\n---\n\n# Background\n",
    );

    // Create a template (should NOT appear in spec listing)
    let templates_dir = dir.path().join(".specs/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("default.md"),
        "---\ntinySpec: v0\ntitle: Default Template\napplications:\n    -\n---\n\n# Background\n",
    )
    .unwrap();

    let output = tinyspec(&dir).args(["list"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("alpha"), "Should list regular spec");
    assert!(
        !stdout.contains("Default Template"),
        "Should NOT list template as a spec"
    );
    assert!(
        !stdout.contains("templates/"),
        "Should NOT show templates group header"
    );
}

// ─── T.38: New spec with --template flag ─────────────────────────────────────

#[test]
fn t38_new_with_template_flag() {
    let dir = TempDir::new().unwrap();
    let templates_dir = dir.path().join(".specs/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("rust-lib.md"),
        "\
---
tinySpec: v0
title: {{title}}
applications:
    -
---

# Background

This is a Rust library spec.

# Proposal



# Implementation Plan

- [ ] A: Set up project structure
- [ ] B: Implement core functionality

# Test Plan

",
    )
    .unwrap();

    tinyspec(&dir)
        .args(["new", "my-lib", "--template", "rust-lib"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created spec:"));

    // Find the created spec
    let specs = dir.path().join(".specs");
    let entries: Vec<_> = fs::read_dir(&specs)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    assert_eq!(entries.len(), 1);

    let content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        content.contains("title: My Lib"),
        "Title should be substituted"
    );
    assert!(
        content.contains("This is a Rust library spec."),
        "Template body should be preserved"
    );
    assert!(
        content.contains("- [ ] A: Set up project structure"),
        "Template tasks should be preserved"
    );
}

// ─── T.39: New spec auto-applies default template ────────────────────────────

#[test]
fn t39_new_auto_applies_default_template() {
    let dir = TempDir::new().unwrap();
    let templates_dir = dir.path().join(".specs/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("default.md"),
        "\
---
tinySpec: v0
title: {{title}}
applications:
    -
---

# Background

Default template background.

# Proposal



# Implementation Plan



# Test Plan

",
    )
    .unwrap();

    // No --template flag — should auto-apply "default"
    tinyspec(&dir)
        .args(["new", "auto-default"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created spec:"));

    let specs = dir.path().join(".specs");
    let entries: Vec<_> = fs::read_dir(&specs)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    assert_eq!(entries.len(), 1);

    let content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        content.contains("title: Auto Default"),
        "Title should be substituted"
    );
    assert!(
        content.contains("Default template background."),
        "Default template body should be used"
    );
}

// ─── T.40: New spec without default template uses built-in scaffold ──────────

#[test]
fn t40_new_without_default_template() {
    let dir = TempDir::new().unwrap();

    // No templates at all
    tinyspec(&dir)
        .args(["new", "vanilla-spec"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created spec:"));

    let specs = dir.path().join(".specs");
    let entries: Vec<_> = fs::read_dir(&specs)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 1);

    let content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(content.contains("title: Vanilla Spec"));
    assert!(content.contains("# Background"));
    assert!(content.contains("# Implementation Plan"));
}

// ─── T.41: Template substitutes {{date}} placeholder ─────────────────────────

#[test]
fn t41_template_substitutes_date() {
    let dir = TempDir::new().unwrap();
    let templates_dir = dir.path().join(".specs/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("dated.md"),
        "\
---
tinySpec: v0
title: {{title}}
applications:
    -
---

# Background

Created on {{date}}.

# Proposal



# Implementation Plan



# Test Plan

",
    )
    .unwrap();

    tinyspec(&dir)
        .args(["new", "date-test", "--template", "dated"])
        .assert()
        .success();

    let specs = dir.path().join(".specs");
    let entries: Vec<_> = fs::read_dir(&specs)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    let content = fs::read_to_string(entries[0].path()).unwrap();
    // Date should be a real date, not the placeholder
    assert!(
        !content.contains("{{date}}"),
        "{{date}} placeholder should be replaced"
    );
    // Should contain a date pattern like 2026-02-18
    assert!(
        content.contains("Created on 20"),
        "Should contain a date starting with 20xx"
    );
}

// ─── T.42: --template with nonexistent template fails ────────────────────────

#[test]
fn t42_template_not_found() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(".specs")).unwrap();

    tinyspec(&dir)
        .args(["new", "fail-spec", "--template", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No template found"));
}

// ─── T.43: Check/uncheck with emoji task group IDs ───────────────────────────

#[test]
fn t43_check_emoji_task_ids() {
    let dir = TempDir::new().unwrap();
    let content = "\
---
tinySpec: v0
title: Emoji Tasks
applications:
    -
---

# Implementation Plan

- [ ] \u{1F9EA}: Testing tasks
    - [ ] \u{1F9EA}.1: Write unit tests
    - [ ] \u{1F9EA}.2: Write integration tests
- [ ] \u{1F680}: Deployment tasks
    - [ ] \u{1F680}.1: Deploy to staging

# Test Plan

";
    create_sample_spec(&dir, "2025-03-01-10-00-emoji-tasks.md", content);

    // Check an emoji subtask
    tinyspec(&dir)
        .args(["check", "emoji-tasks", "\u{1F9EA}.1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Checked task \u{1F9EA}.1"));

    let updated =
        fs::read_to_string(dir.path().join(".specs/2025-03-01-10-00-emoji-tasks.md")).unwrap();
    assert!(
        updated.contains("- [x] \u{1F9EA}.1: Write unit tests"),
        "Emoji subtask should be checked"
    );
    assert!(
        updated.contains("- [ ] \u{1F9EA}.2: Write integration tests"),
        "Other emoji subtask should remain unchecked"
    );
    assert!(
        updated.contains("- [ ] \u{1F680}: Deployment tasks"),
        "Other emoji group should remain unchecked"
    );

    // Uncheck the emoji subtask
    tinyspec(&dir)
        .args(["uncheck", "emoji-tasks", "\u{1F9EA}.1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Unchecked task \u{1F9EA}.1"));

    let updated =
        fs::read_to_string(dir.path().join(".specs/2025-03-01-10-00-emoji-tasks.md")).unwrap();
    assert!(
        updated.contains("- [ ] \u{1F9EA}.1: Write unit tests"),
        "Emoji subtask should be unchecked"
    );
}

// ─── T.44: Status counts emoji tasks correctly ───────────────────────────────

#[test]
fn t44_status_with_emoji_tasks() {
    let dir = TempDir::new().unwrap();
    let content = "\
---
tinySpec: v0
title: Emoji Status
applications:
    -
---

# Implementation Plan

- [ ] \u{1F9EA}: Testing tasks
    - [x] \u{1F9EA}.1: Write unit tests
    - [ ] \u{1F9EA}.2: Write integration tests
- [x] \u{1F680}: Deploy

# Test Plan

";
    create_sample_spec(&dir, "2025-03-01-10-00-emoji-status.md", content);

    tinyspec(&dir)
        .args(["status", "emoji-status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2/4 tasks complete"));
}

// ─── T.45: Check emoji top-level group ID ────────────────────────────────────

#[test]
fn t45_check_emoji_top_level() {
    let dir = TempDir::new().unwrap();
    let content = "\
---
tinySpec: v0
title: Emoji Top Level
applications:
    -
---

# Implementation Plan

- [ ] \u{1F9EA}: Testing tasks
- [ ] \u{1F680}: Deploy

# Test Plan

";
    create_sample_spec(&dir, "2025-03-01-10-00-emoji-top.md", content);

    tinyspec(&dir)
        .args(["check", "emoji-top", "\u{1F680}"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Checked task \u{1F680}"));

    let updated =
        fs::read_to_string(dir.path().join(".specs/2025-03-01-10-00-emoji-top.md")).unwrap();
    assert!(updated.contains("- [x] \u{1F680}: Deploy"));
    assert!(updated.contains("- [ ] \u{1F9EA}: Testing tasks"));
}

// ─── T.46: Format preserves emoji task IDs ───────────────────────────────────

#[test]
fn t46_format_preserves_emoji_task_ids() {
    let dir = TempDir::new().unwrap();
    let content = "\
---
tinySpec: v0
title: Emoji Format
applications:
    -
---

# Implementation Plan

- [ ] \u{1F9EA}: Testing tasks
    - [ ] \u{1F9EA}.1: Write unit tests
    - [ ] \u{1F9EA}.2: Write integration tests
- [ ] \u{1F680}: Deploy

# Test Plan

";
    create_sample_spec(&dir, "2025-03-01-10-00-emoji-format.md", content);

    tinyspec(&dir)
        .args(["format", "emoji-format"])
        .assert()
        .success();

    let formatted =
        fs::read_to_string(dir.path().join(".specs/2025-03-01-10-00-emoji-format.md")).unwrap();
    assert!(
        formatted.contains("\u{1F9EA}"),
        "Emoji should be preserved after formatting"
    );
    assert!(
        formatted.contains("\u{1F9EA}.1: Write unit tests"),
        "Emoji subtask IDs should be preserved"
    );
    assert!(
        formatted.contains("\u{1F680}: Deploy"),
        "Other emoji IDs should be preserved"
    );

    // Formatting should be idempotent
    tinyspec(&dir)
        .args(["format", "emoji-format"])
        .assert()
        .success();
    let second =
        fs::read_to_string(dir.path().join(".specs/2025-03-01-10-00-emoji-format.md")).unwrap();
    assert_eq!(
        formatted, second,
        "Formatter should be idempotent with emoji"
    );
}
