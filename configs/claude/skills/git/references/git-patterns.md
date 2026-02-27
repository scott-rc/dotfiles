# Git Patterns Reference

Shared patterns used across git skill operations. Reference this file for consistent implementation.

## Script Paths

- `get-pr-comments` -- `~/.claude/skills/git/scripts/get-pr-comments.sh`
- `poll-pr-status` -- `~/.claude/skills/git/scripts/poll-pr-status.sh`
- `get-failed-runs` -- `~/.claude/skills/git/scripts/get-failed-runs.sh`
- `buildkite` -- project-local CI script for querying the Buildkite API. Locate under the project's `.ai/skills/ci/` directory (typically a `.mjs` file). Run via `direnv exec .`. Requires `BUILDKITE_API_TOKEN` env var. Commands: `failed <org> <pipeline> <build>` lists failed jobs; `failed-logs <org> <pipeline> <build>` gets logs for all failed jobs.

## Fish Functions

Custom fish functions (`gbb`, `gwc`, `gwt`) MUST be called via `fish -c '...'` in the Bash tool. They use fish-specific syntax and are not available in bash.

## Base Branch Detection

Run `fish -c 'gbb'` to get the base branch. Returns the nearest fork-point branch (handles stacked branches), falling back to the default branch (`main`/`master`). Accepts an optional branch argument: `fish -c 'gbb [branch]'`.

## Dotfiles Exception

The `dotfiles` repo is special -- direct commits to main are acceptable there.

Detection: Check if the repo path ends with `/dotfiles`:
```bash
[[ "$(git rev-parse --show-toplevel)" == */dotfiles ]]
```

When this applies:
- **Commit operation**: Skip the "create a branch first" prompt when on main
- **push.md**: Push directly to main without PR creation
- **worktree.md**: When in dotfiles repo, scan other repositories instead
- **clean-worktrees.md**: Exclude dotfiles repo from cleanup scans

## Main Branch Protection

Before committing or pushing on main/master:

1. Check if current branch is main or master: `git branch --show-current`
2. If yes, check for dotfiles exception (above)
3. If not dotfiles, ask user before proceeding

## Branch Naming

All new branches MUST use the `sc/` prefix, e.g. `sc/fix-login-redirect`.

- Do NOT use other prefixes (`scott/`, `gadget--scott/`, etc.)
- When suggesting or creating branch names, MUST use `sc/<kebab-case-slug>`
- The `gwt` fish function adds the `sc/` prefix automatically for worktrees -- do not add it when calling `gwt`
- This rule applies to all other branch creation or suggestion contexts

## Fetch Safety

Always use:
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

**Note**: These comparisons assume the branch has been rebased onto `origin/<base>`. If the branch has diverged (main advanced since the branch was created), the diff will include the reversal of main's changes. Rebase first: `git rebase origin/<base>`

```bash
# Show files that will be in the commit
git diff --name-only origin/<base> HEAD

# Show file stats for human review
git diff --stat origin/<base> HEAD
```

Ask the user to verify these files match the branch's intended scope. If unexpected files appear:
- Offer to investigate with `git log --oneline origin/<base>..HEAD`
- Offer to fix with `git rebase -i origin/<base>`

## CI System Detection

CI system is detected automatically by `poll-pr-status` and reported in `ci.ciSystem`. For `fix-ci.md` standalone use, detect by checking `.github/workflows/` (github-actions) or `.buildkite/` (buildkite). `gh pr checks` works for all systems; `gh run list` / `gh run view` / `get-failed-runs` / `ci-triager` only work for `github-actions`.

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

## Bulk Thread Handling

When there are 5+ review threads to process, spawn a Task subagent (type: Explore, model: sonnet) to read all referenced files upfront and return a concise per-thread context summary. This avoids loading many files inline and keeps the orchestrator's context small.

### For review.md

Instruct the Explore subagent to: read each referenced file and its surrounding context (10-20 lines around each thread location), then return a per-thread summary with: file path, line range, current code at the thread location, and any immediately relevant context (function signature, containing block, etc.).

### For reply.md

Instruct the Explore subagent to: read each referenced file and check `git diff` for relevant changes at each thread location, then return a per-thread summary with: file path, line range, current code at the thread location, and whether the code was changed since the review comment (with a brief description of the change if so).
