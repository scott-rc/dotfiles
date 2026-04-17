---
name: prd
description: Create a PRD through user interview, codebase exploration, and module design, saved as the Brief section of a plan file at ./tmp/<name>/plan.md. Use when the user wants to write a PRD, create a product requirements document, or plan a new feature.
---

This skill is invoked when the user wants to create a PRD. The output is a plan file at `./tmp/<name>/plan.md` with its `## Brief` section populated — ready for `plan create` to phase into runnable work. You may skip steps below if you don't consider them necessary.

## Process

1. Ask the user for a long, detailed description of the problem they want to solve and any potential ideas for solutions.

2. Explore the repo to verify their assertions and understand the current state of the codebase.

3. Interview the user relentlessly about every aspect of this plan until you reach a shared understanding. Walk down each branch of the design tree, resolving dependencies between decisions one-by-one.

4. Sketch out the major modules you will need to build or modify to complete the implementation. Actively look for opportunities to extract deep modules that can be tested in isolation.

A deep module (as opposed to a shallow module) is one which encapsulates a lot of functionality in a simple, testable interface which rarely changes.

Check with the user that these modules match their expectations. Check with the user which modules they want tests written for.

5. Once you have a complete understanding, write the plan file. Create `./tmp/<name>/` if it doesn't exist (flat layout — no `tmp/prd/` subdirectory). Save as `./tmp/<name>/plan.md` (e.g. `./tmp/user-onboarding/plan.md`). The file contains ONLY the `## Brief` section at this stage — no phases. Use the template below.

6. Report the plan file path and the next step: `plan create <plan-path>` to phase the Brief into a runnable plan.

<plan-brief-template>
# Plan: <name>

## Brief

### Problem Statement

The problem that the user is facing, from the user's perspective.

### Solution

The solution to the problem, from the user's perspective.

### User Stories

A LONG, numbered list of user stories. Each user story in the format:

1. As an <actor>, I want a <feature>, so that <benefit>

<user-story-example>
1. As a mobile bank customer, I want to see balance on my accounts, so that I can make better informed decisions about my spending
</user-story-example>

This list should be extensive and cover all aspects of the feature.

### Implementation Decisions

A list of implementation decisions that were made. Include:

- The modules that will be built/modified
- The interfaces of those modules that will be modified
- Technical clarifications from the developer
- Architectural decisions
- Schema changes
- API contracts
- Specific interactions

Do NOT include specific file paths or code snippets — they may go stale quickly.

### Testing Decisions

A list of testing decisions. Include:

- What makes a good test for this feature (test observable behavior, not implementation details)
- Which modules will be tested
- Prior art for the tests (similar types of tests already in the codebase)

### Out of Scope

Things explicitly out of scope for this PRD.

### Further Notes

Any further notes about the feature.

### Review Criteria

Criteria `plan create` will turn into acceptance criteria on the terminal review phase. Split into static code checks and behavioral checks.

**Code**:
- [one bullet per static criterion — e.g. "No feature-flag leakage", "Module interfaces match the PRD", "Test coverage ≥ X%", "Lint clean"]

**Behavior**:
- [one bullet per behavioral criterion — each user-visible behavior described in the user stories becomes a reviewable check, plus any regression surfaces worth exercising]
</plan-brief-template>
