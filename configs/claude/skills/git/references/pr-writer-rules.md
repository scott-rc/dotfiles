# PR Writer Delegation Rules

Rules for callers that spawn the `pr-writer` agent.

## Delegation Fields

Canonical payload for spawning the `pr-writer` agent. Pass these fields in the task prompt:

- `mode`: `create` or `update`
- `base_branch`: run `fish -c 'gbb'` to detect
- `pr_number`: required for `update` mode
- `commit_messages`: all branch commit messages verbatim (see Commit Message Forwarding below)
- `branch_context` (optional): contents of the branch context file (`tmp/branches/<sanitized-branch>.md`, sanitize: replace `/` with `--`) -- purpose statement and links captured at branch creation. When present, this is the primary source for the PR's motivation/narrative; commit messages remain supplementary hints.
- `context` (optional): one sentence describing what changed in this particular update. When `branch_context` is present, do NOT restate the branch purpose here -- instead describe what's new (e.g., "addresses review feedback on error handling", "adds test coverage for edge cases"). If nothing substantive changed (e.g., post-squash cleanup), omit this field entirely.

## Commit Message Forwarding

- MUST read all branch commit messages (`git log origin/<base>..HEAD --format=%B`) and pass them verbatim as `commit_messages` in the prompt
- Commit messages are supplementary hints -- they provide context but are NOT the source of truth; the diff is
- Commit messages often describe intermediate states (fixups, reverts, mid-PR bugs that were later corrected) that MUST NOT appear in the PR description
- After a squash this is a single message; with multiple commits it is the full set

## Boundaries

- Do NOT include diff summaries, file lists, change descriptions, pre-drafted PR text, workflow commands, or references to skill/reference files -- the agent gathers its own diff and follows its own rules
- Do NOT write the PR description yourself
- If the agent fails, re-spawn it once -- if it fails again, report the error to the user
