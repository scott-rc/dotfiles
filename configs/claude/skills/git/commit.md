# Commit Operation

Commit outstanding changes with a well-formatted message.

## Instructions

1. **Check current branch**:
   - If on `main` or `master`:
     - Skip this check for `dotfiles` repo (committing directly to main is fine there)
     - Otherwise, ask the user if they want to create a new branch first
     - If yes, create and switch to the new branch before committing

2. **Review uncommitted changes**:
   - Show status: `git status`
   - Show changes: `git diff` (unstaged) and `git diff --staged` (staged)

3. **Stage files**:
   - If all changes belong together: `git add -A`
   - If mixed changes: ask user which files to include, then `git add <files>`

4. **Create commit** following [commit-guidelines.md](commit-guidelines.md)

5. **If commit fails due to a pre-commit hook**: read the error output, fix the issue, re-stage changes, and retry the commit. MUST NOT use `--no-verify`.

See [git-patterns.md](git-patterns.md) for dotfiles exception and main branch protection patterns.
