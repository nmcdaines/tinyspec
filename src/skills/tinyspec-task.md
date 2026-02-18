---
name: tinyspec:task
description: Implement a single task from a spec's Implementation Plan
---

IMPORTANT: `tinyspec` is a native binary CLI tool (installed via cargo/crates.io), NOT an npm package. Run it directly as `tinyspec <command>`. Never use npm, npx, or node to run it.

The arguments contain a spec name and a task ID separated by a space: `$ARGUMENTS`
Parse the first word as the spec name and the second word as the task ID.

Read the tinyspec specification at `.specs/<spec-name>.md` (resolve by matching the suffix after the timestamp prefix).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to complete a specific task:

1. Read the full spec using `tinyspec view <spec-name>` to understand the context (Background, Proposal, Implementation Plan). This command resolves application references to folder paths automatically.
   - If `tinyspec view` fails with a config error, inform the user that they need to configure repository paths with `tinyspec config set <repo-name> <path>` and stop.
2. If the spec references applications (listed in the `applications` frontmatter field), explore the relevant parts of each referenced repository to understand context:
   - For each resolved application folder path, explore the directory tree and read key source files relevant to the task at hand.
   - Consider how the task's changes will interact across all referenced repositories.
   - If no `applications` field is present (or it's empty), explore only the current repository from the working directory onwards.
3. Locate the specified task in the Implementation Plan.
4. Implement just that task.
5. Mark it complete with `tinyspec check <spec-name> <task-id>`.
6. If the task has subtasks, complete and check each subtask as well.

If the task depends on uncompleted prior tasks, use the `AskUserQuestion` tool to warn the user and ask how to proceed. Always verify your work compiles/runs before marking the task complete.
