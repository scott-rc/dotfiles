# Amend Operation

Fold outstanding changes into the last commit.

## Instructions

1. **Fetch latest from remote**:
   ```bash
   git fetch origin
   ```

2. **Check current branch**: Check main branch protection per [git-patterns.md](git-patterns.md). If on main/master and not dotfiles, present branch options via AskUserQuestion.

3. **Check for changes to amend**: Run `git status`, `git diff --staged`, and `git diff`. If there are no staged or unstaged changes, inform the user there is nothing to amend and stop.

4. **Detect base branch**: Run `fish -c 'gbb'` to get the base branch name.

5. **Record pre-amend state**: Record the current file set (`git diff --name-only origin/<base> HEAD`) and the current commit message (`git log -1 --format=%B`).

6. **Amend the commit**: Delegate to the `committer` agent with prompt: "Amend the last commit with the current changes. No-edit."

7. **Compare file sets**: Record the post-amend file set (`git diff --name-only origin/<base> HEAD`) and compare against the pre-amend file set from step 5. If the file sets are identical, keep the original message and skip to step 9.

8. **Evaluate commit message** (file sets differ): Delegate to the `committer` agent with prompt: "Draft a commit message for the current HEAD commit. Return only the message, do not commit." Present both via AskUserQuestion: the proposed new message and "Keep original message". If the user picks the new message, delegate to the `committer` agent with prompt: "Amend the last commit with this message: <message>".

9. **Push if already pushed**: Check if a remote tracking branch exists:
   ```bash
   git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null
   ```
   If no remote branch, skip to step 10. If remote branch exists, present options via AskUserQuestion: "Force push" or "Skip push". Only run `git push --force-with-lease` if the user picks "Force push".

10. **Evaluate PR description**: Check for an existing PR:
    ```bash
    gh pr view --json number,url,title,body 2>/dev/null
    ```
    If no PR exists, skip to step 11. Reuse the file-set comparison from step 7: if the file sets are identical, keep the current PR description. If files were added or removed, detect base branch per [git-patterns.md](git-patterns.md), spawn the `pr-writer` agent with mode `update`, base_branch, and pr_number, then confirm the PR was updated and show the PR URL.

11. **Report**: Confirm what happened -- amend, message update (if any), force push (if any), PR description update (if any).

See [git-patterns.md](git-patterns.md) for base branch detection, dotfiles exception, and fetch safety patterns.
