# Status Operation

Show a quick overview of the current git state.

## Instructions

Display the following information:

1. **Current branch**: `git branch --show-current`

2. **Tracking status**:
   ```bash
   git rev-parse --abbrev-ref @{upstream} 2>/dev/null
   ```
   - Show if branch tracks a remote, and commits ahead/behind

3. **Working tree status**:
   - Staged changes count
   - Unstaged changes count
   - Untracked files count
   - Use `git status --porcelain` and summarize

4. **Git-spice stack info** (if initialized):
   ```bash
   gs log short
   ```
   - Show the branch's position in the stack
   - Show if any branches need restacking

5. **Recent commits** (last 3):
   ```bash
   git log --oneline -3
   ```

Format output concisely for quick scanning.
