# Tinyspec

A tiny framework for writing specs for use with language models.

## Spec formatting

After directly editing a spec file (outside of `tinyspec` commands), always run `tinyspec format <spec-name>` to normalize the Markdown formatting. This keeps specs consistent and reduces noise in diffs.

Commands like `tinyspec new`, `tinyspec check`, and `tinyspec uncheck` auto-format automatically.

## CLI commands reference

- `tinyspec search <query> [--group <name>] [--status pending|in-progress|completed]` — Full-text search across spec titles and body content.
- `tinyspec list [--json] [--include-archived]` — List specs; `--json` returns a JSON array of all spec summaries.
- `tinyspec status [<spec>] [--json] [--include-archived]` — Show task completion; `--json` returns the full task tree.
- `tinyspec view <spec> [--json]` — Display spec contents; `--json` returns front matter fields and task tree.
- `tinyspec archive [<spec>|--all-completed]` — Move spec(s) to `.specs/archive/`; archived specs are hidden by default.
- `tinyspec unarchive <spec>` — Move a spec back from the archive to its original group.
- `tinyspec lint [<spec>|--all]` — Validate spec health (missing sections, empty sections, non-sequential IDs, unconfigured applications). Exits non-zero on errors.
- `tinyspec dashboard [--include-archived]` — Real-time TUI dashboard.
