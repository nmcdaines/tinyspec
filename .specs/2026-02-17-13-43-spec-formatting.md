---
tinySpec: v0
title: Spec Formatting
applications:
  - tinyspec
---

# Background

Presently, the spec formatting is not standardized. This can make specs difficult to read and understand. Additionally, a language model benefits from a standardized format to reduce noise when reading and writing specs.

# Proposal

Implement auto-formatting for spec files to enforce a standardized Markdown style. Rather than re-inventing the wheel, integrate an existing Rust-based Markdown formatter (such as [mdformat](https://crates.io/crates/mdformat) or [dprint](https://crates.io/crates/dprint-plugin-markdown)) into the tinyspec binary. The formatter should rewrite files in place to conform to the standard — not just report violations.

# Implementation Plan

- [x] A: Integrate a Markdown formatting library
  - [x] A.1: Evaluate Rust-based Markdown formatting crates and select one suitable for embedding
  - [x] A.2: Add the chosen crate as a dependency and implement a core formatting function that takes Markdown content and returns formatted output
  - [x] A.3: Ensure the formatter preserves YAML front matter intact
- [x] B: Implement the `tinyspec format` command
  - [x] B.1: Add a `format <SPEC_NAME>` subcommand that formats a single spec file in place
  - [x] B.2: Add a `format --all` flag to format all specs in the `.specs/` directory
- [x] C: Auto-format on spec mutation
  - [x] C.1: After any tinyspec command that modifies a spec file (e.g. `new`, `task`), automatically run the formatter on the affected file
- [x] D: Provide a skill/instruction for the language model
  - [x] D.1: Add a Claude skill or CLAUDE.md instruction that tells the model to run `tinyspec format <SPEC_NAME>` after editing a spec

# Test Plan

- [x] T.1: Given a spec with inconsistent formatting, when `tinyspec format <SPEC_NAME>` is run, then the file is rewritten with standardized formatting
- [x] T.2: Given a spec with YAML front matter, when formatted, then the front matter is preserved unchanged
- [x] T.3: Given the `--all` flag, when `tinyspec format --all` is run, then all specs in `.specs/` are formatted
- [x] T.4: Given a new spec is created with `tinyspec new`, when the command completes, then the resulting file is already formatted
