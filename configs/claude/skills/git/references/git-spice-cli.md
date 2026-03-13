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
