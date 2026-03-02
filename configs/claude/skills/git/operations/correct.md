# Correct

Propagate a user correction about what a change does to all artifacts that contain the incorrect claim.

## Context

When the user corrects a factual claim (e.g., "that's not what this does", "those bugs were introduced in this PR", "that flag doesn't exist"), the correction MUST propagate to every artifact that carries the wrong information -- not just the one the user is looking at.

## Artifacts

Check each of these for the incorrect claim. Order matters -- fix upstream sources first so downstream regeneration uses correct input.

| Artifact | How to check | How to fix |
|---|---|---|
| **Commit message** | `git log -1 --format=%B` (or `git log origin/<base>..HEAD --format=%B` for multi-commit branches) | Delegate to `committer` agent: "Amend the last commit message. Remove or correct: [incorrect claim]. The correction is: [what the user said]." |
| **Branch context file** | Read `tmp/branches/<sanitized-branch>.md` per references/git-patterns.md | Edit the file directly to remove or correct the claim |
| **Changeset files** | `ls .changeset/*.md 2>/dev/null` and read each | Edit affected files directly |
| **PR title** | `gh pr view --json title -q .title 2>/dev/null` | `gh pr edit --title "<corrected>"` |
| **PR description** | `gh pr view --json body -q .body 2>/dev/null` | Delegate to `pr-writer` agent with `mode: update` and `context` set to the correction |

## Instructions

1. **Understand the correction**: Identify what's wrong and what's right from the user's message. If unclear, ask via AskUserQuestion.

2. **Detect base branch**: per references/git-patterns.md.

3. **Scan all artifacts**: Check each artifact in the table above for the incorrect claim. Read them in parallel where possible.

4. **Report findings**: List which artifacts contain the incorrect information and which are clean. Do NOT ask for confirmation -- proceed to fix.

5. **Fix affected artifacts**: Apply corrections to all affected artifacts. For commit messages, delegate to `committer`. For PR descriptions, delegate to `pr-writer` per references/pr-writer-rules.md with `context` describing the correction. For branch context and changeset files, edit directly.

6. **Report**: Confirm what was updated. If the commit was amended and a remote tracking branch exists, present force push options via AskUserQuestion: "Force push (--force-with-lease)" or "Skip push".
