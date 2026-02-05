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

5. Rebase onto the base branch so all comparisons against `origin/<base>` are accurate:
   ```bash
   git rebase origin/<base>
   ```
   If conflicts occur, help resolve them or abort with `git rebase --abort`.

6. **Analyze all commits** to understand what work was done and why:
   ```bash
   git log origin/<base>..HEAD --format="%h %s%n%b"
   ```
   Note the overall purpose for crafting the squash message.

7. **Verify scope before squashing**:
   - Show files that will be in the squashed commit:
     ```bash
     git diff --name-only origin/<base> HEAD
     git diff --stat origin/<base> HEAD
     ```
   - Ask user to verify these files match the branch's intended scope
   - If unexpected files appear, offer to:
     - Investigate with `git log --oneline origin/<base>..HEAD`
     - Fix with `git rebase -i origin/<base>` to drop/edit problematic commits

8. **Confirm before squashing**:
   - Show the commits that will be squashed (from step 3)
   - Show what files will be in the final commit: `git diff --stat origin/<base> HEAD`
   - Ask the user to confirm before proceeding

9. Squash all commits into one:
   ```bash
   git reset --soft origin/<base>
   git commit
   ```

10. Format the commit message following [commit-guidelines.md](commit-guidelines.md).

See [git-patterns.md](git-patterns.md) for base branch detection and scope verification patterns.
