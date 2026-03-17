---
tinySpec: v0
title: Conversational Spec Authoring (tinyspec-chat skill)
---

# Background

The existing skills have clear entry points — you already have a spec (`tinyspec-refine`), or you already have a task to implement (`tinyspec-do`, `tinyspec-task`). But a lot of real thinking happens before either of those states exists. Someone has a half-formed idea, a problem they've been noticing, or a topic they want to reason through before committing it to a structured spec.

Currently there's no skill for this early, exploratory phase. The closest option is `tinyspec-refine`, but it requires an existing spec with content to refine — it's not designed for open-ended conversation. Users end up either jumping straight into a blank spec (and struggling to fill it) or having an unstructured conversation with Claude that never produces anything actionable.

The gap is a skill that starts from zero — a topic, a problem statement, or even just a spec name — and ends with either an updated spec or a new one, whenever the user decides they're ready.

# Proposal

Add a `tinyspec-chat` skill that opens a free-form conversation between the user and Claude about any topic or existing spec. The conversation continues for as long as the user wants. At any point, the user can say "write this up" or "update the spec" and Claude acts on the conversation to date.

**Starting modes:**

- `/tinyspec:chat` — No spec specified. Claude asks what the user wants to explore.
- `/tinyspec:chat <spec-name>` — Load an existing spec as conversation context. Claude summarizes what it sees, then invites the user to discuss, question, or challenge it.
- `/tinyspec:chat <topic or free-text prompt>` — Start from a topic description with no existing spec. Claude explores the codebase for relevant context and begins asking questions.

**During conversation:**

Claude's role is to be a thinking partner, not a spec-writing machine. It should:

- Ask clarifying questions to surface assumptions
- Offer alternative framings if the user's initial approach seems problematic
- Reference the actual codebase when relevant ("that would touch the auth middleware in X")
- Maintain a running internal summary of what's been decided, what's open, and what's been ruled out
- Avoid pushing toward writing the spec prematurely — follow the user's pace

The conversation is fully open-ended: the user can go in circles, change their mind, revisit earlier decisions, or go on tangents. Claude tracks the thread without forcing closure.

**Exiting to a spec:**

When the user signals readiness (e.g., "let's write this up", "create a spec for this", "update the spec"), Claude:

1. Summarizes the key decisions and open questions from the conversation
1. Asks the user to confirm or adjust the summary before writing anything
1. Then either:
   - **Creates a new spec** via `tinyspec new`, populates Background and Proposal from the conversation, and scaffolds an Implementation Plan
   - **Updates an existing spec** by diffing the conversation's conclusions against the current spec content and applying only the changes that reflect what was decided

If the conversation produced more than one distinct idea, Claude offers to split into multiple specs rather than forcing everything into one.

**Capturing open questions:**

Any unresolved questions from the conversation (things explicitly left open, or things Claude flagged as needing more information) are written into a `# Open Questions` section in the spec, so they're not silently dropped.

# Implementation Plan

- [x] A: Implement the `tinyspec-chat` skill
  
  - [x] A.1: Create `src/skills/tinyspec-chat.md` with skill prompt
  - [x] A.2: Handle three starting modes: no argument (open topic), existing spec name, free-text topic
  - [x] A.3: When starting from an existing spec, load it with `tinyspec view` and summarize it before opening conversation
  - [x] A.4: When starting from a topic, explore the codebase for relevant context using the Explore agent
  - [x] A.5: Maintain an internal running summary throughout the conversation (not shown to user unless asked)
- [x] B: Implement conversation-to-spec transitions
  
  - [x] B.1: Detect user signals that they want to write or update a spec ("write this up", "create a spec", "update the spec", etc.)
  - [x] B.2: Before writing anything, present a structured summary of decisions and confirm with the user
  - [x] B.3: Implement "create new spec" path: derive a spec name, run `tinyspec new`, populate sections from conversation
  - [x] B.4: Implement "update existing spec" path: compare conversation conclusions to current spec, apply changes section by section
  - [x] B.5: If multiple distinct ideas emerged, offer to split into separate specs before writing
- [x] C: Implement `# Open Questions` section support
  
  - [x] C.1: Track unresolved questions during the conversation
  - [x] C.2: When writing a spec, append an `# Open Questions` section if any exist
  - [x] C.3: When updating an existing spec that already has `# Open Questions`, merge rather than overwrite
  - [x] C.4: Ensure `tinyspec format` handles the new section gracefully (no parser errors)
- [x] D: Register the skill and update init
  
  - [x] D.1: Add `tinyspec-chat` to `src/spec/init.rs` alongside existing skills
  - [x] D.2: Run `tinyspec init --force` to install the new skill in `.claude/skills/`
  - [x] D.3: Update `CLAUDE.md` to document the new skill

# Test Plan

- [x] T.1: `/tinyspec:chat` with no argument prompts Claude to ask what the user wants to explore
- [x] T.2: `/tinyspec:chat <existing-spec>` loads the spec and Claude produces a summary before opening conversation
- [x] T.3: `/tinyspec:chat <topic>` with no existing spec causes Claude to explore the codebase for context
- [x] T.4: After a multi-turn conversation, saying "write this up" triggers the summary-confirmation flow before any file is written
- [x] T.5: Saying "create a spec" results in a new spec file with Background, Proposal, and Implementation Plan populated from the conversation
- [x] T.6: Saying "update the spec" on a session started with an existing spec modifies only the relevant sections, not the whole file
- [x] T.7: Unresolved questions from the conversation appear in `# Open Questions` in the written spec
- [x] T.8: If the conversation contains two distinct ideas, Claude offers to split them into separate specs
- [x] T.9: `tinyspec format` runs without error on a spec that contains `# Open Questions`
