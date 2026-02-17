IMPORTANT: `tinyspec` is a native binary CLI tool (installed via cargo/crates.io), NOT an npm package. Run it directly as `tinyspec <command>`. Never use npm, npx, or node to run it.

The arguments contain a spec name and a task ID separated by a space: `$ARGUMENTS`
Parse the first word as the spec name and the second word as the task ID.

Read the tinyspec specification at `.specs/<spec-name>.md` (resolve by matching the suffix after the timestamp prefix).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to complete a specific task:

1. Read the full spec using `tinyspec view <spec-name>` to understand the context (Background, Proposal, Implementation Plan). This command resolves application references to folder paths automatically.
   - If `tinyspec view` fails with a config error, inform the user that they need to configure repository paths with `tinyspec config set <repo-name> <path>` and stop.
   - If the spec references multiple applications, use the `AskUserQuestion` tool to ask the user which repositories to focus on before proceeding.
2. Locate the specified task in the Implementation Plan.
3. Implement just that task.
4. Mark it complete with `tinyspec check <spec-name> <task-id>`.
5. If the task has subtasks, complete and check each subtask as well.

If the task depends on uncompleted prior tasks, use the `AskUserQuestion` tool to warn the user and ask how to proceed. Always verify your work compiles/runs before marking the task complete.
