# Commit Operation

Commit outstanding changes with a well-formatted message.

## Instructions

1. **Check current branch**:
   - If on `main` or `master`:
     - Skip this check for `dotfiles` repo (committing directly to main is fine there)
     - Otherwise, present options via AskUserQuestion: suggest 1-2 branch names inferred from the changes, plus "Stay on main"
     - If a branch is chosen, create and switch to it before committing

2. **Review uncommitted changes**:
   - Show status: `git status`
   - Show changes: `git diff` (unstaged) and `git diff --staged` (staged)

3. **Stage files**:
   - If all changes belong together: `git add -A`
   - If mixed changes: group files by logical change and present groups as AskUserQuestion options, then `git add <files>`

4. **Create commit** following [commit-guidelines.md](commit-guidelines.md)
   - For title-only commits, inline `-m` is acceptable:
     ```bash
     git commit -m "<title>"
     ```
   - For any multi-line message (or any message containing shell-significant characters like backticks), MUST write the message to a temp file and commit with `-F`:
     ```bash
     cat > ./.tmp-commit-msg.txt <<'EOF'
     <title>
     
     <body>
     EOF
     git commit -F ./.tmp-commit-msg.txt
     rm ./.tmp-commit-msg.txt
     ```
   - MUST NOT pass multi-line commit messages via inline `-m` arguments.

5. **If commit fails due to a pre-commit hook**: read the error output, fix the issue, re-stage changes, and retry the commit. MUST NOT use `--no-verify`.

See [git-patterns.md](git-patterns.md) for dotfiles exception and main branch protection patterns.
