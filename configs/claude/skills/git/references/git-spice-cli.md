# git-spice CLI Reference

Official docs — overview: https://abhinav.github.io/git-spice/llms.txt — full reference: https://abhinav.github.io/git-spice/llms-full.txt

Use the official docs for CLI syntax, flags, and configuration options. This file covers only skill-specific conventions.

## Conventions

### Non-Interactive Rule

MUST always pass `--no-prompt` to any `git-spice` command that accepts it, to avoid hanging on interactive prompts.

### Branch Prefix

Set `spice.branchCreate.prefix` to `sc/` during initialization:
```
git config spice.branchCreate.prefix sc/
```
This maintains the `sc/` branch naming convention used across the skill.

### Command Name

Use `git-spice` (the installed binary). The fish config has `alias gs=git-spice` for interactive convenience, but `git-spice` is the canonical binary name and MUST be used in skill operations.

## Notable Features

### JSON Output (v0.18+)

`git-spice log short --json` streams JSONL (one JSON object per line) with branch metadata. Used by the skill for tracked branch checks, PR existence detection, and divergence detection. See the operation files for schema details and jq recipes.

### --update-only (v0.10+)

Available on `branch submit` and `stack submit`. Updates existing CRs only — skips branches without CRs. Use `--update-only` when a PR exists; use `--no-publish` when it does not.

### repo restack (v0.16+)

`git-spice repo restack` rebases ALL tracked branches in dependency order (not just the current stack). Used by `repo sync --restack`.

