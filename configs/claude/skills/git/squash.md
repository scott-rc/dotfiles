# Squash Operation

Squash all commits on the current branch into a single commit.

## Instructions

1. Fetch the latest from remote:
   ```bash
   git fetch origin
   ```

2. Detect the base branch:
   ```bash
   git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||' || echo 'main'
   ```

3. Get the list of commits to squash: `git log origin/<base>..HEAD --oneline`

4. If there are uncommitted changes, commit them first (follow commit guidelines).

5. Analyze all commits to understand what work was done and why.

6. **Verify scope before squashing**:
   - Get files changed in original commits: `git diff --name-only origin/<base>...HEAD`
   - Get files that will be in squashed commit: `git diff --name-only origin/<base> HEAD`
   - If these lists differ, **STOP** and warn the user:
     - Show the unexpected files (files in the second list but not the first)
     - Explain this usually means the rebase picked up unrelated changes from conflict resolution
     - Offer to help fix with: `git rebase -i origin/<base>` to drop/edit problematic commits, or reset and start fresh

7. **Confirm before squashing**:
   - Show the commits that will be squashed (from step 3)
   - Show what files will be in the final commit: `git diff --stat origin/<base> HEAD`
   - Ask the user to confirm before proceeding

8. Squash all commits into one:
   ```bash
   git reset --soft origin/<base>
   git commit
   ```

9. Format the commit message following [commit-guidelines.md](commit-guidelines.md).
