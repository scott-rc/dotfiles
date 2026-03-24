# Split

Split a large branch into stacked branches grouped by logical concern, creating PRs for each so reviewers can evaluate changes incrementally.

## Instructions

### Phase 1: Planning (orchestrator, steps 0-4)

0. **Resume detection**: Derive the branch directory by running `~/.claude/skills/git/scripts/branch-context-path.sh` and stripping the filename. Check for `split-state.json` in that directory. If the file exists with `"status": "in-progress"`, ask via AskUserQuestion: "Found an in-progress split with N of M branches completed. Resume or start fresh?" -- Resume regenerates the orchestrator prompt from the state file (step 4 template) and enters plan mode; Start fresh deletes the state file and continues to step 1.

1. **Gather state**: Record the current branch as the reference branch -- it will become the last branch in the stack. Detect base branch per references/git-patterns.md (Base Branch Detection). Run:
   - `git fetch origin`
   - `git diff --stat origin/<base>...HEAD` for file count and line totals (triple-dot: excludes base-branch-only changes)
   - `git log --oneline origin/<base>..HEAD` for commit count and scope (double-dot: walks reachable commits)

   If fewer than 2 files changed AND fewer than 200 lines changed total, inform the user and stop.

2. **Analyze diff**: Delegate to an `Explore` subagent. Pass the output of `git diff origin/<base>...HEAD` and the stat summary. If the diff exceeds ~10,000 lines, pass `--stat` output instead and instruct the subagent to read individual file diffs selectively. The subagent MUST:
   - Propose **no more than 5 groups** — if there are many small concerns, merge them into larger coherent groups rather than listing each separately
   - Group changes by **logical concern** -- assign each file to exactly one group (see File Assignment below)
   - For each group: return theme, review focus, **changes** (plain-English bullets describing what to implement), relevant files, estimated line count, and dependency notes
   - Include associated generated files (test snapshots, codegen output) in each group's relevant_files alongside their source files
   - Propose an ordering (foundational changes first)
   - **File assignment**: Each file MUST appear in exactly one group. If a file has changes relevant to multiple concerns, assign it to the most relevant group. Later groups inherit the changes via stacking. This enables clean per-group staging without hunk-level splitting.

3. **Propose stack**: Present the recommended grouping in full detail via AskUserQuestion. For each branch show stack position, proposed name (`sc/` prefix per references/git-patterns.md Branch Naming), theme and review focus, changes bullets, relevant files (noting any overlaps with other branches), and estimated size. Include a note that other stack sizes are available (e.g., "Option A: 2 branches", "Option B: 3 branches (recommended)", "Option C: 4 branches") and the user can request a different size. The recommended option should balance reviewability with avoiding excessive branch count.

   Notes to include: each file is assigned to one branch (later branches inherit changes via stacking); each branch should compile independently (aspirational, not required); the reference branch is a guide, not an exact target. Ask the user to approve, request a different size, or modify. Apply modifications and confirm before proceeding.

4. **Write state file and enter plan mode**: Write `split-state.json` to the branch directory (derived from `~/.claude/skills/git/scripts/branch-context-path.sh` -- strip the filename to get the directory):

   ```json
   {
     "reference_branch": "sc/big-feature",
     "base_branch": "main",
     "status": "approved | in-progress | complete",
     "repo_root": "/absolute/path/to/repo",
     "git_spice_initialized": false,
     "stack": [
       {
         "position": 1,
         "branch_name": "sc/big-feature-schema",
         "theme": "Database schema changes",
         "review_focus": "Migration safety, index coverage",
         "changes": [
           "Add user_preferences table with JSON column",
           "Add index on users.email for new lookup pattern"
         ],
         "relevant_files": ["db/migrations/001.sql", "db/schema.prisma"],
         "estimated_lines": 45,
         "status": "pending | committed | verified | pr-created | failed",
         "pr_url": null,
         "error": null
       }
     ]
   }
   ```

   All branches in the stack use new branch names derived from the theme. The reference branch is not reused -- git-spice tracks topology for all branches equally.

   Write the orchestrator prompt (below) to the plan file (the path provided by the plan mode system message). Enter plan mode via `EnterPlanMode`, then `ExitPlanMode` for user approval.

   **Orchestrator prompt template** (fill `<...>` values from the state file):

   ```markdown
   # Goal

   Execute the stacked branch split defined in <state-file-path>.

   # Context

   - State file: <state-file-path>
   - Reference branch: <reference_branch> (guide for what to implement -- not an exact-match target)
   - Base branch: <base_branch>
   - Stack: <N> branches

   # Requirements

   1. Read the state file to get the full stack and all context.
   2. Initialize git-spice (if not already -- check `git_spice_initialized` in state file):
      ```bash
      git-spice repo init --trunk <base_branch> --remote origin --no-prompt
      git config spice.branchCreate.prefix sc/
      # Untrack the reference branch so it won't be submitted with the stack
      git-spice branch untrack <reference_branch> --no-prompt 2>/dev/null || true
      ```
      Update state file: set `git_spice_initialized: true`.
   3. **Restructure commits** — collapse all commits and re-stage by group. **Requires exclusive working tree access** — MUST NOT run concurrently with other agents on the same working tree (steps 3.c and error recovery use broad `git reset` commands):
      a. Ensure we're on the reference branch and it's tracked:
         ```bash
         git checkout <reference_branch>
         git-spice branch track --base <base_branch> --no-prompt 2>/dev/null || true
         ```
      b. Save a backup tag in case restructuring needs to be reverted:
         ```bash
         git tag split-backup
         ```
      c. Collapse all commits into unstaged working tree changes:
         ```bash
         git reset --soft origin/<base_branch>
         git reset HEAD
         ```
         Working tree now has the net change from all commits; nothing is staged.
      d. For each group in stack order, stage and commit:
         - Stage this group's files: `git add <relevant_files>`. Pass explicit file paths from the state file — do not use globs. Since each file is assigned to exactly one group (per the analysis), no deduplication is needed.
         - Commit with a properly formatted message per references/commit-message-format.md (format rules only — use plain `git commit` here, not `git-spice commit create`, since there are no upstack branches to restack pre-split). Draft the message from `git diff --staged`, noting branch N of M with theme. Write to `./tmp/commit-msg.txt`, sanitize, then: `~/.claude/skills/git/scripts/sanitize.sh --commit-msg ./tmp/commit-msg.txt && git commit -m "$(cat ./tmp/commit-msg.txt)"`.
         - Update state file: set branch status to `committed`.
      e. Verify no changes are left behind: `git diff` and `git diff --cached` should both be empty. If unstaged changes remain (files not assigned to any group), add them to the last group (which is HEAD): `git add <files> && git commit --amend --no-edit`.
   4. **Split into stacked branches** via git-spice:
      a. Capture the commit SHAs for each group (in stack order):
         ```bash
         git log --reverse --format=%H origin/<base_branch>..HEAD
         ```
      b. Split at commit boundaries per references/git-spice-patterns.md "Branch Split". Use `--at` for each group except the last (which stays on the reference branch). The SHA for each group is its final (topmost) commit; with `--reverse` in 4.a, this is the last line before the next group starts:
         ```bash
         git-spice branch split \
           --at <last-sha-of-group-1>:<branch-name-1> \
           --at <last-sha-of-group-2>:<branch-name-2> \
           ... --no-prompt
         ```
         Rename the reference branch to the last group's planned name: `git-spice branch rename <reference_branch> <last-branch-name> --no-prompt`.
      c. Verify each branch: navigate with `git-spice bottom`, then for each branch run verification per references/git-patterns.md "Local Fix Commands" (type checker, linter, tests for affected files). Move up with `git-spice up`. Update state to `verified` on success.
      d. If verification fails on a branch:
         - **Compilation issues** (missing imports, forward references to later branches): fix directly using error output, affected file paths, branch theme, and the constraint to only modify files on the current branch. Squash the fix: `git-spice branch squash --no-prompt`, then restack upstack: `git-spice upstack restack --no-prompt`.
         - **Test failures for features in later branches**: expected — note and skip.
         - **Other failures** after two fix attempts: ask the user via AskUserQuestion with options: "Retry verification fix", "Skip this branch's verification", "Stop the split".
      e. MUST write the branch context file for EVERY branch before proceeding to step 5. Use `branch-context-path.sh --branch <branch-name>` to get each path — do NOT rely on the current checkout. Write 1-3 sentences of purpose/motivation that naturally incorporate the theme and stack position (e.g., "Branch 2 of 4 in a stacked split. <theme purpose>. Review focus: <review_focus>.").
   5. **Submit the stack** — create PRs with proper titles and descriptions via git-spice. After step 4.b, the current branch is the renamed last-group branch (top of the stack). Navigate to the bottom before beginning the PR loop:
      a. Navigate to the bottom of the stack:
         ```bash
         git-spice bottom
         ```
      b. For each branch in stack order:
         - Verify the branch context file exists at `./tmp/branches/<sanitized-branch>/context.md`. If missing, write it now per step 4.e before proceeding.
         - Write PR title and body per references/pr-writer-rules.md (create mode). Gather context:
           - `base_branch`: previous stack branch (or `origin/<base>` for the first)
           - `commit_messages`: `git log <base-for-this-branch>..HEAD --format=%B`
           - `branch_context`: contents of this branch's context file
           - `context`: "Branch N of M in a stacked split."
         - Sanitize and submit via git-spice. The split flow uses branch-name slugs (not base-branch slugs as in pr-writer-rules.md create mode) to avoid collisions across the stack: `BRANCH_SLUG=$(echo "<branch_name>" | tr '/' '-')`:
           ```bash
           ~/.claude/skills/git/scripts/sanitize.sh ./tmp/pr-${BRANCH_SLUG}-body.txt
           ~/.claude/skills/git/scripts/sanitize.sh --title ./tmp/pr-${BRANCH_SLUG}-title.txt
           TITLE=$(cat ./tmp/pr-${BRANCH_SLUG}-title.txt)
           git-spice branch submit --title "$TITLE" --body "$(cat ./tmp/pr-${BRANCH_SLUG}-body.txt)" --no-prompt
           ```
         - Record PR URL via Stack Metadata (see references/git-spice-patterns.md "Stack Metadata via JSON"):
           ```bash
           git-spice log short --json | jq -r 'select(.name == "<branch>") | .change.url'
           ```
         - Update state file: set status to `pr-created`, record PR URL.
         - Navigate to the next branch: `git-spice up`
      c. Refresh navigation comments across all PRs now that the full stack is submitted:
         ```bash
         # Updates navigation comments only -- descriptions were set per-branch above
         git-spice stack submit --update-only --no-prompt
         ```
   6. Report all branches with PR URLs, themes, sizes. Navigate to the top of the stack: `git-spice top`.

   # Cross-Branch Compatibility

   Since the original code was authored as a single branch, splitting may create branches where an earlier branch doesn't compile independently (e.g., branch 1 changes a type signature that only branch 3's code handles). The verification step (4.c) catches these. Fix options:
   - Move the dependent file to an earlier group (re-do the restructure with adjusted file assignments — restore from `split-backup` tag)
   - Add a minimal compatibility fix on the affected branch (then squash with `git-spice branch squash --no-prompt`)

   # Error Handling

   - Update the state file on every status change so the split can be resumed.
   - If restructuring fails (step 3), restore from the backup tag: `git checkout <reference_branch> && git reset --hard split-backup`.
   - If `git-spice branch split` fails, restore from backup and retry with adjusted commit boundaries.
   - If a verification fix fails after two attempts, ask the user: retry / skip / stop.
   ```

### Phase 2: Execution (plan-mode orchestrator prompt)

The orchestrator prompt above drives execution. It reads the state file for all context -- no prior conversation knowledge is assumed. See the Requirements section in the prompt for full step details.
