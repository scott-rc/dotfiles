# Git Patterns Reference

Shared patterns used across git skill operations. Reference this file for consistent implementation.

## Contents

- Script Paths
- Fish Functions
- Base Branch Detection
- Dotfiles Exception
- Main Branch Protection
- Branch Naming
- Branch Context File
- Fetch Safety
- Scope Verification
- Downstream PR Safety
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
- `branch-context-path` -- `~/.claude/skills/git/scripts/branch-context-path.sh`
- `buildkite` -- project-local CI script for querying the Buildkite API. Locate under the project's `.ai/skills/ci/` directory (typically a `.mjs` file). Run via `direnv exec .`. Requires `BUILDKITE_API_TOKEN` env var. Commands: `failed <org> <pipeline> <build>` lists failed jobs; `failed-logs <org> <pipeline> <build>` gets logs for all failed jobs.

## Fish Functions

Custom fish functions (`gwc`, `gwt`) MUST be called via `fish -c '...'` in the Bash tool. They use fish-specific syntax and are not available in bash.

## Base Branch Detection

1. **Read base from git-spice JSON**:
   ```bash
   git-spice log short --json 2>/dev/null | jq -r --arg branch "$(git branch --show-current)" 'select(.name == $branch) | .down.name'
   ```
   If the command fails, follow the Error Recovery pattern from references/git-spice-patterns.md and retry. Once the branch is tracked, `.down.name` is populated. This is the authoritative base for both regular and stacked branches. If `.down.name` is null or empty (e.g., trunk branch), fall back to `git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||'` with default `main`.

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

Read this file when it exists and use its contents as `branch_context` when writing PR descriptions.

**Opt-out sentinel**: If the file contains exactly `N/A` (single line, no other content), the user opted out of providing context. Treat the file as present but empty for routing purposes. MUST NOT use `N/A` as `branch_context` when writing PR descriptions -- omit the field instead.

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

### Trunk Sync

Run immediately after every `git fetch origin` and before any rebase or restack. The local trunk ref may be behind `origin/<trunk>` — especially in worktrees where trunk is rarely checked out. A stale local trunk causes git-spice restack to include already-merged commits in the branch diff.

```bash
TRUNK=$(git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||')
TRUNK=${TRUNK:-main}
if ! git rev-parse --verify refs/heads/$TRUNK &>/dev/null; then
  git update-ref refs/heads/$TRUNK origin/$TRUNK
elif git merge-base --is-ancestor refs/heads/$TRUNK origin/$TRUNK 2>/dev/null; then
  git update-ref refs/heads/$TRUNK origin/$TRUNK
fi
```

If the local trunk ref doesn't exist (common in worktrees), it's created unconditionally. If it exists, `merge-base --is-ancestor` ensures only fast-forward (no local-only trunk commits lost). `update-ref` doesn't touch the working tree, so it's safe when trunk is checked out in another worktree.

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
- Offer to fix with `git-spice branch edit` (interactive rebase scoped to the current branch's commits)

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

**Same-stack exception**: When force-pushing via `git-spice stack submit --force`, git-spice manages the entire stack atomically — all branches are pushed together and their base refs are updated. Downstream PRs within the same git-spice stack are safe because git-spice restacks them automatically. The safety check still applies to downstream PRs that are NOT part of the current git-spice stack (e.g., other developers' PRs targeting your branch).

## CI Detection

Use these two steps to verify CI is configured and check status. Referenced by fix.md.

**Step 1 -- Verify CI is configured**: Run `gh pr checks --json name,state 2>/dev/null` (or `gh run list --branch $(git branch --show-current) --limit 1` if no PR exists). If the command returns no check runs and no runs exist, inform the user that no CI checks were found and stop.

**Step 2 -- Check CI status**:
- If a PR exists (`gh pr view --json number,url 2>/dev/null`): run `gh pr checks`
- Otherwise check branch runs: run `gh run list --branch $(git branch --show-current) --limit 5`
- Group checks by status (failed, pending, passed)

## CI System Detection

Detect CI system by checking `.github/workflows/` (github-actions) or `.buildkite/` (buildkite). `gh pr checks` works for all systems; `gh run list` / `gh run view` / `get-failed-runs` only work for `github-actions`.

## Branch Context Creation

Read or create the branch context file that captures the "why" for the current branch.

**Cross-check rule**: If the draft includes factual claims about before/after states, verify them against `git diff origin/<base>...HEAD` -- the diff is the source of truth. If the diff contradicts a claim, correct the draft to match the diff and note the correction to the user.

1. **Check branch**: If on main/master, inform the user that branch context is for feature branches and stop.

2. **Check for existing file**: Check if the branch context file exists (path per "Branch Context File" above). If it exists, display its contents and ask if the user wants to update it. If they decline, stop.

3. **Assess conversation context**: Before prompting the user, assess whether the current conversation already contains enough information to draft a meaningful branch context -- problem discussed, motivation clear, relevant links shared. Context is "sufficient" when you can answer *why* this branch exists and *what problem it solves* from information already in the conversation. Sources that count: spec/design docs read earlier, implementation work done in-session, user descriptions of the problem, commit messages, referenced issues or PRs. Reading a spec file and implementing from it provides full context — do not fall through to step 5 just because the user didn't explicitly state the motivation in chat. Also assess whether the branch involves multiple distinct concerns visible in the conversation (e.g., docs site, CI workflow, tooling rules, config removal). A single-sentence summary that omits known concerns is NOT "sufficient" -- the context must cover all known concerns. If the conversation lacks sufficient context (e.g., invoked at the start of a session with no prior discussion and no spec file), fall through to step 5.

4. **Draft from conversation**: If context is sufficient (step 3 passed), draft a branch context: 1-3 sentences of purpose/motivation, related links if discussed, no headers/change lists/implementation details. If the conversation reveals multiple distinct concerns (e.g., docs site + CI workflow + tooling rules + config removal), enumerate each concern as a separate sentence or bullet -- downstream operations use this to decide whether to structure output as prose or numbered lists. Follow the format constraints in step 6. Then skip to step 6 (write the file).

5. **Gather context**: Prompt via AskUserQuestion with a single combined prompt. The header is "Branch Context" and the question is "What's the purpose of this branch?". Include exactly these options (MUST NOT substitute domain-specific alternatives -- they are intentionally domain-agnostic so they work consistently across all repos and contexts):
   - **"Skip"** -- write `N/A` to the branch context file and skip to step 8 (skip confirmation).
   - **"Help me articulate it"** -- pre-fill answers from whatever IS available in the conversation (spec files, commit messages, diff stats) and present them as a single AskUserQuestion with three questions: "What problem are you solving or what triggered this work?" (pre-fill the best guess as the first option, with "Something else" as second), "What's the expected outcome when this branch merges?" (pre-fill if inferable, with "Something else" as second), and "Any related issues, PRs, or links?" (with "Skip" and "Yes" options). This MUST be a single AskUserQuestion call with all three questions, not sequential prompts.

   The free-text input ("Type something...") serves as the direct-entry path -- no separate option is needed for it. If the user provides free text, treat it as the purpose. Synthesize all answers into a concise purpose statement (1-3 sentences) plus any links provided.

6. **Write the file**: Create the branch context file at the path from `~/.claude/skills/git/scripts/branch-context-path.sh`. The file MUST contain only:
   - 1-3 sentences of purpose/motivation (the "why")
   - Related links, if given (each on its own line)

   Lead with the problem or trigger, not the solution. "CONTRIBUTING.md mixed audiences and couldn't scale" is why; "Add a Starship docs site" is what. If the branch includes changes with separate motivations (e.g., a cleanup alongside a feature), mention each motivation -- the reader needs to understand why the diff touches seemingly unrelated areas. For branches with 3 or more distinct concerns, each concern MUST be its own sentence or bullet -- do NOT compress them into a single compound sentence with semicolons. This gives downstream operations explicit concern boundaries to work from. Do NOT include headers, change lists, implementation details, or what files were modified -- the diff is the source of truth for "what". Keep the user's original phrasing where possible.

7. **Confirm with user**: Show the written content and ask via AskUserQuestion -- "Does this accurately capture the purpose?" with options:
   - **"Looks good"** -- proceed to report.
   - **"Needs changes"** -- user provides corrections; update the file and re-confirm.

8. **Report**: Confirm the file was written and show its contents.

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
If none of the above match, skip automated lint fixing and check for project-specific tooling.
