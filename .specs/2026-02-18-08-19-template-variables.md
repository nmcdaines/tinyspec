---
tinySpec: v0
title: Template Variables
---

# Background

Tinyspec templates currently support two hardcoded variable substitutions — `{{title}}` and `{{date}}` — via simple string replacement in `commands.rs`. However, the default template shipped with `tinyspec init` uses `{ name }` which doesn't match the `{{...}}` syntax and is never actually substituted. The variable handling needs to be formalized and documented.

# Proposal

Formalize template variable substitution with the following behavior:

1. **Dual syntax support** — Templates can use either `{{variable}}` or `${variable}` syntax. Both are equivalent.
1. **Code block protection** — Variable substitution is skipped inside fenced code blocks (```` ``` ... ``` ````) and inline code (`` ` ... ` ``). This allows templates to document their own variable syntax without triggering substitution.
1. **Built-in variables:**
   - `{{title}}` / `${title}` — Title-cased version of the kebab-case spec name (e.g., `my-feature` → `My Feature`)
   - `{{date}}` / `${date}` — Current date in `YYYY-MM-DD` format
1. **Undefined variables** — Unknown variables (e.g., `{{foo}}`) are left as-is in the output. No warnings, no errors.
1. **Fix default template** — Update the default template to use `{{title}}` instead of `{ name }`.
1. **README documentation** — Add a "Templates" section to the README documenting template creation, storage locations, and variable syntax.

# Implementation Plan

- [x] A: Core variable substitution engine
  - [x] A.1: Extract variable substitution into a dedicated function that handles both `{{var}}` and `${var}` syntax
  - [x] A.2: Add code block detection — skip substitution inside fenced code blocks and inline code
  - [x] A.3: Wire the new substitution function into the `new_spec` command, replacing the current inline `.replace()` calls
- [x] B: Fix default template
  - [x] B.1: Update `.specs/templates/default.md` to use `{{title}}` instead of `{ name }`
  - [x] B.2: Update the built-in fallback scaffold in `commands.rs` if it also uses incorrect syntax
- [x] C: README documentation
  - [x] C.1: Add a "Templates" section to README.md covering template creation, storage locations (repo vs user-level), and the `tinyspec templates` command
  - [x] C.2: Document variable syntax (`{{var}}` and `${var}`), built-in variables, code block protection, and undefined variable behavior

# Test Plan

- [x] 🧪: Tests
  - [x] 🧪.1: Test `{{title}}` and `${title}` substitution
  - [x] 🧪.2: Test `{{date}}` and `${date}` substitution
  - [x] 🧪.3: Test that variables inside fenced code blocks are not substituted
  - [x] 🧪.4: Test that variables inside inline code are not substituted
  - [x] 🧪.5: Test that undefined variables are left as-is
  - [x] 🧪.6: Run `cargo test` upon completion
