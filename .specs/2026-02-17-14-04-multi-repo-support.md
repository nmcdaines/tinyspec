---
tinySpec: v0
title: Multi Repo Support
applications:
  - tinyspec
---

# Background

Whilst monorepos are a common approach to managing multiple projects, they are
not always chosen. This makes it difficult to write specs across multiple
repositories for a single project.

# Proposal

Tinyspec will support multi-repo deployments by having an optional applications
tag in the frontmatter of spec (markdown) file.

- The applications array is used to reference a name of another repository.
- The list of repository names to folder locations is to be stored in a config file in the user's config directory. e.g. ~/.tinyspec/config.yaml
  - The config file will define a mapping of repository names to folder locations. (see Appendix A)
  - A `tinyspec config` subcommand (set/list/remove) manages the config file.
- In the event the applications array is empty (or non existent) assume that the spec is for the current repository.
- If the applications array is not empty, the spec references the repositories in the applications array.
  - `tinyspec view` becomes the resolution point: it reads the config, resolves application names to folder paths, and performs find-and-replace in its output.
  - The `/spec-work` and `/spec-task` slash command prompts are updated to read specs via `tinyspec view <spec-name>` so that application references are resolved before Claude sees the spec content.
  - When multiple applications are specified, the prompt instructs Claude to ask the user which repositories to focus on.
  - In the event the config file is not found, or application names are not mapped, `tinyspec view` presents a clear error telling the user to create/update the config file and which repository names are missing.

## Appendix A. Example Config.yaml

```yaml
repositories:
  tinyspec: /path/to/tinyspec
  another-repo: /path/to/another-repo
```

# Implementation Plan

- [x] A: Config file infrastructure
  - [x] A.1: Add config types (`Config` struct, path helper for `~/.tinyspec/config.yaml`)
  - [x] A.2: Implement config read/write (YAML parse and serialize)
  - [x] A.3: Implement `tinyspec config set <repo-name> <path>`
  - [x] A.4: Implement `tinyspec config list`
  - [x] A.5: Implement `tinyspec config remove <repo-name>`
- [ ] B: Application resolution in `tinyspec view`
  - [ ] B.1: Parse `applications` array from spec frontmatter into the existing struct
  - [ ] B.2: Load config and resolve application names to folder paths
  - [ ] B.3: Perform find-and-replace of application names with resolved folder paths in view output
  - [ ] B.4: Error if config file is missing or application names are unmapped
- [ ] C: Update slash command prompts
  - [ ] C.1: Update `spec-work` prompt to read specs via `tinyspec view <spec-name>`
  - [ ] C.2: Update `spec-task` prompt to read specs via `tinyspec view <spec-name>`
  - [ ] C.3: Add instruction: when multiple applications are specified, prompt the user about which to focus on
  - [ ] C.4: Add instruction: when config is missing or repos unresolved, present a clear error to the user

# Test Plan

- [ ] T.1: Given no config file exists, when `config set tinyspec /path/to/tinyspec` is run, then `~/.tinyspec/config.yaml` is created with the mapping
- [ ] T.2: Given a config file with mappings exists, when `config list` is run, then all repository mappings are displayed
- [ ] T.3: Given a config file with a mapping exists, when `config remove <repo-name>` is run, then the mapping is deleted
- [ ] T.4: Given a config file with an existing mapping, when `config set` is run with the same name and a new path, then the path is updated
- [ ] T.5: Given a spec with applications and a valid config, when `view <spec-name>` is run, then application names are replaced with folder paths in the output
- [ ] T.6: Given a spec with applications but no config file, when `view <spec-name>` is run, then a clear error is shown
- [ ] T.7: Given a spec with applications and a config missing one mapping, when `view <spec-name>` is run, then an error identifies the unmapped name
- [ ] T.8: Given a spec with no applications, when `view <spec-name>` is run, then the spec is displayed normally (backwards compatible)
- [ ] T.9: Given `init` is run, then the generated slash command files contain updated multi-repo instructions
