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

    tinyspec(&dir)
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
    create_sample_spec(&dir, "2025-02-17-09-36-hello-world.md", &sample_spec_content());

    // Confirm deletion by piping "y" to stdin
    tinyspec(&dir)
        .args(["delete", "hello-world"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted"));

    assert!(!dir
        .path()
        .join(".specs/2025-02-17-09-36-hello-world.md")
        .exists());
}

// ─── T.7: Check a task ──────────────────────────────────────────────────────

#[test]
fn t7_check_task() {
    let dir = TempDir::new().unwrap();
    create_sample_spec(&dir, "2025-02-17-09-36-hello-world.md", &sample_spec_content());

    tinyspec(&dir)
        .args(["check", "hello-world", "A"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Checked task A"));

    let content = fs::read_to_string(
        dir.path().join(".specs/2025-02-17-09-36-hello-world.md"),
    )
    .unwrap();
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

    let content = fs::read_to_string(
        dir.path().join(".specs/2025-02-17-09-36-hello-world.md"),
    )
    .unwrap();
    assert!(content.contains("- [ ] B: Do that"));
}

// ─── T.9: Check with invalid task ID ────────────────────────────────────────

#[test]
fn t9_check_invalid_task_id() {
    let dir = TempDir::new().unwrap();
    create_sample_spec(&dir, "2025-02-17-09-36-hello-world.md", &sample_spec_content());

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
        .stdout(predicate::str::contains("Created spec-refine.md"))
        .stdout(predicate::str::contains("Created spec-work.md"))
        .stdout(predicate::str::contains("Created spec-task.md"));

    let commands_dir = dir.path().join(".claude/commands");
    assert!(commands_dir.join("spec-refine.md").exists());
    assert!(commands_dir.join("spec-work.md").exists());
    assert!(commands_dir.join("spec-task.md").exists());

    // Verify skill files have content
    let refine = fs::read_to_string(commands_dir.join("spec-refine.md")).unwrap();
    assert!(refine.contains("$ARGUMENTS"));
}

// ─── T.12: Init does not overwrite existing skill files ─────────────────────

#[test]
fn t12_init_no_overwrite() {
    let dir = TempDir::new().unwrap();
    let commands_dir = dir.path().join(".claude/commands");
    fs::create_dir_all(&commands_dir).unwrap();
    fs::write(commands_dir.join("spec-refine.md"), "custom content").unwrap();

    tinyspec(&dir)
        .args(["init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Skipped spec-refine.md"))
        .stdout(predicate::str::contains("Created spec-work.md"))
        .stdout(predicate::str::contains("Created spec-task.md"));

    // Verify custom content preserved
    let content = fs::read_to_string(commands_dir.join("spec-refine.md")).unwrap();
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
