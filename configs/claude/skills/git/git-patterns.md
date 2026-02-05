# Git Patterns Reference

Shared patterns used across git skill operations. Reference this file for consistent implementation.

## Base Branch Detection

Detect the default branch (main/master) from the remote:

```bash
git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||' || echo 'main'
```

This returns the branch that `origin/HEAD` points to (typically `main` or `master`).

## Dotfiles Exception

The `dotfiles` repo is special—direct commits to main are acceptable there.

Detection: Check if the repo path ends with `/dotfiles`:
```bash
[[ "$(git rev-parse --show-toplevel)" == */dotfiles ]]
```

When this applies:
- **commit.md**: Skip the "create a branch first" prompt when on main
- **push.md**: Push directly to main without PR creation
- **worktree.md**: When in dotfiles repo, scan other repositories instead
- **clean-worktrees.md**: Exclude dotfiles repo from cleanup scans

## Main Branch Protection

Before committing or pushing on main/master:

1. Check if current branch is main or master: `git branch --show-current`
2. If yes, check for dotfiles exception (above)
3. If not dotfiles, ask user before proceeding

## Fetch Safety

Always use:
```bash
git fetch origin
```

**Never** use:
```bash
git fetch origin <branch>:<branch>  # WRONG - fails if branch is checked out in another worktree
```

After fetching, reference remote branches as `origin/<branch>`.

## Scope Verification

After rebase or before squash, verify the branch only contains expected changes.

> **Important**: These comparisons assume the branch has been rebased onto `origin/<base>`.
> If the branch has diverged (main advanced since the branch was created), the diff will
> include the reversal of main's changes. Rebase first: `git rebase origin/<base>`

```bash
# Show files that will be in the commit
git diff --name-only origin/<base> HEAD

# Show file stats for human review
git diff --stat origin/<base> HEAD
```

Ask the user to verify these files match the branch's intended scope. If unexpected files appear:
- Offer to investigate with `git log --oneline origin/<base>..HEAD`
- Offer to fix with `git rebase -i origin/<base>`
