---
name: tinyspec:do
description: Implement a spec's full Implementation Plan, working through tasks in order
---

IMPORTANT: `tinyspec` is a native binary CLI tool (installed via cargo/crates.io), NOT an npm package. Run it directly as `tinyspec <command>`. Never use npm, npx, or node to run it.

## Argument parsing

`$ARGUMENTS` may contain a spec name and an optional `--parallel` flag in any order. Parse them as follows:

- **Spec name**: the argument that does not start with `--`
- **Parallel mode**: enabled if `--parallel` is present anywhere in `$ARGUMENTS`

Examples:
- `my-spec --parallel` → spec: `my-spec`, parallel: enabled
- `--parallel my-spec` → spec: `my-spec`, parallel: enabled
- `my-spec` → spec: `my-spec`, parallel: disabled

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

## Setup

1. Read the full spec using `tinyspec view <spec-name>` to understand the context (Background, Proposal). This command resolves application references to folder paths automatically.
   - If `tinyspec view` fails with a config error, inform the user that they need to configure repository paths with `tinyspec config set <repo-name> <path>` and stop.
2. If the spec references applications (listed in the `applications` frontmatter field), explore each referenced repository before beginning work:
   - For each resolved application folder path, explore the directory tree and read key source files to understand the codebase structure, architecture, and patterns.
   - Consider how the implementation will interact across all referenced repositories.
   - If no `applications` field is present (or it's empty), explore only the current repository from the working directory onwards.
3. Run `tinyspec status <spec-name>` to see current progress.

## Dependency analysis (parallel mode only)

If `--parallel` was given, before starting any implementation work, analyse the top-level task groups to build an execution plan:

1. List every top-level task group (A, B, C, …) and read their subtask descriptions.
2. Reason about dependencies: does group B require output from group A? Would running them concurrently cause conflicts (e.g. both modify the same file or schema)? Use the task descriptions and your knowledge of the codebase to judge — look for phrases like "builds on", "extends", "uses the X from", or shared module references.
3. Produce an **execution plan** — an ordered list of batches, where each batch contains groups that are safe to run concurrently:
   - Groups with no dependencies on other groups go in the first batch.
   - Groups that depend on a previous batch go in a later batch.
   - Example: if A and B are independent but C depends on both, the plan is: `[A, B]` then `[C]`.

## Execution

### Sequential mode (default, no `--parallel`)

For each top-level task group in order:

1. Implement all subtasks within the group.
2. After completing each subtask, mark it done with `tinyspec check <spec-name> <task-id>`.
3. After completing the top-level task and all its subtasks, mark it done too.
4. Commit your progress with a descriptive commit message referencing the spec and task group.

Move on to the next task group and repeat.

### Parallel mode (`--parallel`)

Work through the execution plan batch by batch:

1. For each batch of independent groups, spawn one background agent per group using the Agent tool with `run_in_background: true`. Each agent's prompt should instruct it to implement that specific task group for the spec (read the spec for context, implement all subtasks in the group, mark each subtask and the group itself done with `tinyspec check`, and commit when complete).
2. Wait for all agents in the current batch to complete before starting the next batch.
3. After all batches are done, run `tinyspec status <spec-name>` to confirm all implementation tasks are checked.

If a batch contains only one group, you may implement it directly rather than spawning an agent.

## Test Plan

After all Implementation Plan tasks are complete, work through the Test Plan:

1. The Test Plan uses `T`-prefixed IDs (e.g. `T.1`, `T.2`, `T.1.1`).
2. For each test task, perform the described verification or test.
3. Mark each test task done with `tinyspec check <spec-name> T.N`.
4. Commit test plan progress after completing all test tasks.

If you encounter ambiguity or a task that requires user input, use the `AskUserQuestion` tool to present structured, selectable options rather than asking inline. Always verify your work compiles/runs before marking tasks complete.
