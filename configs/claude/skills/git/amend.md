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

4. **Amend the commit**: Spawn the `committer` agent with prompt: "Amend the last commit with the current changes. No-edit."

5. **Evaluate commit message**:
   - Record the post-amend file set: `git diff --name-only origin/<base> HEAD`
   - Compare against the pre-amend file set from step 3
   - If the file sets are identical: keep the original message, skip to step 6
   - If files were added or removed: spawn the `committer` agent with prompt: "Draft a commit message for the current HEAD commit. Return only the message, do not commit." Present both via AskUserQuestion: the proposed new message and "Keep original message"
   - If the user picks the new message: spawn the `committer` agent with prompt: "Amend the last commit with this message: <message>"

6. **Push if already pushed**:
   - Check if a remote tracking branch exists:
     ```bash
     git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null
     ```
   - If no remote branch: skip to step 7
   - If remote branch exists: confirm with the user, then `git push --force-with-lease`

7. **Evaluate PR description**:
   - Check for an existing PR:
     ```bash
     gh pr view --json number,url,title,body 2>/dev/null
     ```
   - If no PR exists: skip to step 8
   - Reuse the file-set comparison from step 5:
     - If the file sets are identical: keep the current PR description
     - If files were added or removed: detect base branch per [git-patterns.md](git-patterns.md), spawn the `pr-writer` agent with mode `update`, base_branch, and pr_number, then confirm the PR was updated and show the PR URL

8. **Report**: Confirm what happened -- amend, message update (if any), force push (if any), PR description update (if any).

See [git-patterns.md](git-patterns.md) for base branch detection, dotfiles exception, and fetch safety patterns.
