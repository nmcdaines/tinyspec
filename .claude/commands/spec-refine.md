Read the tinyspec specification at `.specs/$ARGUMENTS.md` (resolve the name by matching the suffix after the timestamp prefix, e.g., `hello-world` matches `2025-02-17-09-36-hello-world.md`).

If no matching spec is found, list available specs with `tinyspec list` and ask the user which one they meant.

Your goal is to collaborate with the user to refine this spec:

1. Read and understand the spec's Background and Proposal sections.
2. Ask clarifying questions about ambiguous requirements or missing context.
3. Suggest improvements to the Background and Proposal sections.
4. Once the user is satisfied with the problem definition, scaffold or update the **Implementation Plan**:
   - Break the work into logical task groups (A, B, C, ...)
   - Each group gets subtasks (A.1, A.2, ...)
   - Use markdown checkboxes: `- [ ] A: Task description`
5. If the user wants tests, scaffold the **Test Plan** using Given/When/Then syntax with task IDs (T.1, T.2, ...).
6. Wait for user approval before writing any changes to the spec file.

Use `tinyspec view <spec-name>` to read the current spec and directly edit the file when making approved changes. Keep the front matter and existing structure intact.

After editing a spec file directly, run `tinyspec format <spec-name>` to normalize the Markdown formatting.
