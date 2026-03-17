# Tinyspec

A tiny framework for writing specs for use with language models.

## Spec formatting

After directly editing a spec file (outside of `tinyspec` commands), always run `tinyspec format <spec-name>` to normalize the Markdown formatting. This keeps specs consistent and reduces noise in diffs.

Commands like `tinyspec new`, `tinyspec check`, and `tinyspec uncheck` auto-format automatically.

## Test plan tracking

The `# Test Plan` section is fully tracked alongside `# Implementation Plan`. Test tasks use `T`-prefixed IDs (`T.1`, `T.2`, `T.1.1`).

- `tinyspec check <spec> T.1` marks a test task done
- `tinyspec status <spec>` shows `N/M impl, P/Q tests` when test tasks are present
- A spec only reaches `Completed` status when **all** impl tasks and all test tasks are checked
- `tinyspec status --skip-tests` ignores test tasks in the completion count (useful for specs that use the Test Plan as freeform notes)
- The dashboard detail view shows `# Test Plan` as a separate collapsible section
- The `◑` icon (cyan) in the dashboard means: impl complete, tests still pending

## Skills reference

- `/tinyspec:chat [topic|spec-name]` — Start a free-form conversation before writing a spec. Supports three modes: no argument (open exploration), existing spec name (load and discuss), or free-text topic (codebase-grounded exploration). When the user says "write this up", Claude presents a decisions summary, then creates or updates the spec. Unresolved questions land in `# Open Questions`.
- `/tinyspec:new <description>` — Create a new spec from a description.
- `/tinyspec:refine <spec-name>` — Collaborate to refine an existing spec's structure and implementation plan.
- `/tinyspec:do [spec-name]` — Work through all Implementation Plan and Test Plan tasks in order.
- `/tinyspec:task <spec-name> <task-id>` — Implement a single task (supports both impl `A.1` and test `T.1` IDs).
- `/tinyspec:oneshot [prompt]` — Execute all pending specs or generate and execute from a prompt.

## CLI commands reference

- `tinyspec search <query> [--group <name>] [--status pending|in-progress|completed]` — Full-text search across spec titles and body content.
- `tinyspec list [--json] [--include-archived]` — List specs; `--json` returns a JSON array of all spec summaries.
- `tinyspec status [<spec>] [--json] [--include-archived]` — Show task completion; `--json` returns the full task tree.
- `tinyspec view <spec> [--json]` — Display spec contents; `--json` returns front matter fields and task tree.
- `tinyspec archive [<spec>|--all-completed]` — Move spec(s) to `.specs/archive/`; archived specs are hidden by default.
- `tinyspec unarchive <spec>` — Move a spec back from the archive to its original group.
- `tinyspec lint [<spec>|--all]` — Validate spec health (missing sections, empty sections, non-sequential IDs, unconfigured applications). Exits non-zero on errors.
- `tinyspec dashboard [--include-archived]` — Real-time TUI dashboard.
- `tinyspec hooks test <event>` — Fire a named event with dummy context to test hook configuration.

## Hooks

Hooks let you run shell commands in response to tinyspec lifecycle events. Configure them in `~/.tinyspec/config.yaml`:

```yaml
hooks:
  on_spec_complete:
    - 'echo "Spec $TINYSPEC_SPEC_TITLE completed"'
  on_task_check:
    - './scripts/update-dashboard.sh'
```

Project-level hooks can also be defined in `.tinyspec.yaml` at the repo root — they run before user-level hooks.

**Events:** `on_task_check`, `on_task_uncheck`, `on_spec_complete`, `on_spec_start`, `on_spec_create`

**Environment variables:** `TINYSPEC_EVENT`, `TINYSPEC_SPEC`, `TINYSPEC_SPEC_TITLE`, `TINYSPEC_SPEC_GROUP`, `TINYSPEC_TASK_ID`, `TINYSPEC_SPEC_PATH`

Hook failures (non-zero exit) print a warning but do not block the triggering command. Use `--no-hooks` on `check`, `uncheck`, or `new` to skip hooks for a single invocation.
