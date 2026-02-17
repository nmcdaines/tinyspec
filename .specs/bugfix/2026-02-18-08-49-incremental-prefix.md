---
tinySpec: v0
title: Incremental Prefix
---

# Background

When `tinyspec new` is invoked by a script or Claude Code it's possible for multiple specs to be created in the same minute. Since the timestamp prefix uses minute precision (`YYYY-MM-DD-HH-MM`), this breaks the intent of the prefix being unique per spec.

# Proposal

Ensure that the timestamp prefix is unique when using `tinyspec new`. After generating the initial timestamp, scan existing spec files for a matching prefix. If there is a conflict, increment by 1 minute. Chrono handles rollover naturally (minute 59 → next hour, hour 23 → next day, etc.).

# Implementation Plan

- [x] A: Add unique prefix generation to `tinyspec new`
  - [x] A.1: After generating the initial timestamp in `commands.rs`, collect existing spec filenames and check if any start with the same `YYYY-MM-DD-HH-MM-` prefix
  - [x] A.2: If a conflict exists, increment the timestamp by 1 minute using `chrono::Duration` and re-check
  - [x] A.3: Loop until a unique prefix is found
- [x] B: Add tests
  - [x] B.1: Test that the no-conflict case uses the current timestamp
  - [x] B.2: Test that a conflicting prefix gets incremented by 1 minute
  - [x] B.3: Test rollover from minute 59 to the next hour
- [ ] 🧪 Run `cargo test` upon completion

# Test Plan
