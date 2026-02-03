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

6. **Confirm before squashing**:
   - Show the commits that will be squashed (from step 3)
   - Show what files will be in the final commit: `git diff --stat origin/<base> HEAD`
   - Ask the user to confirm before proceeding

7. Squash all commits into one:
   ```bash
   git reset --soft origin/<base>
   git commit
   ```

8. Format the commit message following [commit-guidelines.md](commit-guidelines.md).
