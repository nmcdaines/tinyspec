---
name: tinyspec:do
description: Implement a spec's full Implementation Plan, working through tasks in order
---

IMPORTANT: `tinyspec` is a native binary CLI tool (installed via cargo/crates.io), NOT an npm package. Run it directly as `tinyspec <command>`. Never use npm, npx, or node to run it.

Read the tinyspec specification at `.specs/$ARGUMENTS.md` (resolve the name by matching the suffix after the timestamp prefix).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to work through the spec's Implementation Plan:

1. Read the full spec using `tinyspec view <spec-name>` to understand the context (Background, Proposal). This command resolves application references to folder paths automatically.
   - If `tinyspec view` fails with a config error, inform the user that they need to configure repository paths with `tinyspec config set <repo-name> <path>` and stop.
2. If the spec references applications (listed in the `applications` frontmatter field), explore each referenced repository before beginning work:
   - For each resolved application folder path, explore the directory tree and read key source files to understand the codebase structure, architecture, and patterns.
   - Consider how the implementation will interact across all referenced repositories.
   - If no `applications` field is present (or it's empty), explore only the current repository from the working directory onwards.
3. Run `tinyspec status <spec-name>` to see current progress.
4. Find the next unchecked task in the Implementation Plan (top-level tasks in order: A, B, C, ...).
5. For each top-level task group:
   a. Implement all subtasks within the group.
   b. After completing each subtask, mark it done with `tinyspec check <spec-name> <task-id>`.
   c. After completing the top-level task and all its subtasks, mark it done too.
   d. Commit your progress with a descriptive commit message referencing the spec and task group.
6. Move on to the next task group and repeat.

If you encounter ambiguity or a task that requires user input, use the `AskUserQuestion` tool to present structured, selectable options rather than asking inline. Always verify your work compiles/runs before marking tasks complete.
