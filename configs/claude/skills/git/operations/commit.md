# Commit

Commit outstanding changes with a well-formatted message.

## Instructions

**Routing**: If the user asked to **amend** rather than create a new commit, skip to the **Amend** path at the end of this file.

1. **Check branch protection**: MUST follow main branch protection per references/git-patterns.md. If on main/master and not dotfiles, present branch options via AskUserQuestion. If chosen, create and switch to the branch before committing. Branch name MUST follow the `sc/` prefix convention defined there.
2. **Ensure git-spice**: Run the Ensure Git-Spice pattern from references/git-spice-patterns.md.
3. **Branch context file (orchestrator -- do NOT delegate)**: If on main/master, skip this step. Otherwise, check if the branch context file exists (path per references/git-patterns.md "Branch Context File"). If missing, MUST run the Branch Context Creation pattern from `references/git-patterns.md` **before** proceeding to step 4. This step requires user interaction and MUST complete before proceeding to the commit step.
4. **Determine scope**: MUST identify the session file set -- files modified in this conversation. Skip the user prompt if:
   - The user asked to "commit all" or "commit everything" → stage everything
   - The user provided their own file list → use that list
   - The user said "commit this", "commit these changes", "commit my changes", or "commit what I changed" → use only the session file set
   If the resolved file set is empty (no files to stage), inform the user there is nothing to commit and stop.
   If none of those apply and `git status` shows modified files outside the session file set, ask the user which files to include before proceeding. If the user says to skip the extra files, proceed with only the session file set.
5. **Analyze cohesion**: Before staging, determine whether all changes form a single logical unit or represent mixed concerns. Consider: Do the files touch the same feature/system? Are there unrelated changes mixed in (e.g., a refactor alongside a bug fix)? Would a reviewer expect these in one commit? If diff output exceeds ~500 lines, use `git diff --stat` for an overview and read individual file diffs selectively.
6. **If mixed concerns detected**: Present the groups as AskUserQuestion options with descriptions (e.g., "Group 1: <description> — files: ..."). The user picks which group to commit first. Stage and commit only the selected files, then repeat for any remaining groups if the user wants.
7. **Commit**: Stage files, draft message, and commit per the Inline Commit Procedure in references/commit-message-format.md. Use `git-spice commit create -m "<message>" --no-prompt` instead of `git commit -m "<message>"` -- this commits and auto-restacks any upstack branches. Report via `git log -1 --oneline`.

8. **Context enrichment**: If on main/master, skip this step. If the branch context file exists and does NOT contain the `N/A` sentinel, check whether this commit introduces files in a new concern area not reflected in the context. Run `git diff --name-only HEAD~1..HEAD 2>/dev/null` (skip this step if the command fails, e.g., first commit on an orphan branch) and check if any files fall under a top-level directory (e.g., `.claude/`, `.github/`, `docs/`, CI config dirs) that the branch context does not mention. If a clearly distinct new category appears, suggest via AskUserQuestion: "This commit adds <category> changes not mentioned in branch context. Update context?" with options:
    - **"Update it"** -- run the Branch Context Creation pattern (update path) from `references/git-patterns.md`.
    - **"Skip"** -- proceed without updating.

    Only trigger when the new concern is clearly a distinct category -- not a new file in an existing concern area already covered by the context.

---

## Amend

Fold outstanding changes into the last commit.

1. **Fetch**: Run `git fetch origin`.

2. **Check for changes**: Run `git status`, `git diff --staged`, and `git diff`. If there are no staged or unstaged changes, inform the user there is nothing to amend and stop.

3. **Detect base branch**: Detect base branch per references/git-patterns.md.

4. **Record pre-amend state**: Record the current file set (`git diff --name-only origin/<base>...HEAD`) and the current commit message (`git log -1 --format=%B`).

5. **Amend the commit**: Stage changed files (`git diff --name-only` then `git add <file1> ...`), then `git-spice commit amend --no-prompt`. This amends the last commit AND auto-restacks any upstack branches in one atomic operation. If the pre-commit hook fails, read the error, fix the issue, re-stage, and retry. MUST NOT use `--no-verify`.

6. **Compare file sets**: Record the post-amend file set (`git diff --name-only origin/<base>...HEAD`) and compare against the pre-amend file set from step 4. If the file sets are identical, keep the original message and skip to step 8.

7. **Ask about commit message** (file sets differ): Present options via AskUserQuestion: "Update commit message" or "Keep original message". If the user picks "Update commit message", read `git diff --stat origin/<base>...HEAD` and draft a new message per the Inline Commit Procedure in references/commit-message-format.md (amend path). The message should reflect the updated scope.

8. **Push if already pushed**: Check if a remote tracking branch exists: `git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null`. If no remote branch, skip to step 9. If remote branch exists, present options via AskUserQuestion: "Force push" or "Skip push". Only force push if the user accepts: first run the Downstream PR Safety check from references/git-patterns.md, then check PR existence via the Stack Metadata via JSON pattern in references/git-spice-patterns.md (`.change` field). If the branch has a PR, use `git-spice branch submit --update-only --force --no-prompt`; if no PR, use `git-spice branch submit --no-publish --force --no-prompt`.

9. **Update PR description if needed**: Check for an existing PR: `gh pr view --json number,url,title,body 2>/dev/null`. If no PR exists, skip to step 10. Reuse the file-set comparison from step 6: if the file sets are identical, skip to step 10. If files were added or removed, run the Refresh Description mode in push.md with context noting what changed in the amend.

10. **Report**: Confirm what happened -- amend, message update (if any), force push (if any), PR description update (if any).
