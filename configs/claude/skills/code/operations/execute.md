# Execute Plan

Orchestrate multi-phase plan execution with commit checkpoints, skip tracking, and interactive verification.

## Instructions

1. **Read the plan file**: If the user provided a path, use it. Otherwise, search for plan files in `./tmp/` (e.g., `tmp/feedback/plan.md`, `tmp/prd/*/plan.md`). If exactly one is found, use it. If multiple are found, present them as options to the user. If none are found, ask the user for the path. Parse phases from the plan file. Each phase has a title, "What to build" section, and acceptance criteria checkboxes (`- [ ]`). Identify the first phase with unchecked criteria -- resume from there.

2. **For each phase in order**:

   a. **Announce the phase**: Tell the user which phase you're starting and what it covers.

   b. **Implement**: Build what the phase describes. Route to the appropriate Write mode internally -- Feature mode for new behavior, Fix mode for bug fixes, Apply mode for refactoring/config/glue. For phases that span multiple concerns, work through them sequentially within the phase.

   c. **Verify acceptance criteria**: For each criterion, confirm it's met and mark `- [x]` in the plan file. Run the project's build and test commands. If a criterion can't be met after 3 attempts, STOP and report to the user -- do not continue to the next phase.

   d. **Interactive verification for UI work**: If the phase involves UI, frontend, or visual changes, MUST verify interactively -- start a dev server if not running, open the browser via preview tools, and confirm the feature works visually. Test suites miss keyboard handling bugs, CSS layering issues, and initial render state problems. Do not rely solely on test passes for UI phases.

   e. **Commit**: After all acceptance criteria pass, commit the phase's changes using the git skill (`skill: "git", args: "commit"`). Each phase = one focused commit. Do NOT batch multiple phases into a single commit.

   f. **Clean up processes**: Kill any dev servers or background processes started during this phase that aren't needed for the next phase.

3. **Skipping a phase**: If a phase is deemed unnecessary given prior phases' results, do NOT silently proceed. Instead:
   - Add `**Skipped**: <rationale>` below the phase title in the plan file
   - Mark all its checkboxes as `- [s]`
   - Inform the user which phase was skipped and why before continuing

4. **After all phases complete**: Kill any remaining dev servers or background processes, then report the full plan as done with a summary of phases completed, skipped, and any deferred work.
