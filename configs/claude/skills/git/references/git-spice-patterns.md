# Git-Spice Patterns Reference

git-spice patterns for stacked branch management. Reference this file for all git-spice-specific operations.

## Contents

- Detection
- Initialization
- Ensure Git-Spice
- Tracked Branch Check
- Stack Metadata via JSON
- Push via Git-Spice
- Commit via Git-Spice
- Amend via Git-Spice
- Squash via Git-Spice
- Rebase Conflict Resolution
- Branch Split
- Stack Navigation
- Branch Fold
- Stack Submit
- CR Discovery

## Detection

Check if the repo is initialized:
```bash
git rev-parse --verify refs/spice/data 2>/dev/null
```
Succeeds (exit 0) if git-spice is initialized for this repo.

## Initialization

Auto-initialize whenever any operation needs git-spice and it is not yet initialized. Do NOT prompt the user — initialize silently.

```bash
git-spice repo init --trunk <base> --remote origin --no-prompt
git config spice.branchCreate.prefix sc/
```

## Ensure Git-Spice

A composite pattern that all operations use before running any `git-spice` command:

1. **Detect**: Run the Detection check above. If git-spice is already initialized, skip to step 3.
2. **Initialize**: Run initialization silently (no user prompt):
   ```bash
   DEFAULT_BRANCH=$(git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||')
   git-spice repo init --trunk "${DEFAULT_BRANCH:-main}" --remote origin --no-prompt
   ```
   Then: `git config spice.branchCreate.prefix sc/`
3. **Check tracked**: If the current branch is trunk (main/master) or the dotfiles exception applies (on main in dotfiles repo), skip to step 4. Otherwise, check if the current branch is tracked per the Tracked Branch Check below. If not tracked, run `git-spice branch track --no-prompt` to track it (git-spice auto-guesses the base by comparing against other tracked branches).
4. **Done**: git-spice is ready to use.

## Tracked Branch Check

Check if a branch is tracked by git-spice:
```bash
git-spice log short --json 2>/dev/null | jq -e 'select(.name == "<branch>")' >/dev/null
```
Exit 0 means the branch appears in git-spice's tracked stack. This is read-only — it does not switch branches.

## Stack Metadata via JSON

`git-spice log short --json` outputs one JSON object per line (JSONL). Each object has these fields:

- `.name` — branch name
- `.change` — object with `.id` (e.g. `"#2315"`) and `.url`; absent if no PR exists
- `.push` — object with `.ahead` (local commits not pushed) and `.behind` (remote commits not local, indicating divergence from rebase/amend); absent if not applicable
- `.down` — object with `.name` (base branch) and `.needsRestack` (boolean); absent on trunk
- `.current` — boolean; only present on trunk when it is checked out
- `.ups` — array of upstream branch objects; only present on trunk
- `.worktree` — worktree path string; only present when the branch is checked out in a worktree

jq recipes:

- All branches in the stack: `jq -r '.name'`
- Branches with existing PRs: `jq -r 'select(.change != null) | "\(.name) \(.change.id)"'`
- Check if specific branch has a PR: `jq -e 'select(.name == "<branch>") | .change' 2>/dev/null`
- Branches needing push (ahead > 0): `jq -r 'select(.push.ahead > 0) | .name'`
- Branches with divergence (behind > 0, needs force push): `jq -r 'select(.push.behind > 0) | .name'`

## Push via Git-Spice

For tracked branches, replace `git push -u origin HEAD`. Choose the flag based on whether a PR already exists:

- `--no-publish` — for branches WITHOUT an existing PR (push code only; the PR is created separately)
- `--update-only` — for branches WITH an existing PR (update remote ref and existing CR metadata, no warning noise)

Without PR:
```bash
git-spice branch submit --no-publish --no-prompt
git-spice branch submit --no-publish --force --no-prompt
```

With existing PR:
```bash
git-spice branch submit --update-only --no-prompt
git-spice branch submit --update-only --force --no-prompt
```

## Commit via Git-Spice

For tracked branches, use `git-spice commit create` instead of `git commit`. This commits and auto-restacks any upstack branches:
```bash
git-spice commit create -m "<message>" --no-prompt
```

With staging:
```bash
git-spice commit create -a -m "<message>" --no-prompt
```

## Amend via Git-Spice

Use `git-spice commit amend` instead of `git commit --amend` + `git-spice upstack restack`. This amends the last commit AND auto-restacks any upstack branches in one atomic operation:
```bash
git-spice commit amend --no-prompt
```

## Squash via Git-Spice

Use `git-spice branch squash` to squash all commits on the current branch into one and auto-restack upstack branches:
```bash
git-spice branch squash --no-prompt
```

With an explicit message:
```bash
git-spice branch squash -m "<message>" --no-prompt
```

## Rebase Conflict Resolution

Use `git-spice rebase continue --no-prompt` (alias `rbc`) after resolving conflicts — auto-restacks upstack branches. To cancel: `git-spice rebase abort` (alias `rba`).

## Branch Split

Use `git-spice branch split` to split a branch into multiple stacked branches at commit boundaries. Useful when commits already map cleanly to logical groups.

Non-interactive (specify split points):
```bash
git-spice branch split --at <last-sha-of-group>:<branch-name> --at <last-sha-of-group>:<branch-name> --no-prompt
```

Each `--at` flag creates a new branch containing commits up to and including that SHA (inclusive boundary — the SHA is the final commit of the group). The final group of commits (those after the last `--at` SHA) remains on the original branch — pass `--at` flags for all groups except the last. The `--at` flag is repeatable. If the split fails due to conflicts, fall back to manual branch creation.

After splitting, rename the original branch if needed:
```bash
git-spice branch rename <old-name> <new-name> --no-prompt
```

## Stack Navigation

Move between branches in a stack. These commands switch the working tree to the target branch:
```bash
git-spice top        # Jump to the highest branch in the stack
git-spice bottom     # Jump to the lowest branch in the stack
git-spice up         # Move one branch up (prompts if multiple upstack branches)
git-spice down       # Move one branch down
```

Checkout a specific branch by name (interactive selection if name omitted):
```bash
git-spice branch checkout <name>
```

## Branch Fold

Use `git-spice branch fold --no-prompt` to merge into base, delete current branch, and restack upstack. This is destructive — always confirm with the user.

## Stack Submit

Push all branches in the stack and create/update PRs with navigation comments. Choose the flag based on whether PRs already exist:

- `--fill` — when creating PRs for branches that don't have them yet
- `--update-only` — when all branches already have PRs (descriptions are written inline)

Creating PRs:
```bash
git-spice stack submit --fill --no-prompt
git-spice stack submit --fill --force --no-prompt
```

Updating existing PRs:
```bash
git-spice stack submit --update-only --no-prompt
git-spice stack submit --update-only --force --no-prompt
```

## CR Discovery

Only needed when PRs are created outside of git-spice (e.g., via GitHub web UI or other tooling). The skill's push operation creates PRs through git-spice directly in Create mode (via `branch submit --title --body`), so CR Discovery is not required in the normal push flow.

After PRs are created externally, git-spice does not know about them. Run submit so git-spice discovers existing CRs and links them internally:

**Single branch:**
```bash
git-spice branch submit --no-prompt
```

**Stack:**
```bash
git-spice stack submit --no-prompt
```

git-spice will log `INF <branch>: Found existing CR #NNN` for each discovered PR. This is idempotent — safe to run even if git-spice already knows about the PRs (it logs "CR #NNN is up-to-date"). These commands also push, which is harmless after a push.md flow since code is already at remote HEAD.
