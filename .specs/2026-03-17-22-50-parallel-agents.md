---
tinySpec: v0
title: Parallel Agents
---

# Background

`/tinyspec:do` and `/tinyspec:oneshot` currently work sequentially: one task group at a time, one spec at a time. For specs with independent task groups, or projects with multiple unrelated pending specs, this leaves parallelism on the table.

Claude Code's Agent tool supports background agents, making it possible to implement independent units of work concurrently. The challenge is knowing what's safe to parallelise — task groups within a spec often depend on each other (B builds on A), but not always.

# Proposal

Add a `--parallel` flag to both `/tinyspec:do` and `/tinyspec:oneshot`. When the flag is present:

- **`/tinyspec:do --parallel`**: Before starting work, Claude reads all task groups in the spec's Implementation Plan and infers which groups are independent (no shared code paths, no build-on-each-other relationships). Independent groups are launched as concurrent background agents; groups with dependencies run sequentially after their prerequisites complete.

- **`/tinyspec:oneshot --parallel`**: Before starting work, Claude reads all pending specs and infers which are independent (touch different parts of the codebase, no shared concerns). Independent specs are launched as concurrent background agents, each running `/tinyspec:do` on its spec.

Dependency inference is done by LLM reasoning over task descriptions — no explicit markers in spec files. Concurrent `tinyspec check` calls writing to the same file are treated as acceptable risk given the speed of the operations.

# Implementation Plan

- [x] A: Update `/tinyspec:do` skill to support `--parallel`
  - [x] A.1: Add `--parallel` flag detection to the skill's argument parsing section
  - [x] A.2: Add a pre-work dependency analysis step: read all task groups, reason about which are independent, produce an ordered execution plan
  - [x] A.3: When `--parallel` is set and independent groups exist, spawn them as background agents using the Agent tool; await completion before starting any dependent groups
  - [x] A.4: Update skill docs/comments to describe the parallel mode behaviour
- [x] B: Update `/tinyspec:oneshot` skill to support `--parallel`
  - [x] B.1: Add `--parallel` flag detection to the skill's argument parsing section
  - [x] B.2: Add a pre-work dependency analysis step: read all pending specs, reason about which are independent across the codebase
  - [x] B.3: When `--parallel` is set, spawn independent specs as background agents (each running `/tinyspec:do` on its spec); run dependent specs sequentially
  - [x] B.4: Update skill docs/comments to describe the parallel mode behaviour
- [x] C: Update embedded skill source files
  - [x] C.1: Mirror A's changes to `src/skills/tinyspec-do.md`
  - [x] C.2: Mirror B's changes to `src/skills/tinyspec-oneshot.md`
- [ ] 🧪 Run `cargo test` upon completion

# Test Plan

- [x] T.1: `/tinyspec:do my-spec --parallel` with a spec whose groups are clearly independent — verify multiple background agents are spawned
- [x] T.2: `/tinyspec:do my-spec --parallel` with a spec whose groups are clearly sequential (B explicitly builds on A) — verify agents run in order, not in parallel
- [x] T.3: `/tinyspec:do my-spec` (no flag) — verify sequential behaviour is unchanged
- [x] T.4: `/tinyspec:oneshot --parallel` with multiple unrelated pending specs — verify specs run as concurrent background agents
- [x] T.5: `/tinyspec:oneshot --parallel` with specs that touch the same modules — verify they are not parallelised
- [x] T.6: `/tinyspec:oneshot` (no flag) — verify sequential behaviour is unchanged
