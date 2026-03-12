# PR Writer Delegation Rules

Rules for callers that spawn the `pr-writer` agent.

## Delegation Fields

Canonical payload for spawning the `pr-writer` agent. Pass these fields in the task prompt:

- `mode`: `create` or `update`
- `base_branch`: run `fish -c 'gbb'` to detect
- `pr_number`: required for `update` mode
- `commit_messages`: all branch commit messages verbatim (see Commit Message Forwarding below)
- `branch_context` (optional): contents of the branch context file (path per references/git-patterns.md "Branch Context File") -- purpose statement and links captured at branch creation. When present, this is the primary source for the PR's motivation/narrative; commit messages remain supplementary hints.
- `context` (optional): one sentence describing what changed in this particular update. When `branch_context` is present, do NOT restate the branch purpose here -- instead describe what's new (e.g., "addresses review feedback on error handling", "adds test coverage for edge cases"). If nothing substantive changed (e.g., post-squash cleanup), omit this field entirely.

## Commit Message Forwarding

- MUST read all branch commit messages (`git log origin/<base>..HEAD --format=%B`) and pass them verbatim as `commit_messages` in the prompt
- Commit messages are supplementary hints -- they provide context but are NOT the source of truth; the diff is
- Commit messages often describe intermediate states (fixups, reverts, mid-PR bugs that were later corrected) that MUST NOT appear in the PR description
- After a squash this is a single message; with multiple commits it is the full set

## Stacked PR Batch Updates

When updating descriptions for multiple PRs in a stack:

- `pr-writer` agents MAY be spawned in parallel -- the agent uses PR-specific temp file paths and resolves the head branch from the PR number (not `HEAD`), so parallel execution is safe (see also: references/github-text.md Concurrent Agents for the underlying temp-file uniqueness rule)
- After all agents complete, the caller MUST verify descriptions are distinct: fetch titles and first body lines via `gh pr view <number> --json title,body` for each PR and confirm they are not identical
- If duplicates are found, re-spawn the affected agents sequentially

## Boundaries

- MUST NOT include diff summaries, file lists, change descriptions, pre-drafted PR text, workflow commands, or references to skill/reference files -- the agent gathers its own diff and follows its own rules
- MUST NOT write the PR description yourself
- If the agent fails, re-spawn it once -- if it fails again, report the error to the user
