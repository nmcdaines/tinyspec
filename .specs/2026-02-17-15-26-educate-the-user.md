---
tinySpec: v0
title: Educate The User
applications:
  - tinyspec
---

# Background

The README currently has no useful content. Users discovering tinyspec have no guidance on installation, workflow, configuration, or development.

# Proposal

Rewrite `README.md` with four sections: Install (cargo install + from source), Usage (workflow walkthrough: init → new → refine → work), Configure (multi-repo setup), and Develop (building from source, running tests).

# Implementation Plan

- [x] A: Write README.md
  - [x] A.1: Write Install section — `cargo install tinyspec` from crates.io + clone & `cargo install --path .`
  - [x] A.2: Write Usage section — workflow walkthrough covering `init` → `new` → `/tinyspec:refine` → `/tinyspec:work`, explaining the full spec lifecycle
  - [x] A.3: Write Configure section — multi-repo setup with `tinyspec config set`
  - [x] A.4: Write Develop section — building from source, running tests

# Test Plan
