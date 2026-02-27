# PR Writer Delegation Rules

Rules for callers that spawn the `pr-writer` agent.

- Pass `mode` (`create` or `update`), `base_branch` (detected per references/git-patterns.md), and optionally `context` (one sentence of motivation -- the "why," not the "what")
- For `update` mode, also pass `pr_number`
- Do NOT include diff summaries, file lists, change descriptions, pre-drafted PR text, workflow commands, or references to skill/reference files in the prompt -- the agent gathers its own diff and follows its own rules
- If the agent fails, re-spawn it once -- if it fails again, report the error to the user
- Do NOT write the PR description yourself
