---
name: git
description: Preferences for any git or GitHub work -- committing, amending, squashing, rebasing, pushing, creating or updating PRs, handling review threads, fixing CI, and merging. Use whenever the user asks to commit, push, open or update a PR, address review feedback, fix CI, rebase, squash, or merge.
---

# Git

Conventions for git and GitHub work. These are preferences, not procedures: do the work the way a careful engineer would, and follow these rules where they apply.

## Branches

- New branches use the `sc/` prefix (`sc/<kebab-slug>`). No other prefix.
- In the `dotfiles` repo, committing and pushing directly to `main` is fine. Everywhere else, ask before committing on `main`/`master` and offer to create a branch.
- Fetch with `git fetch origin`. Never `git fetch origin <branch>:<branch>` -- it fails when the branch is checked out in another worktree.
- `gwt` and `gwc` are fish functions. Call them via `fish -c '...'` from Bash.

## Commits

- "Commit this" / "commit my changes" means the files touched in this session, with no prompt. If other modified files exist and the user did not say "commit all", ask which to include.
- If the changes mix concerns (a refactor beside a bug fix), split into one commit per concern, most foundational first, without asking the user to pick an order. Ask only when a file genuinely belongs to either.
- Draft the message from the diff, not from `git log`. Imperative mood, capitalized, subject under 72 characters, no `feat:`/`fix:` prefixes. Explain why, not what.
- Never `--no-verify`. If a hook fails, fix the cause and retry.
- Amend: if the amended file set differs from the original, offer to rewrite the message; otherwise keep it. If the branch is already pushed, offer a force push (see Rewriting history).

## Pull requests

- The diff is the source of truth. Commit messages, branch history, and session memory are hints and often describe intermediate states; every claim in the description must be verifiable against the diff.
- Describe the net change, not the journey. Scan the draft for journey language per the no-historical-narration rule; a "replaced X with Y" claim stands only if the `-` lines actually show X.
- Explain why. Do not enumerate files, types, helpers, or functions; name only what a reviewer needs to orient.
- Keep it short -- a few sentences for a typical PR. Do not wrap lines; GitHub wraps.
- Title: imperative, under 70 characters, derived from the diff.
- Never reference `tmp/`, gitignored, or other local-only paths. Restate the relevant rationale inline instead.
- No invented numbers, percentages, or performance claims unless they appear literally in the diff.
- When updating an existing PR, preserve any bot-appended sections (BugBot, Dependabot) that are not part of the new text.
- Write multi-line bodies to a per-PR file under `tmp/` and pass `--body-file`; inlining them in `--body` mangles quoting.
- The motivation comes from the conversation or the user's request. If neither has it, ask one question before writing the description.

## Review threads

- Fetch unresolved threads with `~/.claude/skills/git/scripts/get-pr-comments.sh` (`--unreplied`, `--count`, `--summary`, `--pr N`). The REST comment id and owner/repo it returns are what the reply and reaction endpoints need.
- Bot threads (bugbot, dependabot, any automated reviewer) are handled autonomously: fix valid findings, or thumbs-down and reply with a one-line reason when the finding does not apply.
- Human threads require explicit user approval before applying a code fix and before posting any reply. Show each draft reply beside the reviewer's comment and offer approve / skip / edit.
- Reply in-thread via `gh api repos/{owner}/{repo}/pulls/{pull_number}/comments/{comment_id}/replies -F body=@<file>`. Never `gh pr comment` for thread replies, and never pass `commit_id`/`path`/`line` -- that opens a new thread.
- Fixed threads get a thumbs-up and a reply linking the commit.

## CI

- Rerun rather than fix when the failure is transient infrastructure (timeout, 429, 502/503/504, OOM, runner lost, no space left) or when the same workflow has also failed on the base branch in the last week.
- Snapshot mismatches, type errors, and lint errors are always real, even if the base branch has similar failures.

## Rewriting history

- Force push is opt-in. After an amend, squash, or rebase, present the choice; never force push unprompted.
- Before any force push, check `gh pr list --base <this-branch> --state open`. If PRs target this branch, warn and offer to re-base them (`gh pr edit <n> --base <this-branch's-base>`) first -- GitHub can auto-close or auto-merge them otherwise.

## Merge

- Match the repo's existing strategy (merge commit, squash, or rebase) from recent history on the default branch. Ask if it is mixed.
- `gh pr merge <n> --<strategy> --delete-branch`.
- Never `--admin` to bypass failing required checks. Never merge a draft unless the user asks to mark it ready first.

## Outbound text

Applies to commit messages, PR titles and bodies, and review replies:

- ASCII only: `--` not em dashes, straight quotes, `...` not an ellipsis character. Non-ASCII corrupts through the `gh` CLI.
- Wrap anything containing `~` or `*` in backticks; bare they trigger strikethrough and emphasis.
