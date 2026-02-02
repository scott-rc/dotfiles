# Squash Operation

Squash all commits on the current branch into a single commit.

## Instructions

1. Detect the base branch:
   ```bash
   git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||' || echo 'main'
   ```

2. Get the list of commits to squash: `git log <base>..HEAD --oneline`

3. If there are uncommitted changes, commit them first (follow commit guidelines).

4. Analyze all commits to understand what work was done and why.

5. **Confirm before squashing**:
   - Show the commits that will be squashed (from step 2)
   - Show what files will be in the final commit: `git diff --stat <base> HEAD`
   - Ask the user to confirm before proceeding

6. Squash all commits into one:
   ```bash
   git reset --soft <base>
   git commit
   ```

7. Format the commit message following [commit-guidelines.md](commit-guidelines.md).
