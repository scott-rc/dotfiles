# Amend Operation

Fold outstanding changes into the last commit.

## Instructions

1. **Check current branch**:
   Check main branch protection per [git-patterns.md](git-patterns.md). If on main/master and not dotfiles, present branch options via AskUserQuestion.

2. **Check for changes to amend**:
   - Run `git status`, `git diff --staged`, and `git diff`
   - If there are no staged or unstaged changes, inform the user there is nothing to amend and stop

3. **Capture pre-amend scope**:
   - Detect base branch: `fish -c 'gbb'`
   - Fetch latest: `git fetch origin`
   - Record the current file set: `git diff --name-only origin/<base> HEAD`
   - Record the current commit message: `git log -1 --format=%B`

4. **Stage changes**:
   - If all changes belong together: `git add -A`
   - If mixed changes: group files by logical change and present groups as AskUserQuestion options, then `git add <files>`

5. **Amend the commit**:
   ```bash
   git commit --amend --no-edit
   ```
   If the commit fails due to a pre-commit hook: read the error output, fix the issue, re-stage changes, and retry the amend. MUST NOT use `--no-verify`.

6. **Evaluate commit message**:
   - Record the post-amend file set: `git diff --name-only origin/<base> HEAD`
   - Compare against the pre-amend file set from step 3
   - If the file sets are identical: keep the original message, skip to step 7
   - If files were added or removed: draft a new message per [commit-guidelines.md](commit-guidelines.md), present both via AskUserQuestion: the proposed new message and "Keep original message"
   - If the user picks the new message, apply it with `git commit --amend -m` (title-only) or `--amend -F` per the Shell-Safe Application rules in [commit-guidelines.md](commit-guidelines.md).

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
     - If files were added or removed: follow the [Update Description operation](update-description.md) steps 2-4 to rewrite the title and description

9. **Report**: Confirm what happened -- amend, message update (if any), force push (if any), PR description update (if any).

See [git-patterns.md](git-patterns.md) for base branch detection, dotfiles exception, and fetch safety patterns.
