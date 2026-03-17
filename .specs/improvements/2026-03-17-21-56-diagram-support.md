---
tinySpec: v0
title: Diagram Support in Specs
---

# Background

Specs are pure prose and task lists today. This works well for straightforward features, but some ideas are genuinely hard to communicate in text alone — system boundaries, data flow, state machines, entity relationships, and sequence interactions between components all benefit from a visual. Without diagrams, authors either write long paragraphs that are hard to scan, or omit the structural explanation entirely, leaving implementers to infer architecture from the task list.

Markdown already has a widely supported solution: Mermaid code blocks (```` ```mermaid ````). Mermaid is rendered natively by GitHub, GitLab, Obsidian, and most modern markdown viewers. It's text-based, so it diffs cleanly, lives alongside prose, and can be authored by Claude without any external tooling.

The challenge is not rendering — that's handled by the viewer. The challenge is knowing *when* a diagram would help, *what kind* to use, and ensuring the tinyspec formatter doesn't mangle fenced code blocks that contain diagrams.

# Proposal

Add diagram authoring as a capability across tinyspec's skill layer, with light tooling support in the CLI.

**Mermaid as the standard format**

All diagrams are authored as Mermaid code blocks. Claude chooses the diagram type based on what's being explained:

|Diagram type|When to use|
|------------|-----------|
|`flowchart`|Decision logic, data pipelines, process flow|
|`sequenceDiagram`|Request/response flows, inter-service calls, API interactions|
|`stateDiagram-v2`|State machines, spec lifecycle, task status transitions|
|`erDiagram`|Data models, schema relationships|
|`graph`|Dependency graphs, component maps|

Diagrams are embedded inline in spec sections — most naturally in `# Background` (to show current state or problem context) or `# Proposal` (to show the intended design). They're placed immediately after the prose they illustrate, not in a separate section.

**Skill integration**

During `tinyspec-refine` and `tinyspec-chat`, Claude proactively identifies moments where a diagram would clarify the explanation. It doesn't ask permission — it just includes a Mermaid block when the concept warrants it, the same way a good technical writer would include a figure without asking. If the user doesn't want it, they can remove it.

Specific triggers:

- The proposal involves more than two components interacting → sequence or flow diagram
- There's a described state machine or lifecycle → state diagram
- The background describes a data schema → ER diagram
- The implementation plan has a dependency graph among task groups → graph diagram

**`tinyspec diagram <spec-name>` command**

A standalone CLI command that reads a spec and asks Claude to suggest diagram additions. Claude analyzes the prose, identifies sections that would benefit from visualization, and proposes Mermaid blocks with a brief rationale for each. The user confirms which to add, then Claude writes them into the spec and runs `tinyspec format`.

This command is useful for retrofitting diagrams onto existing specs that were written without them.

**Formatter safety**

The existing `tinyspec format` command uses `pulldown-cmark` to normalize Markdown. Fenced code blocks (including Mermaid blocks) must pass through untouched — no whitespace normalization, no content changes. This is likely already the case since `pulldown-cmark` treats code blocks as opaque, but it needs explicit test coverage.

# Implementation Plan

- [x] A: Verify and harden formatter behavior with Mermaid blocks
  
  - [x] A.1: Write integration tests that pass specs containing Mermaid fenced blocks through `tinyspec format` and assert the block content is byte-for-byte identical after formatting
  - [x] A.2: Test with each Mermaid diagram type (`flowchart`, `sequenceDiagram`, `stateDiagram-v2`, `erDiagram`, `graph`) to ensure none are mangled
  - [x] A.3: Fix any formatter issues discovered (whitespace stripping inside code blocks, fence normalization, etc.)
- [ ] B: Update skills to proactively include diagrams
  
  - [ ] B.1: Update `tinyspec-refine` prompt: instruct Claude to identify structural concepts during refinement and include a Mermaid block when a diagram would reduce ambiguity
  - [ ] B.2: Update `tinyspec-chat` prompt similarly — when the conversation converges on an architecture or flow, include a diagram in the written spec output
  - [ ] B.3: Update `tinyspec-do` prompt: when reading a spec before implementation, treat Mermaid blocks as authoritative design documentation (not decorative)
  - [ ] B.4: Add guidance on diagram type selection to all relevant skills (the table from the Proposal section)
- [ ] C: Implement `tinyspec diagram <spec-name>` command
  
  - [ ] C.1: Add `diagram` subcommand to the CLI; it is a skill-backed command (shells out to Claude), not a pure Rust implementation
  - [ ] C.2: The skill reads the spec with `tinyspec view`, analyzes prose sections for visualization opportunities, and proposes Mermaid blocks with rationale
  - [ ] C.3: Present proposals to the user with `AskUserQuestion` — each proposed diagram is shown with its Mermaid source and a yes/no choice
  - [ ] C.4: Write accepted diagrams into the spec at the appropriate location (after the relevant paragraph)
  - [ ] C.5: Run `tinyspec format <spec-name>` after writing
- [ ] D: Update templates and documentation
  
  - [ ] D.1: Update the default spec template to include a commented example of a Mermaid block in the Proposal section
  - [ ] D.2: Update `CLAUDE.md` to document the diagram convention and the `tinyspec diagram` command

# Test Plan

- [ ] T.1: A spec containing a `flowchart` Mermaid block passes through `tinyspec format` with the diagram content unchanged
- [ ] T.2: A spec containing a `sequenceDiagram` block passes through `tinyspec format` unchanged
- [ ] T.3: `tinyspec diagram <spec>` on a spec with multi-component interactions proposes at least one sequence or flow diagram
- [ ] T.4: Accepting a proposed diagram writes the Mermaid block into the correct location in the spec (after the relevant paragraph, not at the end of the file)
- [ ] T.5: Declining all proposed diagrams leaves the spec file unchanged
- [ ] T.6: `tinyspec-refine` includes a Mermaid diagram in the output spec when the proposal involves inter-service interactions
- [ ] T.7: `tinyspec-do` does not strip or skip Mermaid blocks when reading a spec for context
