# PR Writer Delegation Rules

Rules for callers that spawn the `pr-writer` agent.

## Required Fields

- `mode`: `create` or `update`
- `base_branch`: run `fish -c 'gbb'` to detect
- `pr_number`: required for `update` mode

## Commit Message Forwarding

- MUST read all branch commit messages (`git log origin/<base>..HEAD --format=%B`) and pass them verbatim as `commit_messages` in the prompt
- Commit messages are verified ground truth -- they anchor the PR description and prevent hallucination
- After a squash this is a single message; with multiple commits it is the full set

## Optional Context

- `context`: one sentence of motivation -- the "why," not the "what"

## Boundaries

- Do NOT include diff summaries, file lists, change descriptions, pre-drafted PR text, workflow commands, or references to skill/reference files -- the agent gathers its own diff and follows its own rules
- Do NOT write the PR description yourself
- If the agent fails, re-spawn it once -- if it fails again, report the error to the user
