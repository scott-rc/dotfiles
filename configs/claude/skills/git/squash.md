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

5. Squash all commits into one:
   ```bash
   git reset --soft <base>
   git commit
   ```

6. Format the commit message following [commit-guidelines.md](commit-guidelines.md).
