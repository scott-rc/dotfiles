---
name: plan
description: Conventions for phased implementation plans at tmp/<name>/plan.md -- the Brief, the phases, and working through them one commit at a time. Use when the user wants to plan a feature or refactor, break work into phases, execute a plan, or continue implementing one.
---

# Plan

Conventions for phased implementation plans. These are preferences, not procedures: plan and build the way a careful engineer would, and follow these rules where they apply.

## The plan file

- Plans live at `tmp/<name>/plan.md`, one file per piece of work. The file holds the Brief, then the phases.
- Never re-phase a plan that already has phases without asking.

## Brief

- Covers the problem, the decisions made (modules, interfaces, testing approach), and what is out of scope.
- No file paths or code snippets. They go stale before the plan is done.
- If there is no Brief yet, interview the user first and write one. A Brief written by `code architect` is a Brief.

## Phases

- Each phase has a title, a description of what to build, and acceptance criteria as checkboxes.
- Before reporting a new plan, have a fresh-context subagent check that the phases deliver the Brief and that cited facts (call sites, line numbers) are accurate. Verify its findings and apply what holds.

## Executing

- Start at the first phase with unchecked criteria. Earlier phases are done.
- Check criteria off in the file as you go. The file is the only record of progress.
- One commit per phase. Never batch phases into one commit, and never amend an earlier phase's commit to fix something a later phase found.
- If a phase turns out to be unnecessary, say so in the file and move on. If a criterion cannot be met, stop and report rather than checking it off.
