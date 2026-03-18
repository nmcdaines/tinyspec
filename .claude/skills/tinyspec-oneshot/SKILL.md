---
name: tinyspec:oneshot
description: Execute all pending specs or generate specs from a prompt and execute them
---

IMPORTANT: `tinyspec` is a native binary CLI tool (installed via cargo/crates.io), NOT an npm package. Run it directly as `tinyspec <command>`. Never use npm, npx, or node to run it.

This skill operates in two modes based on whether arguments are provided:

- **No arguments** (`$ARGUMENTS` is empty): Execute all incomplete specs (Mode 1)
- **With arguments** (`$ARGUMENTS` is non-empty): Generate specs from the prompt, then execute (Mode 2)

---

## Mode 1: Execute existing specs (no arguments)

1. Run `tinyspec status` to list all specs and their completion progress.
2. Identify incomplete specs — any spec where not all tasks are complete.
3. If there are no incomplete specs, inform the user that all specs are complete and stop.
4. For each incomplete spec (in chronological order, earliest first):
   a. Announce which spec you are starting (e.g., "Working on spec: feature-name (3/10 tasks complete)").
   b. Read the full spec using `tinyspec view <spec-name>` to understand the context.
      - If `tinyspec view` fails with a config error, inform the user that they need to configure repository paths with `tinyspec config set <repo-name> <path>` and stop.
   c. Find the next unchecked task in the Implementation Plan.
   d. Implement all subtasks within the current task group.
   e. After completing each subtask, mark it done with `tinyspec check <spec-name> <task-id>`.
   f. After completing a top-level task group and all its subtasks, mark the group done too.
   g. Commit progress with a descriptive commit message referencing the spec and task group.
   h. Move on to the next task group and repeat until the spec is fully complete.
5. After all specs are complete, summarize what was accomplished.

### Autonomous decision-making

You are operating in autonomous mode. When you encounter ambiguity or questions during implementation:

- **Self-resolve first**: Use your best judgment based on the spec's Background, Proposal, and surrounding code context. Choose sensible defaults and conventional approaches.
- **Look at existing patterns**: Check how similar things are done elsewhere in the codebase and follow those patterns.
- **Only defer to the user when truly necessary**: If a decision has significant architectural implications, affects user-facing behavior in a way that could go either way, or you genuinely cannot determine the right approach, then ask.
- When you must ask, use the `AskUserQuestion` tool to present structured, selectable options rather than asking inline.

### Error handling

If a task fails (code doesn't compile, tests fail, or implementation hits a dead end):

1. Do NOT silently skip or force your way through.
2. Use the `AskUserQuestion` tool to present the user with options:
   - **Retry**: Attempt the task again with a different approach.
   - **Skip**: Mark the task as skipped and move on to the next one.
   - **Stop**: Halt execution entirely so the user can investigate.
3. Include a brief description of what went wrong so the user can make an informed choice.

---

## Mode 2: Generate specs from prompt (with arguments)

The user's prompt is: `$ARGUMENTS`

### Step 1: Plan the specs

1. Analyze the prompt to identify the distinct features, requirements, and components of the product.
2. Break these into a logical sequence of specs, ordered so that foundational/dependency specs come first.
3. For each planned spec, determine:
   - A kebab-case name
   - A brief summary of what it covers
4. Present the planned spec list to the user for review using `AskUserQuestion`:
   - Show the ordered list of spec names and summaries
   - Options: **Approve and proceed**, **I want to make changes**

### Step 2: Create and refine specs

For each planned spec (in order):

1. Create the spec file using `tinyspec new <spec-name>`.
2. Populate the spec by directly editing the file:
   - **Background**: Context for why this feature/component is needed, how it fits into the product.
   - **Proposal**: Detailed description of what should be built, including behavior and requirements.
   - **Implementation Plan**: Break into task groups (A, B, C, ...) with subtasks (A.1, A.2, ...) using markdown checkboxes.
   - **Test Plan**: Add Given/When/Then test cases where appropriate.
3. Run `tinyspec format <spec-name>` after editing.
4. Use the codebase context, the user's prompt, and any previously created specs to inform the content.

### Step 3: Present for approval

1. After all specs are created, show the user a summary of every spec that was generated.
2. Use `AskUserQuestion` to ask for approval:
   - **Approve and execute**: Proceed to execute all specs (Mode 1 flow).
   - **Let me review first**: Stop so the user can review and refine specs manually.
3. If approved, execute all the newly created specs using the Mode 1 flow above.
