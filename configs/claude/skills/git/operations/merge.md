# Merge

Merge a PR via the GitHub API, then sync the local stack by default.

## Instructions

1. **Identify the target PR**:
   - If the user supplied a PR number, use it.
   - Otherwise, resolve the current branch's CR via `git-spice log short --json 2>/dev/null | jq -e --arg b "$(git branch --show-current)" 'select(.name == $b) | .change.id'` (the `#` prefix is part of the value; strip it for `gh`).
   - If no CR exists for the current branch and the user did not supply one, ask which PR to merge.

2. **Verify mergeable state**:
   ```bash
   gh pr view <pr> --json mergeable,state,statusCheckRollup --jq '{mergeable, state, failedChecks: [.statusCheckRollup[] | select((.state // .conclusion) | IN("FAILURE","ERROR"))] | length}'
   ```
   - If `state` is not `OPEN`, stop and report.
   - If `mergeable` is `CONFLICTING`, stop and tell the user to resolve conflicts.
   - If `failedChecks > 0`, stop and surface the failures (user can rerun /git fix or override explicitly).
   - If `mergeable` is `UNKNOWN`, wait briefly (`sleep 5`) and re-check up to 3 times before stopping.

3. **Detect merge strategy** from repo convention. Read recent merges on the default branch:
   ```bash
   git log --merges origin/$(git rev-parse --abbrev-ref origin/HEAD | sed 's|origin/||') --oneline -5
   ```
   - Merge commits (`Merge pull request #N from ...`) → use `--merge` (default).
   - Squashed PRs (single-commit messages matching PR titles) → use `--squash`.
   - Linear history (each commit looks like a PR commit, no merges) → use `--rebase`.
   - If the recent history is mixed or ambiguous, ask the user.

4. **Merge**:
   ```bash
   gh pr merge <pr> --<strategy> --delete-branch
   ```
   `--delete-branch` removes the remote branch on merge so the next sync prunes the local copy cleanly.

5. **Verify merged**:
   ```bash
   gh pr view <pr> --json state,mergedAt --jq '{state, mergedAt}'
   ```
   `state` must be `MERGED`.

6. **Sync by default**. Unless the user explicitly opted out (see Combined Operations entry "**merge without sync**" / "**merge only**" / "**merge and skip sync**"), immediately run the Sync operation (`operations/sync.md`) — `git-spice repo sync --restack --no-prompt` fetches the merge commit, prunes the deleted branch, restacks any upstack branches onto the new base, and reports the resulting stack. Without sync, upstack branches keep their old base ref locally and any open downstream PRs continue targeting the (now-deleted) branch on GitHub, which silently breaks the stack.

7. **Force-push any restacked branches**. After sync, run `git-spice log short --json` and identify branches where `.push.behind > 0` (history rewritten by restack but not yet pushed). For each such branch with an open CR, switch to it and run `git-spice branch submit --update-only --force --no-prompt` per the Push via Git-Spice pattern in references/git-spice-patterns.md. Run the Downstream PR Safety check from references/git-patterns.md first if any of the restacked branches have downstream PRs from other developers.

8. **Report**: which PR merged, the strategy used, the sync summary (branches pruned, branches restacked), any force-pushed branches, and the final `git-spice log short`.

## Rules

- **Never merge to override failing required checks.** If GitHub's branch protection requires checks that haven't passed, fix the underlying issue or wait — do NOT use `--admin` to bypass.
- **Never skip the sync** when the merged branch was part of a git-spice stack. Upstack branches and their PRs will silently drift from main otherwise; the next push from an upstack branch may then either fail (base ref deleted) or push to a stale base. Sync resolves both in one step.
- **Never merge a draft PR** without an explicit user instruction to mark it ready first.
- **Default to merge commit (`--merge`)** unless repo convention clearly indicates otherwise (step 3). Matching convention preserves linear/non-linear history expectations downstream tools rely on.
