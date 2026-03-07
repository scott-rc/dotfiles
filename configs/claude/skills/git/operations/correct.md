# Correct

Propagate a user correction about what a change does to all artifacts that contain the incorrect claim.

## Context

When the user corrects a factual claim (e.g., "that's not what this does", "those bugs were introduced in this PR", "that flag doesn't exist"), the correction MUST propagate to every artifact that carries the wrong information -- not just the one the user is looking at.

## Instructions

1. **Understand the correction**: Identify what's wrong and what's right from the user's message. If unclear, ask via AskUserQuestion.

2. **Detect base branch**: per references/git-patterns.md.

3. **Scan all artifacts**: Read the following artifacts in parallel, checking each for the incorrect claim:
   - Commit message — `git log -1 --format=%B` (or `git log origin/<base>..HEAD --format=%B` for multi-commit branches)
   - Branch context file — the branch context file per references/git-patterns.md "Branch Context File"
   - Changeset files — `ls .changeset/*.md 2>/dev/null`, then read each
   - PR title — `gh pr view --json title -q .title 2>/dev/null`
   - PR description — `gh pr view --json body -q .body 2>/dev/null`

4. **Report findings**: List which artifacts contain the incorrect information and which are clean. Do NOT ask for confirmation -- proceed to fix.

5. **Fix affected artifacts**: Apply corrections to all affected artifacts. Order matters -- fix upstream sources first so downstream regeneration uses correct input:
   - Commit message -- the current message was already read in step 3. Apply the correction, then amend per the Inline Commit Procedure in references/commit-message-format.md (use `git commit --amend -F <file>`).
   - Branch context file — edit the file directly to remove or correct the claim
   - Changeset files — edit affected files directly
   - PR title — delegate to `pr-writer` agent with `mode: update` and `context` set to the correction (title only)
   - PR description — delegate to `pr-writer` agent with `mode: update` and `context` set to the correction

6. **Report**: Confirm what was updated. If the commit was amended and a remote tracking branch exists, present force push options via AskUserQuestion: "Force push (--force-with-lease)" or "Skip push".
