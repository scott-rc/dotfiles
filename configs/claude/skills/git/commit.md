# Commit Operation

Commit outstanding changes with a well-formatted message.

## Instructions

1. **Check current branch**:
   Check main branch protection per [git-patterns.md](git-patterns.md). If on main/master and not dotfiles, present branch options via AskUserQuestion. If a branch is chosen, create and switch to it before committing.

2. **Review uncommitted changes**:
   - Show status: `git status`
   - Show changes: `git diff` (unstaged) and `git diff --staged` (staged)

3. **Stage files**:
   - If all changes belong together: `git add -A`
   - If mixed changes: group files by logical change and present groups as AskUserQuestion options, then `git add <files>`

4. **Create commit** following [commit-guidelines.md](commit-guidelines.md) (including its Shell-Safe Application rules).

5. **If commit fails due to a pre-commit hook**: read the error output, fix the issue, re-stage changes, and retry the commit. MUST NOT use `--no-verify`.

See [git-patterns.md](git-patterns.md) for dotfiles exception and main branch protection patterns.
