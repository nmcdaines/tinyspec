---
tinySpec: v0
title: Test Plan Tracking
---

# Background

Every tinyspec spec template includes a `# Test Plan` section, but the tool completely ignores it. Task IDs like `T.1`, `T.2` are never parsed, never surfaced in `tinyspec status`, and never checked off via `tinyspec check`. This means a spec can show "100% complete" in the dashboard while its test cases are entirely unverified.

This gap creates a false sense of completion and undermines the purpose of writing test plans in the first place. The test plan section is currently decoration — it has no operational weight.

# Proposal

Treat `# Test Plan` as a first-class tracked section alongside `# Implementation Plan`. Test tasks should:

- Be parsed by `summary.rs` using the same `TaskNode` structure, using `T` as the top-level prefix (e.g., `T.1`, `T.2`, `T.1.1`)
- Be checkable via `tinyspec check <spec-name> T.1`
- Contribute to the spec's overall completion count, or optionally be surfaced as a separate counter (e.g., `6/8 impl, 2/4 tests`)
- Be visible in the dashboard's detail view under a distinct "Test Plan" heading
- Block the spec from reaching `Completed` status unless all test tasks are also checked

The `# Test Plan` section should accept the same checkbox syntax as `# Implementation Plan`. The distinction is semantic — impl tasks are about building, test tasks are about verifying.

A `--skip-tests` flag on `tinyspec status` and in skills can be provided for specs that intentionally have no test tasks (or for users who use the section as freeform notes rather than tracked tasks).

# Implementation Plan

- [x] A: Extend the spec parser to recognize `# Test Plan`
  
  - [x] A.1: Update `summary.rs` to parse `# Test Plan` into a separate `test_tasks: Vec<TaskNode>` field on `SpecSummary`
  - [x] A.2: Support `T` prefix for top-level test tasks and `T.N` / `T.N.N` for nested ones
  - [x] A.3: Ensure `tinyspec check` and `tinyspec uncheck` accept `T`\-prefixed IDs
- [x] B: Update status calculation
  
  - [x] B.1: Add `total_tests` and `checked_tests` fields to `SpecSummary`
  - [x] B.2: Change `Completed` status to require all test tasks checked (in addition to impl tasks)
  - [x] B.3: Add `--skip-tests` flag to `tinyspec status` for specs without tracked test tasks
- [x] C: Surface test task progress in the dashboard
  
  - [x] C.1: Show separate impl and test progress bars (or combined with a distinct label) in the spec list view
  - [x] C.2: Show `# Test Plan` tasks under a distinct heading in the detail view, collapsible independently from the impl plan
  - [x] C.3: Distinguish between "impl done, tests pending" and "fully complete" visually
- [x] D: Update the `tinyspec-do` and `tinyspec-task` skills
  
  - [x] D.1: After completing all impl tasks, prompt Claude to work through the test plan tasks
  - [x] D.2: Teach `tinyspec-task` to accept `T`\-prefixed task IDs
- [x] E: Update templates and documentation
  
  - [x] E.1: Update the default spec template to include example `T.1`, `T.2` test task entries
  - [x] E.2: Update `CLAUDE.md` to document the test plan tracking behavior

# Test Plan

- [ ] T.1: Create a spec with test plan tasks; verify `tinyspec status` counts them separately
- [ ] T.2: Run `tinyspec check <spec> T.1`; verify it marks the task checked and reflects in status
- [ ] T.3: Mark all impl tasks done but leave test tasks unchecked; verify spec is `InProgress`, not `Completed`
- [ ] T.4: Mark all impl and test tasks done; verify spec reaches `Completed`
- [ ] T.5: Verify dashboard detail view shows `# Test Plan` section with correct checked state
- [ ] T.6: Verify `tinyspec-do` proceeds to test tasks after finishing impl tasks
