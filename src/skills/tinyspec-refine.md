---
name: tinyspec:refine
description: Refine and structure a tinyspec specification through collaborative discussion
---

IMPORTANT: `tinyspec` is a native binary CLI tool (installed via cargo/crates.io), NOT an npm package. Run it directly as `tinyspec <command>`. Never use npm, npx, or node to run it.

Read the tinyspec specification at `.specs/$ARGUMENTS.md` (resolve the name by matching the suffix after the timestamp prefix, e.g., `hello-world` matches `2025-02-17-09-36-hello-world.md`).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to collaborate with the user to refine this spec:

1. Read the spec using `tinyspec view <spec-name>` to understand the context. This command resolves application references to folder paths automatically.
   - If `tinyspec view` fails with a config error, inform the user that they need to configure repository paths with `tinyspec config set <repo-name> <path>` and stop.
2. If the spec references applications (listed in the `applications` frontmatter field), explore each referenced repository before proposing changes:
   - For each resolved application folder path, explore the directory tree and read key source files to understand the codebase structure, architecture, and patterns.
   - Consider how the proposed changes will interact across all referenced repositories.
   - If no `applications` field is present (or it's empty), explore only the current repository from the working directory onwards.
3. Ask clarifying questions about ambiguous requirements or missing context. Use the `AskUserQuestion` tool to present structured, selectable options rather than asking inline.
4. Suggest improvements to the Background and Proposal sections.
5. Once the user is satisfied with the problem definition, scaffold or update the **Implementation Plan**:
   - Break the work into logical task groups (A, B, C, ...)
   - Each group gets subtasks (A.1, A.2, ...)
   - Use markdown checkboxes: `- [ ] A: Task description`
6. If the user wants tests, scaffold the **Test Plan** using Given/When/Then syntax with task IDs (T.1, T.2, ...).
7. Wait for user approval before writing any changes to the spec file.

Use `tinyspec view <spec-name>` to read the current spec and directly edit the file when making approved changes. Keep the front matter and existing structure intact.

After editing a spec file directly, run `tinyspec format <spec-name>` to normalize the Markdown formatting.
