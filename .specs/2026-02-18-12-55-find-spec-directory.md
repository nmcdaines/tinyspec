---
tinySpec: v0
title: Find Spec Directory
---

# Background

The tinyspec CLI currently resolves the `.specs/` directory as a relative path from the current working directory. This means all commands (`list`, `view`, `new`, etc.) must be run from the project root where `.specs/` lives. If you're in a subdirectory of a project, commands silently fail to find specs or create `.specs/` in the wrong location.

This is a common friction point — tools like `git` and `cargo` solve this by walking up the directory tree to find their root marker (`.git/`, `Cargo.toml`). tinyspec should do the same with `.specs/`.

# Proposal

Modify the `specs_dir()` function to walk up parent directories from the current working directory, looking for a `.specs/` directory. If found, return its path. If not found, fall back to `.specs` relative to cwd (current behavior).

For `tinyspec new` (when no `.specs/` directory exists yet), attempt to detect the git repository root via the presence of a `.git` directory (walking upward) and create `.specs/` there. If not in a git repo, create `.specs/` in the current directory.

# Implementation Plan

- [x] A: Directory discovery
  - [x] A.1: Add a `discover_specs_dir()` function in `src/spec/mod.rs` that walks up from cwd looking for a `.specs/` directory, returning `Option<PathBuf>`
  - [x] A.2: Add a `discover_git_root()` function in `src/spec/mod.rs` that walks up from cwd looking for a `.git/` directory, returning `Option<PathBuf>`
  - [x] A.3: Update `specs_dir()` to call `discover_specs_dir()`, falling back to `PathBuf::from(".specs")` if not found
- [x] B: New command integration
  - [x] B.1: Update `new_spec()` in `src/spec/commands.rs` to use `discover_git_root()` when `.specs/` doesn't exist yet, creating it at the git root instead of cwd
- [x] C: Tests
  - [x] C.1: Test discovery of `.specs/` from a subdirectory
  - [x] C.2: Test fallback to cwd when no `.specs/` exists above
  - [x] C.3: Test `tinyspec new` creates `.specs/` at git root when in a git repo subdirectory
  - [x] C.4: Test `tinyspec new` creates `.specs/` in cwd when not in a git repo
- [ ] 🧪 Run `cargo test` upon completion

# Test Plan

- T.1: Given a project with `.specs/` at root, when running `tinyspec list` from a subdirectory, then it finds and lists specs from the root `.specs/`
- T.2: Given no `.specs/` directory in any parent, when running `tinyspec list`, then it returns empty (falls back to cwd)
- T.3: Given a git repo without `.specs/`, when running `tinyspec new my-spec` from a subdirectory, then `.specs/` is created at the git root
- T.4: Given a non-git directory without `.specs/`, when running `tinyspec new my-spec`, then `.specs/` is created in the current directory
