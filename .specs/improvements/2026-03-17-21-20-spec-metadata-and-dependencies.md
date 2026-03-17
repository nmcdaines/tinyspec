---
tinySpec: v0
title: Spec Metadata and Dependency Tracking
---

# Background

Spec front matter currently supports only `title` and `applications`. In practice, specs have implicit attributes — priority, tags, and relationships to other specs — that are managed mentally or in external tools. This creates several problems:

- When multiple specs are pending, there's no way to know which to tackle first. `tinyspec-oneshot` executes specs in an arbitrary order.
- A spec may build on work defined in another spec. If the upstream spec isn't done, executing the downstream one will fail or produce broken results. Currently nothing warns about this.
- There's no way to filter `tinyspec list` by theme (e.g., "show me all auth-related specs").

# Proposal

Extend the front matter schema with three optional fields:

```yaml
priority: high         # high | medium | low (default: medium)
tags: [auth, api]      # arbitrary string labels
depends_on:            # spec names that must be completed first
  - other-spec-name
```

**Priority** influences the default ordering in `tinyspec list` and `tinyspec status` output (high specs appear first within their status group). The `tinyspec-oneshot` skill uses priority to determine execution order.

**Tags** enable filtering: `tinyspec list --tag auth` shows only tagged specs. Tags are displayed in `tinyspec view` output.

**`depends_on`** is the most behaviorally significant field. It lists spec names (not file names) that must reach `Completed` status before this spec is considered ready. Behavior:

- `tinyspec status <spec>` shows a `BLOCKED` indicator if any dependency is not `Completed`
- `tinyspec-do` and `tinyspec-task` warn and ask for confirmation before starting a blocked spec
- `tinyspec-oneshot` skips blocked specs and re-queues them after their dependencies complete
- `tinyspec lint` reports a warning if a `depends_on` entry doesn't match any known spec name

Circular dependency detection is required — `tinyspec lint` and spec creation should catch cycles.

# Implementation Plan

- [ ] A: Extend front matter parsing
  
  - [ ] A.1: Add `priority: Option<Priority>`, `tags: Vec<String>`, and `depends_on: Vec<String>` to the `FrontMatter` struct
  - [ ] A.2: Update YAML deserialization to parse the new fields; unknown values for `priority` should produce a clear error
  - [ ] A.3: Add `Priority` enum with `High`, `Medium` (default), `Low` variants and serde support
- [ ] B: Surface metadata in list and status output
  
  - [ ] B.1: Display priority indicator (e.g., `[H]`, `[M]`, `[L]`) in `tinyspec list` and `tinyspec status` output
  - [ ] B.2: Sort specs within each status group by priority (High → Medium → Low)
  - [ ] B.3: Add `--tag <tag>` filter to `tinyspec list` and `tinyspec status`
  - [ ] B.4: Show `BLOCKED` indicator in `tinyspec status` when `depends_on` specs are incomplete
- [ ] C: Dependency validation and cycle detection
  
  - [ ] C.1: Implement dependency resolution: given a spec, load all its `depends_on` specs and check their status
  - [ ] C.2: Implement topological sort with cycle detection; return an error on circular dependencies
  - [ ] C.3: Add dependency checks to `tinyspec lint`
- [ ] D: Update skills to respect dependencies and priority
  
  - [ ] D.1: `tinyspec-do`: before starting, check if spec is blocked; warn and ask for confirmation if so
  - [ ] D.2: `tinyspec-oneshot`: sort pending specs by dependency order then priority before executing
  - [ ] D.3: After completing a spec, `tinyspec-oneshot` should re-evaluate which previously blocked specs are now unblocked
- [ ] E: Update templates and documentation
  
  - [ ] E.1: Add commented-out `priority`, `tags`, and `depends_on` examples to the default spec template
  - [ ] E.2: Update `CLAUDE.md` with new front matter field descriptions

# Test Plan

- [ ] T.1: Parse front matter with all three new fields; verify values are correctly deserialized
- [ ] T.2: `tinyspec list` sorts high-priority specs before medium before low within the same status group
- [ ] T.3: `tinyspec list --tag auth` returns only specs tagged with "auth"
- [ ] T.4: A spec with `depends_on` pointing to an incomplete spec shows `BLOCKED` in `tinyspec status`
- [ ] T.5: A spec with `depends_on` pointing to a completed spec does not show `BLOCKED`
- [ ] T.6: `tinyspec lint` reports a warning for a `depends_on` entry that doesn't match any spec
- [ ] T.7: `tinyspec lint` reports an error for circular dependencies
- [ ] T.8: `tinyspec-oneshot` executes specs in dependency-respecting order
