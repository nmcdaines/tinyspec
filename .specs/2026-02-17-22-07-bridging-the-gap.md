---
tinySpec: v0
title: Bridging The Gap
applications:
    -
---

# Background

OpenSpec (by Fission-AI) is a popular spec-driven development framework with 24k+ GitHub stars. It targets brownfield codebases and works across 20+ AI tools. Its core innovation is treating specs as a **living source of truth** — not just planning artifacts, but an evolving record of how the system actually works. It enforces a lifecycle: Proposal → Design → Apply → Archive, where completed changes get merged back into the main spec library.

Tinyspec is intentionally minimal — a single Markdown file per feature, four sections, a CLI, and Claude Code skills. That simplicity is a strength: low ceremony, fast onboarding, zero config. But there are meaningful gaps where OpenSpec's ideas would make tinyspec significantly more useful without compromising its "tiny" philosophy.

This spec identifies the highest-impact gaps and proposes targeted additions that stay true to tinyspec's design principles: Markdown-first, minimal ceremony, and CLI-driven.

# Proposal

After comparing the two frameworks, the following gaps represent the biggest opportunities. They are ordered by impact and feasibility, not by implementation order.

## 1. Global Project Context (`project.md`)

**The gap:** Every tinyspec spec has its own Background section, but there is no shared project-level context. This means every spec must re-explain the tech stack, architecture, and conventions — or assume the AI already knows them.

**The proposal:** Support an optional `.specs/project.md` file (freeform Markdown, no front matter required). When present, `tinyspec view` prepends its contents before the spec body (separated by a clear delimiter). The refine/work/task skills would automatically pick this up through `tinyspec view`. No new commands needed — just create the file manually.

## 2. Spec Validation (`tinyspec validate`)

**The gap:** `tinyspec format` normalizes Markdown but doesn't verify that a spec is structurally complete. A spec with an empty Implementation Plan or missing Background section looks identical to a complete one.

**The proposal:** Add a `tinyspec validate <spec-name>` command (and `--all` flag) that checks:

- All four required sections (Background, Proposal, Implementation Plan, Test Plan) exist as H1 headings
- Background and Proposal sections are non-empty
- Implementation Plan tasks follow the expected format (`- [ ] ID: Description`)
- Task IDs are sequential and properly nested (A before A.1, etc.)
- No duplicate task IDs

Output a clear pass/fail with specific line numbers for issues. Exit code 0 on success, 1 on failure (for CI use).

## 3. Spec Lifecycle with Archiving

**The gap:** Completed tinyspec specs sit in `.specs/` forever alongside active ones. There's no way to distinguish "this is historical" from "this is in progress" other than checking task completion. The dashboard helps, but the directory itself grows unbounded.

**The proposal:** Add a `tinyspec archive <spec-name>` command that:

- Verifies all tasks are checked (or accepts `--force`)
- Moves the spec file to `.specs/archive/` (preserving group subdirectories)
- `tinyspec list` and `tinyspec dashboard` exclude archived specs by default
- `tinyspec list --archived` shows only archived specs
- `tinyspec dashboard` gets a toggle key (e.g., `a`) to show/hide archived specs

## 4. Spec Templates

**The gap:** Every new spec starts with the same blank four-section scaffold. Teams often develop patterns — a "bug fix" spec looks different from a "new feature" spec or an "API endpoint" spec. OpenSpec addresses this with structured artifact types (proposal.md, design.md, tasks.md).

**The proposal:** Support optional templates in `.specs/templates/`:

- `tinyspec new my-feature --template api` looks for `.specs/templates/api.md`
- Templates are regular Markdown files with the four sections pre-filled with guidance text
- `tinyspec new my-feature` (no flag) uses the default blank scaffold as today
- No built-in templates shipped — teams create their own

## 5. Multi-Tool Agent Instructions

**The gap:** Tinyspec's Claude Code skills are powerful but lock the workflow to a single AI tool. OpenSpec's `AGENTS.md` approach works with any tool that reads project files.

**The proposal:** Extend `tinyspec init` to also generate an `AGENTS.md` (or append to an existing one) that describes the tinyspec workflow, spec format, and available CLI commands in a tool-agnostic way. AI tools that read project-level instruction files (Cursor rules, Copilot instructions, Windsurf rules, etc.) can then understand and work with tinyspec specs even without the Claude Code skills.

## 6. Retrofitting — Generate Specs from Code

**The gap:** Tinyspec assumes you write specs before code. For existing features or legacy code, there's no way to generate a spec that documents what already exists.

**The proposal:** Add a `tinyspec retrofit <spec-name>` command that:

- Creates a new spec with the standard scaffold
- Pre-fills the Background and Proposal sections with a prompt instructing the AI to analyze the codebase and document the existing behavior
- The `/tinyspec:refine` skill recognizes retrofit specs and shifts to a documentation-first mode: reading code, asking questions, and populating the sections with descriptions of current behavior rather than proposed changes
- Implementation Plan and Test Plan remain empty (the feature already exists — the spec is documentation, not a build plan)

# Implementation Plan

# Test Plan
