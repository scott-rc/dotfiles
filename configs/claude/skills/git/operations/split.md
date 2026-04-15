# Split

Split a large branch into stacked branches grouped by logical concern, creating PRs for each so reviewers can evaluate changes incrementally.

## Instructions

1. **Gather state**: Record the current branch as the reference branch. Detect base branch per references/git-patterns.md. Run `git fetch origin` (then Trunk Sync per references/git-patterns.md), `git diff --stat origin/<base>...HEAD` for file/line totals, and `git log --oneline origin/<base>..HEAD` for commit count. If fewer than 2 files changed AND fewer than 200 lines total, inform the user and stop. Untrack the reference branch so it won't be submitted with the stack: `git-spice branch untrack <reference_branch> --no-prompt 2>/dev/null || true`.

2. **Analyze diff**: Delegate to an `Explore` subagent with the diff (or `--stat` + selective file reads if >10,000 lines). The subagent MUST propose **no more than 5 groups** ordered by dependency (foundational first), grouped by **logical concern** with each file in exactly one group (assign to the most relevant concern; later branches inherit via stacking). For each group: theme, review focus, changes (plain-English bullets), relevant files (including generated files like snapshots/codegen), estimated line count, dependency notes.

3. **Propose stack**: Present the grouping via AskUserQuestion. For each branch show: stack position, proposed name (`sc/` prefix per references/git-patterns.md Branch Naming), theme, review focus, changes, relevant files, estimated size. Note that other stack sizes are available and the user can request a different grouping. Ask the user to approve or modify.

4. **Back up and collapse**: **Requires exclusive working tree access** -- uses broad `git reset` commands. Ensure on the reference branch (`git checkout <reference_branch>`; track if needed: `git-spice branch track --base <base_branch> --no-prompt 2>/dev/null || true`). Save a backup tag: `git tag split-backup`. Collapse all commits into unstaged changes: `git reset --soft origin/<base> && git reset HEAD`. If this fails, restore: `git checkout <reference_branch> && git reset --hard split-backup`.

5. **Commit by group**: For each group in stack order, stage its files (`git add <files>` -- explicit paths, no globs) and commit. Use plain `git commit` (no upstack branches to restack pre-split). Write the message to `./tmp/commit-msg.txt` per references/commit-message-format.md (format rules), then `~/.claude/skills/git/scripts/sanitize.sh --commit-msg ./tmp/commit-msg.txt && git commit -m "$(cat ./tmp/commit-msg.txt)"`.

6. **Verify clean state**: `git diff` and `git diff --cached` should both be empty. If unstaged changes remain (files not assigned to any group), amend them into the last commit: `git add <files> && git commit --amend --no-edit`.

7. **Split into branches**: Capture commit SHAs with `git log --reverse --format=%H origin/<base>..HEAD`. Split at commit boundaries per references/git-spice-patterns.md "Branch Split" -- pass `--at <sha>:<name>` for each group except the last. Rename the reference branch to the last group's name: `git-spice branch rename <reference_branch> <last-name> --no-prompt`. If the split fails, restore from `split-backup` and retry with adjusted boundaries.

8. **Verify branches**: Navigate to `git-spice bottom`, then for each branch moving up (`git-spice up`), run lint and tests per references/git-patterns.md "Local Fix Commands". Compilation issues (missing imports, forward references): fix directly, then `git-spice branch squash --no-prompt` and `git-spice upstack restack --no-prompt`. Test failures for features in later branches: expected -- skip. Other failures after two attempts: ask the user (retry / skip / stop).

9. **Write branch contexts**: For EVERY branch, write the context file per references/git-patterns.md "Branch Context File" via `branch-context-path.sh --branch <name>`. Content: 1-3 sentences incorporating theme and stack position (e.g., "Branch 2 of 4 in a stacked split. <theme purpose>. Review focus: <focus>.").

10. **Submit PRs**: Navigate to `git-spice bottom`. For each branch in stack order, write PR title and body per references/pr-writer-rules.md (create mode). Use branch-name slugs for temp files (not base-branch slugs, since multiple branches may share a base): `BRANCH_SLUG=$(echo "<name>" | tr '/' '-')`. Sanitize both title (`--title` mode) and body per references/github-text.md, then `TITLE=$(cat ./tmp/pr-${BRANCH_SLUG}-title.txt)`. Set `context` to "Branch N of M in a stacked split." Submit: `git-spice branch submit --title "$TITLE" --body "$(cat ./tmp/pr-${BRANCH_SLUG}-body.txt)" --no-prompt`. Move up: `git-spice up`. After all branches, refresh navigation comments: `git-spice stack submit --update-only --no-prompt`.

11. **Report**: Show all branches with PR URLs (from `git-spice log short --json`), themes, and sizes. Navigate to the top: `git-spice top`.
