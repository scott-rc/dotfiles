# Rebase

Pull down the trunk branch and rebase the current branch (and its stack) onto it using git-spice.

## Instructions

1. Check if git-spice is initialized in this repo:
   ```bash
   gs repo init --check 2>/dev/null
   ```

2. **If git-spice is NOT initialized**:
   - Ask the user if they want to initialize it with `gs repo init`
   - If no, fall back to plain git rebase (fetch trunk, then `git rebase origin/<trunk>`)

3. **If git-spice IS initialized**, check if the current branch is tracked:
   ```bash
   gs log short
   ```
   - If the current branch appears in the output, it's tracked
   - If not, offer to track it with `gs branch track`

4. Fetch the latest from remote:
   ```bash
   git fetch origin
   ```

5. Restack the entire stack:
   ```bash
   gs stack restack
   ```

6. If the restack succeeds, report success to the user.

7. If there are conflicts:
   - Report the conflicting files
   - Offer to help resolve them or abort with `git rebase --abort`
