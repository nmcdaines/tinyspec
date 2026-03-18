---
name: tinyspec:refine
description: Refine and structure a tinyspec specification through collaborative discussion
---

IMPORTANT: `tinyspec` is a native binary CLI tool (installed via cargo/crates.io), NOT an npm package. Run it directly as `tinyspec <command>`. Never use npm, npx, or node to run it.

If `$ARGUMENTS` is empty, check for a focused spec with `tinyspec focus` (no arguments). If a spec is focused, use it. If not, prompt the user to specify a spec or run `tinyspec focus <spec-name>`.

Read the tinyspec specification at `.specs/<spec-name>.md` (resolve the name by matching the suffix after the timestamp prefix, e.g., `hello-world` matches `2025-02-17-09-36-hello-world.md`).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to collaborate with the user to refine this spec:

1. Read and understand the spec's Background and Proposal sections.
2. Ask clarifying questions about ambiguous requirements or missing context. Use the `AskUserQuestion` tool to present structured, selectable options rather than asking inline.
3. **Track decisions**: As the user answers questions and makes choices, keep a running list of key decisions with their reasoning. Each decision should capture: the topic, the chosen direction, and the reason (if given).
4. Suggest improvements to the Background and Proposal sections.
5. Once the user is satisfied with the problem definition, scaffold or update the **Implementation Plan**:
   - Break the work into logical task groups (A, B, C, ...)
   - Each group gets subtasks (A.1, A.2, ...)
   - Use markdown checkboxes: `- [ ] A: Task description`
6. If the user wants tests, scaffold the **Test Plan** using Given/When/Then syntax with task IDs (T.1, T.2, ...).
7. Wait for user approval before writing any changes to the spec file.
8. **After the user approves**, append a `# Decisions` section to the spec documenting the key Q&A from the session:

```markdown
# Decisions

- **Topic:** Chosen direction.
  *Reason: Why this was chosen.*
- **Topic:** Another decision.
  *Reason: Rationale.*
```

Use `tinyspec view <spec-name>` to read the current spec and directly edit the file when making approved changes. Keep the front matter and existing structure intact.

After editing a spec file directly, run `tinyspec format <spec-name>` to normalize the Markdown formatting.
