# TinySpec

A tiny framework for writing specs for use with language models. Tinyspec integrates with [Claude Code](https://docs.anthropic.com/en/docs/claude-code) to provide a structured workflow for planning, refining, and implementing features. It supports multi-repo projects and spec grouping to keep larger efforts organized.

## Install

From crates.io:

```sh
cargo install tinyspec
```

From source:

```sh
git clone https://github.com/nmcdaines/tinyspec.git
cd tinyspec
cargo install --path .
```

## Usage

Tinyspec follows a spec-driven workflow: create a spec, refine it with Claude, then work through the implementation plan.

### 1. Initialize

Run `tinyspec init` in your project directory to install Claude Code skills and set up shell completions:

```sh
cd your-project
tinyspec init
```

This creates three skills in `.claude/skills/`:
- `/tinyspec:refine` — Collaborate with Claude to define the problem and build an implementation plan
- `/tinyspec:do` — Work through the implementation plan task by task
- `/tinyspec:task` — Complete a specific task from the plan

To update the skills after upgrading tinyspec:

```sh
tinyspec init --force
```

### 2. Create a spec

```sh
tinyspec new my-feature
```

This creates a timestamped Markdown file in `.specs/` with sections for Background, Proposal, Implementation Plan, and Test Plan. Open it in your editor to fill in the Background and Proposal:

```sh
tinyspec edit my-feature
```

To organize specs into a group, use the `group/name` syntax:

```sh
tinyspec new v1/my-feature
```

This creates the spec inside `.specs/v1/`. Groups are optional and only one level deep. Spec names must be globally unique across all groups, so every command can reference a spec by name alone:

```sh
tinyspec view my-feature    # works whether grouped or not
tinyspec edit my-feature
tinyspec status my-feature
```

### 3. Refine with Claude

In Claude Code, run:

```
/tinyspec:refine my-feature
```

Claude will read your spec, ask clarifying questions, and help you build out the Implementation Plan with task groups (A, B, C) and subtasks (A.1, A.2, etc.).

### 4. Implement

Once the plan is ready, run:

```
/tinyspec:do my-feature
```

Claude will work through each task group in order, checking off subtasks as they're completed and committing progress after each group.

To work on a specific task:

```
/tinyspec:task my-feature A.1
```

### 5. Track progress

```sh
tinyspec status my-feature
tinyspec status  # all specs
```

### 6. Dashboard

Launch a real-time TUI dashboard to monitor all specs at a glance:

```sh
tinyspec dashboard
```

The dashboard watches `.specs/` for changes and auto-refreshes. Specs are sorted by status (in-progress, pending, completed) and grouped by feature group with aggregate completion percentages.

**Controls:**
- `↑`/`↓` or `j`/`k` — navigate specs
- `Enter` — view a spec's Implementation Plan as a collapsible task tree
- `Esc` — return to the list
- `q` — quit

## Configure

When a spec references multiple repositories, tinyspec resolves application names to folder paths using `~/.tinyspec/config.yaml`.

Map a repository name to a path:

```sh
tinyspec config set my-app /path/to/my-app
```

List configured repositories:

```sh
tinyspec config list
```

Remove a mapping:

```sh
tinyspec config remove my-app
```

Then in your spec front matter, reference applications by name:

```yaml
---
tinySpec: v0
title: My Feature
applications:
  - my-app
---
```

## Templates

Templates let you customize the scaffold used when creating new specs with `tinyspec new`.

### Template locations

Templates are Markdown files stored in one of two directories:

- `.specs/templates/` — repo-level (takes precedence)
- `~/.config/tinyspec/templates/` — user-level (fallback)

The template name is the filename without the `.md` extension. A template named `default` is automatically applied when you run `tinyspec new` without the `--template` flag.

To use a specific template:

```sh
tinyspec new my-feature --template my-template
```

List available templates:

```sh
tinyspec templates
```

### Template variables

Templates support variable substitution using either `{{var}}` or `${var}` syntax. The following built-in variables are available:

| Variable | Description | Example |
|----------|-------------|---------|
| `title` | Title-cased spec name | `my-feature` → `My Feature` |
| `date` | Current date | `2026-02-18` |

Example template:

```markdown
---
tinySpec: v0
title: {{title}}
---

# Background

Created on ${date}.

# Proposal

# Implementation Plan

# Test Plan
```

Variables inside fenced code blocks and inline code are not substituted, so you can safely document variable syntax in your templates. Unknown variables are left as-is.

## Develop

Build from source:

```sh
cargo build
```

Run tests:

```sh
cargo test
```

Install your local build:

```sh
cargo install --path .
```

Install git hooks (runs `cargo fmt`, `cargo clippy --fix`, and `cargo test` before each commit):

```sh
./scripts/install-hooks.sh
```
