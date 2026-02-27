# Amend

Fold outstanding changes into the last commit.

## Instructions

1. **Fetch latest from remote**:
   ```bash
   git fetch origin
   ```

2. **Check current branch**: Check main branch protection per references/git-patterns.md. If on main/master and not dotfiles, present branch options via AskUserQuestion; suggested branch names MUST follow the `sc/` prefix convention from references/git-patterns.md.

3. **Check for changes to amend**: Run `git status`, `git diff --staged`, and `git diff`. If there are no staged or unstaged changes, inform the user there is nothing to amend and stop.

4. **Detect base branch**: Run `fish -c 'gbb'` to get the base branch name.

5. **Record pre-amend state**: Record the current file set (`git diff --name-only origin/<base> HEAD`) and the current commit message (`git log -1 --format=%B`).

6. **Amend the commit**: Delegate to the `committer` agent with prompt: "Amend the last commit with the current changes. No-edit."

7. **Compare file sets**: Record the post-amend file set (`git diff --name-only origin/<base> HEAD`) and compare against the pre-amend file set from step 5. If the file sets are identical, keep the original message and skip to step 9.

8. **Evaluate commit message** (file sets differ): Present options via AskUserQuestion: "Update commit message" or "Keep original message". If the user picks "Update commit message", delegate to the `committer` agent with prompt: "Amend the last commit with a new message. The diff has changed -- new files were added or removed. Here is the current diff summary: `git diff --stat origin/<base> HEAD`. Draft a message that reflects the updated scope." The committer will inspect the diff and draft an appropriate message.

9. **Push if already pushed**: Check if a remote tracking branch exists:
   ```bash
   git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null
   ```
   If no remote branch, skip to step 10. If remote branch exists, present options via AskUserQuestion: "Force push" or "Skip push". Only run `git push --force-with-lease` if the user picks "Force push".

10. **Evaluate PR description**: Check for an existing PR:
    ```bash
    gh pr view --json number,url,title,body 2>/dev/null
    ```
    If no PR exists, skip to step 11. Reuse the file-set comparison from step 7: if the file sets are identical, keep the current PR description. If files were added or removed, detect base branch per references/git-patterns.md, spawn the `pr-writer` agent per references/pr-writer-rules.md with:
    - `mode`: `update`
    - `base_branch`: detected base branch
    - `pr_number`: from the PR view above
    - `context` (optional): one sentence describing what changed in the amend

    Confirm the PR was updated, show the PR URL, and follow references/github-text.md for the text itself.

11. **Report**: Confirm what happened -- amend, message update (if any), force push (if any), PR description update (if any).
