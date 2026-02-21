# Amend Operation

Fold outstanding changes into the last commit.

## Instructions

1. **Check current branch**:
   - If on `main` or `master`:
     - Skip this check for `dotfiles` repo (amending directly on main is fine there)
     - Otherwise, ask the user if they want to create a new branch first

2. **Check for changes to amend**:
   - Run `git status`, `git diff --staged`, and `git diff`
   - If there are no staged or unstaged changes, inform the user there is nothing to amend and stop

3. **Capture pre-amend scope**:
   - Detect base branch: `gbb`
   - Fetch latest: `git fetch origin`
   - Record the current file set: `git diff --name-only origin/<base> HEAD`
   - Record the current commit message: `git log -1 --format=%B`

4. **Stage changes**:
   - If all changes belong together: `git add -A`
   - If mixed changes: ask user which files to include, then `git add <files>`

5. **Amend the commit**:
   ```bash
   git commit --amend --no-edit
   ```
   If the commit fails due to a pre-commit hook: read the error output, fix the issue, re-stage changes, and retry the amend. MUST NOT use `--no-verify`.

6. **Evaluate commit message**:
   - Record the post-amend file set: `git diff --name-only origin/<base> HEAD`
   - Compare against the pre-amend file set from step 3
   - If the file sets are identical: keep the original message, skip to step 7
   - If files were added or removed: draft a new message per [commit-guidelines.md](commit-guidelines.md), show both the original and proposed message to the user, and let them choose
   - Apply with `git commit --amend -m "<message>"` if the user picks the new one

7. **Push if already pushed**:
   - Check if a remote tracking branch exists:
     ```bash
     git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null
     ```
   - If no remote branch: skip to step 8
   - If remote branch exists: confirm with the user, then `git push --force-with-lease`

8. **Evaluate PR description**:
   - Check for an existing PR:
     ```bash
     gh pr view --json number,url,title,body 2>/dev/null
     ```
   - If no PR exists: skip to step 9
   - Reuse the file-set comparison from step 6:
     - If the file sets are identical: keep the current PR description
     - If files were added or removed: follow the [Update Description operation](update-description.md) steps 2-5 to rewrite the title and description

9. **Report**: Confirm what happened -- amend, message update (if any), force push (if any), PR description update (if any).

See [git-patterns.md](git-patterns.md) for base branch detection, dotfiles exception, and fetch safety patterns.
