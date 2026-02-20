---
name: tinyspec:new
description: Create a new tinyspec spec from a description, populate it, and refine it collaboratively
---

IMPORTANT: `tinyspec` is a native binary CLI tool (installed via cargo/crates.io), NOT an npm package. Run it directly as `tinyspec <command>`. Never use npm, npx, or node to run it.

The user's description is: `$ARGUMENTS`

If `$ARGUMENTS` is empty, ask the user to provide a description of the feature or change they want to spec out, then proceed.

Your goal is to create a new spec, populate it with initial content based on the description and codebase context, then refine it collaboratively with the user.

## Step 1: Determine the spec name

Derive a short, descriptive kebab-case spec name from the description. For example:
- "Add user authentication" → `user-authentication`
- "Fix the login redirect bug" → `fix-login-redirect`
- "Support dark mode" → `dark-mode`

## Step 2: Create the spec

Run `tinyspec new <spec-name>` to create the spec file.

## Step 3: Explore the codebase

Before populating the spec, explore the codebase to understand the relevant context:

1. Run `tinyspec view <spec-name>` to confirm the file was created.
   - If `tinyspec view` fails with a config error, inform the user that they need to configure repository paths with `tinyspec config set <repo-name> <path>` and stop.
2. If the spec references applications (listed in the `applications` frontmatter field), explore each referenced repository:
   - For each resolved application folder path, explore the directory tree and read key source files to understand the codebase structure, architecture, and patterns.
   - If no `applications` field is present (or it's empty), explore only the current repository from the working directory onwards.

## Step 4: Populate Background and Proposal

Directly edit the spec file to populate:

- **Background**: Explain why this feature/change is needed and how it fits into the existing codebase. Draw on your codebase exploration to ground the context in reality.
- **Proposal**: Describe what should be built, including intended behavior, requirements, and any relevant constraints or tradeoffs.

Run `tinyspec format <spec-name>` after editing.

## Step 5: Refine collaboratively

Now collaborate with the user to refine the spec:

1. Read the populated spec using `tinyspec view <spec-name>`.
2. Ask clarifying questions about ambiguous requirements or missing context. Use the `AskUserQuestion` tool to present structured, selectable options rather than asking inline.
3. Suggest improvements to the Background and Proposal sections.
4. Once the user is satisfied with the problem definition, scaffold or update the **Implementation Plan**:
   - Break the work into logical task groups (A, B, C, ...)
   - Each group gets subtasks (A.1, A.2, ...)
   - Use markdown checkboxes: `- [ ] A: Task description`
5. If the user wants tests, scaffold the **Test Plan** using Given/When/Then syntax with task IDs (T.1, T.2, ...).
6. Wait for user approval before writing any changes to the spec file.

Use `tinyspec view <spec-name>` to read the current spec and directly edit the file when making approved changes. Keep the front matter and existing structure intact.

After editing a spec file directly, run `tinyspec format <spec-name>` to normalize the Markdown formatting.
