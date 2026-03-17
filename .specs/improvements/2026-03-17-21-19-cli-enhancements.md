---
tinySpec: v0
title: CLI Enhancements (search, json, archive, lint)
---

# Background

As a spec library grows, several gaps in the CLI become friction:

- **No search:** Finding a spec requires knowing its name or scrolling `tinyspec list`. There's no way to search by title keyword or body content.
- **No machine-readable output:** All commands produce human-formatted text. This blocks scripting, CI pipelines, and external dashboards from consuming tinyspec data without brittle text parsing.
- **No archival:** Completed specs pile up in the active list indefinitely. There's no way to move them out of view without deleting them.
- **No linting:** Specs can have missing sections, non-sequential task IDs, ungrouped task hierarchies, or unconfigured `applications` — and there's no command to catch these issues before handing a spec to Claude.

# Proposal

Add four focused CLI commands:

**`tinyspec search <query>`**
Full-text search across spec titles and body content. Matches are printed with the spec name, title, and a snippet of the matching line. Supports `--group <name>` to narrow to a folder and `--status <pending|in-progress|completed>` to filter by state.

**`--json` flag on `list`, `status`, `view`**
Output structured JSON for scripting. `tinyspec list --json` returns an array of spec objects with all `SpecSummary` fields. `tinyspec status <spec> --json` returns the full task tree with checked state. `tinyspec view <spec> --json` returns front matter fields and raw body sections.

**`tinyspec archive [<spec-name>|--all-completed]`**
Move a spec file to `.specs/archive/`. Archived specs are excluded from `tinyspec list`, `tinyspec status`, and the dashboard by default. A `--include-archived` flag on `list` and `status` restores them. `tinyspec unarchive <spec-name>` moves a spec back to its original group.

**`tinyspec lint [<spec-name>|--all]`**
Validate spec health and print warnings/errors:

- Missing required sections (`# Background`, `# Proposal`, `# Implementation Plan`)
- Empty sections (section heading present but no content)
- Non-sequential task IDs (e.g., jumps from A.2 to A.4)
- `applications` entries that aren't configured in `~/.tinyspec/config.yaml`
- Spec has no tasks at all

Exit code 0 if clean, non-zero if errors found (warnings don't affect exit code).

# Implementation Plan

- [x] A: Implement `tinyspec search`
  
  - [x] A.1: Add `search` subcommand to `main.rs` with `query`, `--group`, and `--status` options
  - [x] A.2: Walk all spec files, load title from front matter and full body text
  - [x] A.3: Case-insensitive substring match against title and body; collect matching lines with context
  - [x] A.4: Print results grouped by spec name with match snippets
- [x] B: Implement `--json` output flag
  
  - [x] B.1: Add `--json` flag to `list`, `status`, and `view` subcommands
  - [x] B.2: Derive or implement `serde::Serialize` on `SpecSummary`, `TaskNode`, and front matter types
  - [x] B.3: Serialize and print JSON to stdout when flag is present; suppress all other output
- [x] C: Implement `tinyspec archive` and `tinyspec unarchive`
  
  - [x] C.1: Add `archive` and `unarchive` subcommands
  - [x] C.2: `archive`: move spec file to `.specs/archive/`, preserving group subdirectory structure
  - [x] C.3: `unarchive`: move spec file back to its original group folder
  - [x] C.4: Exclude `.specs/archive/` from all spec discovery by default
  - [x] C.5: Add `--include-archived` flag to `list`, `status`, `dashboard`
- [x] D: Implement `tinyspec lint`
  
  - [x] D.1: Add `lint` subcommand accepting optional spec name or `--all`
  - [x] D.2: Implement checks: required sections, empty sections, sequential task IDs, configured applications
  - [x] D.3: Categorize findings as `error` or `warning`; print with spec name and line reference where possible
  - [x] D.4: Exit non-zero on any errors; exit 0 if only warnings or clean
- [x] E: Shell completion and documentation
  
  - [x] E.1: Extend shell completion to cover new subcommands and flags
  - [x] E.2: Update `CLAUDE.md` with new command descriptions

# Test Plan

- [x] T.1: `tinyspec search foo` returns specs whose title or body contains "foo"
- [x] T.2: `tinyspec search foo --status completed` returns only completed matches
- [x] T.3: `tinyspec list --json` returns valid JSON array with expected fields for all specs
- [x] T.4: `tinyspec status <spec> --json` returns full task tree with `checked` booleans
- [x] T.5: `tinyspec archive <spec>` moves the file to `.specs/archive/`; spec no longer appears in `tinyspec list`
- [x] T.6: `tinyspec unarchive <spec>` moves it back; spec reappears in `tinyspec list`
- [x] T.7: `tinyspec lint` on a spec missing `# Proposal` exits non-zero with a clear error message
- [x] T.8: `tinyspec lint` on a valid spec exits 0
- [x] T.9: `tinyspec lint --all` reports issues across all specs
