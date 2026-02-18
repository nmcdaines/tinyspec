---
tinySpec: v0
title: Explore Multi Repo
---

# Background

Tinyspec supports polyrepo workflows via an `applications` frontmatter field that maps repository names to local folder paths (resolved via `~/.tinyspec/config.yaml`). However, the current skill prompts do not instruct the agent to explore referenced repositories before making decisions. The refine skill has no multi-repo awareness at all, and the do/task skills only ask the user which repo to focus on rather than encouraging cross-repo exploration.

# Proposal

Update the three skill prompt templates (`tinyspec-refine`, `tinyspec-do`, `tinyspec-task`) to add a multi-repo exploration step. When a spec references applications, the agent should:

1. Read the spec via `tinyspec view` to resolve application paths (already done for do/task, needs adding to refine).
1. Explore the directory structure and key source files of **each** referenced repository before proposing plans or implementing changes.
1. Consider the architecture and patterns across all repos when designing the implementation plan.

When no `applications` field is present (or it's empty), fall back to exploring only the current repo — this is the existing default behavior.

Changes are prompt-only — no Rust code modifications are needed. Both the source templates in `src/skills/` and the installed copies in `.claude/skills/` must be updated.

# Implementation Plan

- [x] A: Update tinyspec-refine skill prompt
  - [x] A.1: Add step to read spec via `tinyspec view` to resolve application paths
  - [x] A.2: Add multi-repo exploration step — instruct agent to explore directory trees and key files in each referenced repo before proposing implementation plans
  - [x] A.3: Add fallback instruction — when no applications are defined, explore only the current directory
  - [x] A.4: Update both `src/skills/tinyspec-refine.md` and `.claude/skills/tinyspec-refine/SKILL.md`
- [ ] B: Update tinyspec-do skill prompt
  - [ ] B.1: Replace "ask which repo to focus on" with an active exploration step before beginning work
  - [ ] B.2: Instruct agent to explore directory trees and key files of all referenced repos after reading the spec
  - [ ] B.3: Update both `src/skills/tinyspec-do.md` and `.claude/skills/tinyspec-do/SKILL.md`
- [ ] C: Update tinyspec-task skill prompt
  - [ ] C.1: Replace "ask which repo to focus on" with an active exploration step before implementing the task
  - [ ] C.2: Instruct agent to explore relevant parts of all referenced repos to understand context
  - [ ] C.3: Update both `src/skills/tinyspec-task.md` and `.claude/skills/tinyspec-task/SKILL.md`
- [ ] 🧪 Run `cargo test` upon completion

# Test Plan
