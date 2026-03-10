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

## Script Paths

- `get-pr-comments` -- `~/.claude/skills/git/scripts/get-pr-comments.sh`
- `get-failed-runs` -- `~/.claude/skills/git/scripts/get-failed-runs.sh`
- `sanitize` -- `~/.claude/skills/git/scripts/sanitize.sh`
- `check-ci` -- `~/.claude/skills/git/scripts/check-ci.sh`
- `rerun` -- `~/.claude/skills/git/scripts/rerun.sh`
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

Path: `./tmp/branches/<sanitized-branch>/context.md` where the branch name is sanitized by replacing `/` with `--` (e.g., `sc/fix-login` becomes `sc--fix-login`):

```bash
branch=$(git rev-parse --abbrev-ref HEAD | sed 's|/|--|g')
```

The `./tmp/branches/<sanitized-branch>/` directory holds all per-branch artifacts (context, review findings, etc.).

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

1. **Check branch**: If on main/master, inform the user that branch context is for feature branches and stop.

2. **Check for existing file**: Check if the branch context file exists (path per "Branch Context File" above). If it exists, display its contents and ask if the user wants to update it. If they decline, stop.

3. **Assess conversation context**: Before prompting the user, assess whether the current conversation already contains enough information to draft a meaningful branch context -- problem discussed, motivation clear, relevant links shared. If the conversation lacks sufficient context (e.g., invoked at the start of a session with no prior discussion), fall through to step 5.

4. **Draft from conversation**: If context is sufficient (step 3 passed), draft a branch context: 1-3 sentences of purpose/motivation, related links if discussed, no headers/change lists/implementation details. Cross-check any factual claims about before/after states against `git diff origin/<base>...HEAD` -- the diff is the source of truth for what the code looked like. Then skip to step 7 (write the file).

5. **Gather context**: Prompt via AskUserQuestion -- "What's the purpose of this branch?" with exactly these two options (MUST NOT substitute domain-specific alternatives -- they are intentionally domain-agnostic so they work consistently across all repos and contexts). The free-text input ("Type something...") serves as the direct-entry path -- no separate option is needed for it.
   - **"Help me articulate it"** -- proceed to step 6.
   - **"Skip"** -- write `N/A` to the branch context file and skip to step 9 (skip confirmation).
   If the user provides free text, treat it as the purpose. Optionally ask "Any related links (issues, PRs, Slack)?" with a "Skip" option. If their description includes factual claims about before/after states, cross-check against the diff before writing.

6. **Ask targeted questions**: Ask via AskUserQuestion: "What problem are you solving or what triggered this work?". Then ask "What's the expected outcome when this branch merges?". Then ask "Any related issues, PRs, or links?" with a "Skip" option. Synthesize the answers into a concise purpose statement (1-3 sentences) plus any links provided. Cross-check any factual claims about before/after states against `git diff origin/<base>...HEAD`.

7. **Write the file**: Create the branch context file (`./tmp/branches/<sanitized-branch>/context.md`). The file MUST contain only:
   - 1-3 sentences of purpose/motivation (the "why")
   - Related links, if given (each on its own line)

   Do NOT include headers, change lists, implementation details, or what files were modified -- the diff is the source of truth for "what". Keep the user's original phrasing where possible.

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

