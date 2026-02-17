---
tinySpec: v0
title: Feature Groups
---

# Background

All specs live flat in `.specs/`. As a project grows, the list becomes unwieldy. There's no way to organize related specs (e.g., by version, component, or milestone).

# Proposal

Support single-level subdirectories within `.specs/` as groups. Usage: `tinyspec new v1/feature` creates `.specs/v1/2025-02-17-09-36-feature.md`. Spec names remain globally unique — the group is purely organizational. All existing commands (`view`, `check`, `edit`, etc.) continue to accept just the spec name since there's no ambiguity.

Key design decisions:

- Slash prefix syntax: `group/name`
- `tinyspec list` shows grouped headers
- Single nesting level only
- Globally unique names (reject duplicates across groups)
- Ungrouped specs still work as before

# Implementation Plan

- [x] A: Update validation to support group/name syntax
- [x] A.1: Add a `parse_spec_input` function that splits `group/name` and validates both parts are kebab-case (or just `name` with no group)
- [x] B: Update spec discovery to search subdirectories
- [x] B.1: Modify `find_spec` to walk `.specs/` and one level of subdirectories
- [x] B.2: Modify `complete_spec_names` to include specs from subdirectories
- [x] C: Update `new_spec` to support grouped creation
- [x] C.1: Parse `group/name`, create `.specs/group/` subdirectory if needed
- [x] C.2: Enforce global uniqueness — reject if the spec name already exists in any group
- [x] D: Update `list` to show grouped output
- [x] D.1: Collect specs from all directories, grouped by parent folder
- [x] D.2: Display with group headers (ungrouped specs first, then each group)
- [x] E: Update bulk operations to include subdirectories
- [x] E.1: Update `format_all_specs` to process specs in subdirectories
- [x] E.2: Update `status` (no args) to include specs from subdirectories
- [x] F: Add integration tests
- [x] F.1: Test creating a grouped spec (`tinyspec new v1/feature`)
- [x] F.2: Test finding/viewing a grouped spec by name alone
- [x] F.3: Test `list` output with grouped and ungrouped specs
- [x] F.4: Test that duplicate names across groups are rejected
- [x] F.5: Test `check`/`uncheck` on a grouped spec

# Test Plan
