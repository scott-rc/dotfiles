# git-spice CLI Reference

Quick reference for git-spice (`gs`) commands used by the git skill. Full docs: https://abhinav.github.io/git-spice/ — LLM-friendly full reference: https://abhinav.github.io/git-spice/llms-full.txt

## Repository

- `gs repo init [--trunk BRANCH] [--remote NAME] [--reset] [--no-prompt]` — Initialize repo
- `gs repo sync [--restack] [--no-prompt]` — Pull latest, delete merged branches

## Branch

- `gs branch create [name] [--insert] [--below] [-t BRANCH] [-a] [-m MSG] [--no-commit] [--signoff] [--no-prompt]` — Create tracked branch
- `gs branch track [branch] [-b BASE] [--base BASE] [--no-prompt]` — Track existing branch
- `gs branch untrack [branch]` — Remove from tracking
- `gs branch checkout [branch] [-n] [--detach] [-u]` — Switch to tracked branch
- `gs branch delete [branches...] [--force] [--no-prompt]` — Delete and restack upstack
- `gs branch rename [old [new]]` — Rename tracked branch
- `gs branch restack [--branch NAME] [--no-prompt]` — Rebase onto base
- `gs branch edit` — Interactive rebase of commits in branch
- `gs branch split [--at COMMIT:NAME] [--branch NAME]` — Split branch at commits
- `gs branch squash [-m MSG] [--no-edit] [--no-verify] [--branch NAME] [--no-prompt]` — Squash all commits
- `gs branch fold [--branch NAME]` — Merge into base and delete
- `gs branch onto [onto] [--branch NAME]` — Move to different base (alias: `gs bon`)
- `gs branch diff [--branch NAME]` — Show diff from base
- `gs branch submit [--fill] [--draft|--no-draft] [--publish|--no-publish] [-w] [--force] [--no-verify] [-l LABEL] [-r REVIEWER] [-a ASSIGNEE] [--title T] [--body B] [--no-prompt]` — Create/update PR

## Stack

- `gs stack submit [--fill] [--draft|--no-draft] [-w] [--update-only|--no-update-only] [-l LABEL] [-r REVIEWER] [-a ASSIGNEE] [--no-prompt]` — Submit entire stack
- `gs stack restack [--branch NAME] [--no-prompt]` — Restack all branches in stack
- `gs stack edit [--editor STRING] [--branch NAME]` — Edit branch order (alias: `gs se`)
- `gs stack delete [--force] [--no-prompt]` — Delete all branches in stack

## Navigation

- `gs up` — Branch above (prompts if multiple)
- `gs down` — Branch below
- `gs top` — Topmost branch (prompts if multiple)
- `gs bottom` — Bottommost branch (above trunk)
- `gs trunk` — Trunk branch

## Upstack

- `gs upstack submit [--branch NAME] [--no-prompt]` — Submit current + above
- `gs upstack restack [--skip-start] [--branch NAME] [--no-prompt]` — Restack current + above
- `gs upstack onto [onto] [--branch NAME] [--no-prompt]` — Move branch + above to new base (alias: `gs uso`)
- `gs upstack delete [--force]` — Delete all above

## Downstack

- `gs downstack submit [--branch NAME] [--no-prompt]` — Submit current + below
- `gs downstack track [branch]` — Track all below
- `gs downstack edit [--editor STRING] [--branch NAME]` — Edit order below

## Commit

- `gs commit create [-m MSG] [-a] [--no-verify] [--no-prompt]` — Commit and restack upstack (alias: `gs cc`)
- `gs commit amend [-m MSG] [-a/--all] [--no-verify] [--no-prompt]` — Amend last commit and auto-restack upstack branches (alias: `gs ca`)
- `gs commit split` — Split last commit and restack

## Rebase

Prefer these over `git rebase --continue/--abort` — they auto-restack upstack branches after resuming.

- `gs rebase continue [--no-prompt]` — Resume a paused rebase and auto-restack upstack branches (alias: `gs rbc`)
- `gs rebase abort` — Cancel the current rebase and restore pre-rebase state (alias: `gs rba`)

## Logging

- `gs log short [-a] [-S|--no-cr-status] [--json]` — List branches in stack
- `gs log long [-a] [-S|--no-cr-status] [--json]` — List branches and commits

## Authentication

- `gs auth login [--refresh]` — Authenticate with forge
- `gs auth status` — Show login status
- `gs auth logout` — Remove auth

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
