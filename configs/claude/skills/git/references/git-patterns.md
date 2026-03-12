# Git Patterns Reference

Shared patterns used across git skill operations. Reference this file for consistent implementation.

## Contents

- Script Paths
- Fish Functions
- Base Branch Detection
- Dotfiles Exception
- Main Branch Protection
- Branch Naming
- Fetch Safety
- Scope Verification
- CI Detection
- CI System Detection
- Branch Context Creation
- Local Fix Commands
- Git-Spice

## Script Paths

- `get-pr-comments` -- `~/.claude/skills/git/scripts/get-pr-comments.sh`
- `get-failed-runs` -- `~/.claude/skills/git/scripts/get-failed-runs.sh`
- `sanitize` -- `~/.claude/skills/git/scripts/sanitize.sh`
- `check-ci` -- `~/.claude/skills/git/scripts/check-ci.sh`
- `rerun` -- `~/.claude/skills/git/scripts/rerun.sh`
- `branch-context-path` -- `~/.claude/skills/git/scripts/branch-context-path.sh`
- `buildkite` -- project-local CI script for querying the Buildkite API. Locate under the project's `.ai/skills/ci/` directory (typically a `.mjs` file). Run via `direnv exec .`. Requires `BUILDKITE_API_TOKEN` env var. Commands: `failed <org> <pipeline> <build>` lists failed jobs; `failed-logs <org> <pipeline> <build>` gets logs for all failed jobs.

## Fish Functions

Custom fish functions (`gbb`, `gwc`, `gwt`) MUST be called via `fish -c '...'` in the Bash tool. They use fish-specific syntax and are not available in bash.

## Base Branch Detection

Run `fish -c 'gbb'` to get the base branch. Accepts an optional branch argument: `fish -c 'gbb [branch]'`.

## Dotfiles Exception

The `dotfiles` repo is special -- direct commits to main are acceptable there.

Detection: Check if the repo path ends with `/dotfiles`:
```bash
[[ "$(git rev-parse --show-toplevel)" == */dotfiles ]]
```

When this applies:
- **Commit operation**: Skip the "create a branch first" prompt when on main
- **push.md**: Push directly to main without PR creation

## Main Branch Protection

Before committing or pushing on main/master:

1. Check if current branch is main or master: `git branch --show-current`
2. If yes, check for dotfiles exception (above)
3. If not dotfiles, ask user before proceeding

## Branch Naming

All new branches MUST use the `sc/` prefix, e.g. `sc/fix-login-redirect`.

- MUST NOT use other prefixes (`scott/`, `gadget--scott/`, etc.)
- When suggesting or creating branch names, MUST use `sc/<kebab-case-slug>`
- The `gwt` fish function adds the `sc/` prefix automatically for worktrees -- MUST NOT add it when calling `gwt`
- This rule applies to all other branch creation or suggestion contexts

## Branch Context File

Resolve the path with:

```bash
~/.claude/skills/git/scripts/branch-context-path.sh
```

This outputs `./tmp/branches/<sanitized-branch>/context.md` (e.g., `./tmp/branches/sc--fix-login/context.md`). The parent directory holds all per-branch artifacts (context, review findings, etc.).

Read this file when it exists and forward its contents as `branch_context` to the pr-writer agent.

**Opt-out sentinel**: If the file contains exactly `N/A` (single line, no other content), the user opted out of providing context. Treat the file as present but empty for routing purposes. pr-writer callers MUST NOT pass `N/A` as `branch_context` -- omit the field instead.

## Fetch Safety

MUST use:
```bash
git fetch origin
```

MUST NOT use:
```bash
git fetch origin <branch>:<branch>  # WRONG - fails if branch is checked out in another worktree
```

After fetching, reference remote branches as `origin/<branch>`.

## Scope Verification

After rebase or before squash, verify the branch only contains expected changes.

```bash
# Show files that will be in the commit
git diff --name-only origin/<base>...HEAD

# Show file stats for human review
git diff --stat origin/<base>...HEAD
```

Use triple-dot (`...`); double-dot shows base-branch changes as deletions when main has advanced.

Ask the user to verify these files match the branch's intended scope. If unexpected files appear:
- Offer to investigate with `git log --oneline origin/<base>..HEAD`
- Offer to fix with `git rebase -i origin/<base>`

## Downstream PR Safety

Before any force push, check for open PRs that target the current branch as their base:

```bash
gh pr list --base "$(git branch --show-current)" --state open --json number,title,headRefName --jq '.[]'
```

If any exist, warn the user and present options via AskUserQuestion:
- "Update their bases first" — for each downstream PR, run `gh pr edit <number> --base <new-base>` where `<new-base>` is this branch's own base (detected per Base Branch Detection above); then proceed with the force push
- "Force push anyway" — proceed without updating bases (risk: GitHub may auto-merge downstream PRs)
- "Abort" — stop the operation

This check applies to ANY force push — whether from Push, Squash, Amend, Rebase, or ad-hoc operations. It does NOT apply to regular pushes (non-force), since those only add commits and cannot cause auto-merges.

## CI Detection

Use these two steps to verify CI is configured and check status. Referenced by fix.md.

**Step 1 -- Verify CI is configured**: Run `gh pr checks --json name,state 2>/dev/null` (or `gh run list --branch $(git branch --show-current) --limit 1` if no PR exists). If the command returns no check runs and no runs exist, inform the user that no CI checks were found and stop.

**Step 2 -- Check CI status**:
- If a PR exists (`gh pr view --json number,url 2>/dev/null`): run `gh pr checks`
- Otherwise check branch runs: run `gh run list --branch $(git branch --show-current) --limit 5`
- Group checks by status (failed, pending, passed)

## CI System Detection

Detect CI system by checking `.github/workflows/` (github-actions) or `.buildkite/` (buildkite). `gh pr checks` works for all systems; `gh run list` / `gh run view` / `get-failed-runs` / `ci-triager` only work for `github-actions`.

## Branch Context Creation

Read or create the branch context file that captures the "why" for the current branch.

**Cross-check rule**: If the draft includes factual claims about before/after states, verify them against `git diff origin/<base>...HEAD` -- the diff is the source of truth. If the diff contradicts a claim, correct the draft to match the diff and note the correction to the user.

1. **Check branch**: If on main/master, inform the user that branch context is for feature branches and stop.

2. **Check for existing file**: Check if the branch context file exists (path per "Branch Context File" above). If it exists, display its contents and ask if the user wants to update it. If they decline, stop.

3. **Assess conversation context**: Before prompting the user, assess whether the current conversation already contains enough information to draft a meaningful branch context -- problem discussed, motivation clear, relevant links shared. Also assess whether the branch involves multiple distinct concerns visible in the conversation (e.g., docs site, CI workflow, tooling rules, config removal). A single-sentence summary that omits known concerns is NOT "sufficient" -- the context must cover all known concerns. If the conversation lacks sufficient context (e.g., invoked at the start of a session with no prior discussion), fall through to step 5.

4. **Draft from conversation**: If context is sufficient (step 3 passed), draft a branch context: 1-3 sentences of purpose/motivation, related links if discussed, no headers/change lists/implementation details. If the conversation reveals multiple distinct concerns (e.g., docs site + CI workflow + tooling rules + config removal), enumerate each concern as a separate sentence or bullet -- the pr-writer and committer use this to decide whether to structure output as prose or numbered lists. Follow the format constraints in step 7. Then skip to step 7 (write the file).

5. **Gather context**: Prompt via AskUserQuestion -- "What's the purpose of this branch?" with exactly these two options (MUST NOT substitute domain-specific alternatives -- they are intentionally domain-agnostic so they work consistently across all repos and contexts). The free-text input ("Type something...") serves as the direct-entry path -- no separate option is needed for it.
   - **"Help me articulate it"** -- proceed to step 6.
   - **"Skip"** -- write `N/A` to the branch context file and skip to step 9 (skip confirmation).
   If the user provides free text, treat it as the purpose. Optionally ask "Any related links (issues, PRs, Slack)?" with a "Skip" option.

6. **Ask targeted questions**: Ask via AskUserQuestion: "What problem are you solving or what triggered this work?". Then ask "What's the expected outcome when this branch merges?". Then ask "Any related issues, PRs, or links?" with a "Skip" option. Synthesize the answers into a concise purpose statement (1-3 sentences) plus any links provided.

7. **Write the file**: Create the branch context file at the path from `~/.claude/skills/git/scripts/branch-context-path.sh`. The file MUST contain only:
   - 1-3 sentences of purpose/motivation (the "why")
   - Related links, if given (each on its own line)

   Lead with the problem or trigger, not the solution. "CONTRIBUTING.md mixed audiences and couldn't scale" is why; "Add a Starship docs site" is what. If the branch includes changes with separate motivations (e.g., a cleanup alongside a feature), mention each motivation -- the reader needs to understand why the diff touches seemingly unrelated areas. For branches with 3 or more distinct concerns, each concern MUST be its own sentence or bullet -- do NOT compress them into a single compound sentence with semicolons. This gives downstream tools (pr-writer, committer) explicit concern boundaries to work from. Do NOT include headers, change lists, implementation details, or what files were modified -- the diff is the source of truth for "what". Keep the user's original phrasing where possible.

8. **Confirm with user**: Show the written content and ask via AskUserQuestion -- "Does this accurately capture the purpose?" with options:
   - **"Looks good"** -- proceed to report.
   - **"Needs changes"** -- user provides corrections; update the file and re-confirm.

9. **Report**: Confirm the file was written and show its contents.

## Local Fix Commands

Detect language from the repository root and use the appropriate commands. Subagents should also consult the project's CLAUDE.md for project-specific commands.

### Node.js (`package.json` exists)
- **Lint fix**: `pnpm run lint:fix` (if the script exists in package.json)
- **Test**: `pnpm test`

### Go (`go.mod` exists)
- **Lint fix**: if `.envrc` exists, `direnv exec . fmt`; otherwise `golangci-lint run --fix ./...`
- **Test**: if `.envrc` exists, `direnv exec . tests -short ./...`; otherwise `go test -short ./...`

### Rust (`Cargo.toml` exists)
- **Lint fix**: `cargo clippy --fix --allow-dirty && cargo fmt`
- **Test**: `cargo test`

### Fallback
If none of the above match, skip automated lint fixing and instruct the subagent to check for project-specific tooling.

## Git-Spice

git-spice (`gs`) manages stacked branches — tracking topology, restacking after rebases, and submitting stacked PRs with navigation comments.

### Detection

Check if the repo is initialized:
```bash
git rev-parse --verify refs/spice/data 2>/dev/null
```
Succeeds (exit 0) if git-spice is initialized for this repo.

### Initialization

Only auto-initialize during stacking workflows (Split, Stack). Other operations (Push, Rebase, Squash) check but do not init.

```bash
gs repo init --trunk <base> --remote origin --no-prompt
git config spice.branchCreate.prefix sc/
```

The `sc/` prefix maintains the existing branch naming convention.

### Tracked Branch Check

Check if a branch is tracked by git-spice:
```bash
gs log short 2>&1 | grep -q '<branch>'
```
Exit 0 means the branch appears in git-spice's tracked stack. This is read-only — it does not switch branches. Note: `gs log short` requires `2>&1` (not `2>/dev/null`) because `gs` writes to stderr or uses tty detection that suppresses output under plain redirects.

### Push via Git-Spice

For tracked branches, replace `git push -u origin HEAD`:
```bash
gs branch submit --no-publish --no-prompt
```

Force push:
```bash
gs branch submit --no-publish --force --no-prompt
```

`--no-publish` skips PR creation (pr-writer handles that separately).

### Stack Submit

Push all branches in the stack and create/update PRs with navigation comments:
```bash
gs stack submit --no-prompt
```

### Restack

Rebase current branch and all above it onto their bases:
```bash
gs upstack restack
```

### Navigation

```bash
gs up        # move to the branch above
gs down      # move to the branch below
gs top       # move to the top of the stack
gs bottom    # move to the bottom of the stack
gs trunk     # move to the trunk branch
```

### Sync

Fetch, clean merged branches, and restack:
```bash
gs repo sync --restack --no-prompt
```

### Branch Reorder

Move branches to a new position in the stack using these commands:

- `gs upstack onto <destination> --no-prompt` (alias: `gs uso`) — Move the current branch AND all branches above it to a new base. Use when reordering within a stack.
- `gs branch onto <destination> --no-prompt` (alias: `gs bon`) — Move ONLY the current branch to a new base, leaving upstack branches where they are. Use when extracting a branch from the stack.
- `gs stack edit` (alias: `gs se`) — Open an editor to reorder branches in a linear stack. Requires the stack to be linear (no branch can have multiple branches above it).

### Non-Interactive Rule

MUST always pass `--no-prompt` to any `gs` command that accepts it, to avoid hanging on interactive prompts.

### Command Name

Use `gs` (the installed binary at `/opt/homebrew/bin/gs`). The fish config has `alias git-spice=/opt/homebrew/bin/gs` for interactive use, but `gs` works directly in bash.
