---
tinySpec: v0
title: Skill Improvements (decisions log, dry-run, focus context)
---

# Background

The current skills are functional but have three friction points that emerge as usage scales:

**Lost refinement context.** During `/tinyspec:refine`, Claude asks clarifying questions and the user provides answers. Those answers shape the implementation plan — but they're never written down anywhere. After the conversation ends, the rationale behind specific task choices is gone. Revisiting a spec weeks later, it's unclear why tasks were scoped the way they were.

**No safe preview for `tinyspec-do`.** Before committing an entire implementation plan to Claude for autonomous execution, there's no way to preview what will happen — which repos will be touched, which tasks are blocked, whether dependencies are met. Users have to either trust the spec is right or read through it manually.

**No active spec context.** Every skill invocation requires naming the spec explicitly (e.g., `/tinyspec:task my-feature-name A.3`). In a session where one spec is the clear focus, this repetition is friction. There's also no way for one skill to pick up where another left off without restating the spec name.

# Proposal

**Decisions log in `tinyspec-refine`**
After completing a refinement session, append a `# Decisions` section to the spec containing a structured log of the key Q&A from the session. Each entry records the question asked, the chosen answer, and the reasoning if the user provided it. This section is not parsed for task tracking — it's informational documentation preserved for future readers (human or AI).

Format:

```markdown
# Decisions

- **Scope:** Limit to read-only API endpoints in v1.
  *Reason: Write paths require auth changes scoped to a separate spec.*
- **Storage:** Use existing PostgreSQL schema, no new tables.
  *Reason: Simplest approach; can migrate to dedicated table if needed.*
```

`tinyspec-do` and `tinyspec-task` should read this section before starting implementation to understand the design constraints.

**`tinyspec-do --dry-run` (or `/tinyspec:do --dry-run`)**
Before executing, emit a summary of what will happen:

- List all incomplete tasks in order
- Identify which applications/repos will be touched
- Flag any blocked tasks (depends_on not met) or missing application configs
- Show estimated task count and group breakdown

No files are modified. Output is printed and the skill exits. This gives users a chance to verify the plan before committing.

**`tinyspec focus <spec-name>` / `tinyspec unfocus`**
Write the focused spec name to a `.tinyspec-focus` file at the project root. Skills read this file when no spec name is provided as an argument — enabling `/tinyspec:do` with no arguments to operate on the focused spec. `tinyspec focus` with no argument prints the currently focused spec. `tinyspec unfocus` deletes the `.tinyspec-focus` file.

The focused spec name is also displayed in `tinyspec list` output (e.g., with a `→` marker) and in the dashboard header.

# Implementation Plan

- [ ] A: Implement the `# Decisions` section in `tinyspec-refine`
  
  - [ ] A.1: Update the `tinyspec-refine` skill to collect Q&A decisions during the session
  - [ ] A.2: After the user approves the implementation plan, append a `# Decisions` section to the spec with structured entries
  - [ ] A.3: Update `tinyspec-do` and `tinyspec-task` skills to read `# Decisions` before starting implementation
- [ ] B: Implement dry-run mode in `tinyspec-do`
  
  - [ ] B.1: Add `--dry-run` argument parsing to the `tinyspec-do` skill
  - [ ] B.2: In dry-run mode: load the spec, list all incomplete tasks with group breakdowns, identify repos to be touched, flag any blocked or misconfigured tasks
  - [ ] B.3: Print the dry-run report and exit without modifying any files
- [ ] C: Implement `tinyspec focus` and `tinyspec unfocus`
  
  - [ ] C.1: Add `focus` and `unfocus` subcommands to the CLI
  - [ ] C.2: `focus <spec-name>`: resolve the spec name (validate it exists), write to `.tinyspec-focus`
  - [ ] C.3: `focus` with no argument: print the currently focused spec name (or "no spec focused")
  - [ ] C.4: `unfocus`: delete `.tinyspec-focus` if present
  - [ ] C.5: Add `.tinyspec-focus` to `.gitignore` (it's a local session artifact)
- [ ] D: Update skills to read `.tinyspec-focus` as default spec
  
  - [ ] D.1: In `tinyspec-do`, `tinyspec-task`, `tinyspec-refine`: if no spec name argument is given, check for `.tinyspec-focus` and use it
  - [ ] D.2: If `.tinyspec-focus` is missing and no argument given, prompt the user to specify a spec or run `tinyspec focus`
- [ ] E: Surface focus state in list and dashboard
  
  - [ ] E.1: Mark the focused spec with a `→` indicator in `tinyspec list` output
  - [ ] E.2: Show focused spec name in the dashboard header line

# Test Plan

- [ ] T.1: After running `/tinyspec:refine`, verify a `# Decisions` section is appended with at least one entry
- [ ] T.2: `tinyspec-do` reads the `# Decisions` section without erroring on specs that don't have one
- [ ] T.3: `/tinyspec:do --dry-run` prints a task list and repo summary without modifying any files
- [ ] T.4: `tinyspec focus my-spec` writes `my-spec` to `.tinyspec-focus`
- [ ] T.5: `tinyspec focus` with no argument prints the currently focused spec
- [ ] T.6: `tinyspec unfocus` removes `.tinyspec-focus`
- [ ] T.7: Running `/tinyspec:do` with no arguments on a focused spec starts the correct spec
- [ ] T.8: `tinyspec list` shows `→` marker next to the focused spec
