Read the tinyspec specification at `.specs/$ARGUMENTS.md` (resolve the name by matching the suffix after the timestamp prefix).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to work through the spec's Implementation Plan:

1. Read the full spec using `tinyspec view <spec-name>` to understand the context (Background, Proposal). This command resolves application references to folder paths automatically.
   - If `tinyspec view` fails with a config error, inform the user that they need to configure repository paths with `tinyspec config set <repo-name> <path>` and stop.
   - If the spec references multiple applications, ask the user which repositories to focus on before proceeding.
2. Run `tinyspec status <spec-name>` to see current progress.
3. Find the next unchecked task in the Implementation Plan (top-level tasks in order: A, B, C, ...).
4. For each top-level task group:
   a. Implement all subtasks within the group.
   b. After completing each subtask, mark it done with `tinyspec check <spec-name> <task-id>`.
   c. After completing the top-level task and all its subtasks, mark it done too.
   d. Commit your progress with a descriptive commit message referencing the spec and task group.
5. Move on to the next task group and repeat.

If you encounter ambiguity or a task that requires user input, stop and ask before proceeding. Always verify your work compiles/runs before marking tasks complete.
