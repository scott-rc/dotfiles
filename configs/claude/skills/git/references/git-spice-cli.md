# git-spice CLI Reference

Quick reference for git-spice commands used by the git skill. Full docs: https://abhinav.github.io/git-spice/ — LLM-friendly full reference: https://abhinav.github.io/git-spice/llms-full.txt

## Repository

- `git-spice repo init [--trunk BRANCH] [--remote NAME] [--reset] [--no-prompt]` — Initialize repo
- `git-spice repo sync [--restack] [--no-prompt]` — Pull latest, delete merged branches

## Branch

- `git-spice branch create [name] [--insert] [--below] [-t BRANCH] [-a] [-m MSG] [--no-commit] [--signoff] [--no-prompt]` — Create tracked branch
- `git-spice branch track [branch] [-b BASE] [--base BASE] [--no-prompt]` — Track existing branch
- `git-spice branch untrack [branch]` — Remove from tracking
- `git-spice branch checkout [branch] [-n] [--detach] [-u]` — Switch to tracked branch
- `git-spice branch delete [branches...] [--force] [--no-prompt]` — Delete and restack upstack
- `git-spice branch rename [old [new]]` — Rename tracked branch
- `git-spice branch restack [--branch NAME] [--no-prompt]` — Rebase onto base
- `git-spice branch edit` — Interactive rebase of commits in branch
- `git-spice branch split [--at COMMIT:NAME] [--branch NAME]` — Split branch at commits
- `git-spice branch squash [-m MSG] [--no-edit] [--no-verify] [--branch NAME] [--no-prompt]` — Squash all commits
- `git-spice branch fold [--branch NAME]` — Merge into base and delete
- `git-spice branch onto [onto] [--branch NAME]` — Move to different base (alias: `git-spice bon`)
- `git-spice branch diff [--branch NAME]` — Show diff from base
- `git-spice branch submit [--fill] [--draft|--no-draft] [--publish|--no-publish] [-w] [--force] [--no-verify] [-l LABEL] [-r REVIEWER] [-a ASSIGNEE] [--title T] [--body B] [--no-prompt]` — Create/update PR

## Stack

- `git-spice stack submit [--fill] [--draft|--no-draft] [-w] [--update-only|--no-update-only] [-l LABEL] [-r REVIEWER] [-a ASSIGNEE] [--no-prompt]` — Submit entire stack
- `git-spice stack restack [--branch NAME] [--no-prompt]` — Restack all branches in stack
- `git-spice stack edit [--editor STRING] [--branch NAME]` — Edit branch order (alias: `git-spice se`)
- `git-spice stack delete [--force] [--no-prompt]` — Delete all branches in stack

## Navigation

- `git-spice up` — Branch above (prompts if multiple)
- `git-spice down` — Branch below
- `git-spice top` — Topmost branch (prompts if multiple)
- `git-spice bottom` — Bottommost branch (above trunk)
- `git-spice trunk` — Trunk branch

## Upstack

- `git-spice upstack submit [--branch NAME] [--no-prompt]` — Submit current + above
- `git-spice upstack restack [--skip-start] [--branch NAME] [--no-prompt]` — Restack current + above
- `git-spice upstack onto [onto] [--branch NAME] [--no-prompt]` — Move branch + above to new base (alias: `git-spice uso`)
- `git-spice upstack delete [--force]` — Delete all above

## Downstack

- `git-spice downstack submit [--branch NAME] [--no-prompt]` — Submit current + below
- `git-spice downstack track [branch]` — Track all below
- `git-spice downstack edit [--editor STRING] [--branch NAME]` — Edit order below

## Commit

- `git-spice commit create [-m MSG] [-a] [--no-verify] [--no-prompt]` — Commit and restack upstack (alias: `git-spice cc`)
- `git-spice commit amend [-m MSG] [-a/--all] [--no-verify] [--no-prompt]` — Amend last commit and auto-restack upstack branches (alias: `git-spice ca`)
- `git-spice commit split` — Split last commit and restack

## Rebase

Prefer these over `git rebase --continue/--abort` — they auto-restack upstack branches after resuming.

- `git-spice rebase continue [--no-prompt]` — Resume a paused rebase and auto-restack upstack branches (alias: `git-spice rbc`)
- `git-spice rebase abort` — Cancel the current rebase and restore pre-rebase state (alias: `git-spice rba`)

## Logging

- `git-spice log short [-a] [-S|--no-cr-status] [--json]` — List branches in stack
- `git-spice log long [-a] [-S|--no-cr-status] [--json]` — List branches and commits

## Authentication

- `git-spice auth login [--refresh]` — Authenticate with forge
- `git-spice auth status` — Show login status
- `git-spice auth logout` — Remove auth

## Configuration

Key git config options:
- `spice.branchCreate.prefix` — Prefix for generated branch names
- `spice.branchCreate.commit` — Default commit behavior on create
- `spice.submit.draft` — Mark new CRs as draft
- `spice.submit.navigationComment` — Include navigation comment in CRs
- `spice.submit.label` — Default labels for CRs
- `spice.repoSync.closedChanges` — Handle closed CRs (ignore/prompt)
- `spice.log.all` — Show all branches by default
- `spice.log.crStatus` — Request CR status in log

## Global Flags

All commands support: `--no-prompt` (disable interactive prompts), `-v/--verbose`, `-C/--dir DIR`.
