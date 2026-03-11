# Commit Message Format

Canonical source for inline commit paths. The committer agent maintains a synced copy (see agents/committer.md).

- Draft the message solely from the diff content
- MUST NOT run `git log` or reference previous commit messages
- Imperative mood, start with a capital letter, under 72 chars, explain _why_ not _what_
- No prefix conventions (no `type:`, `scope:`, `feat:`, etc.) -- just a plain sentence
- ASCII only: use `--` instead of em dashes, straight quotes instead of curly quotes, `...` instead of `…`
- No invented metrics: never cite specific numbers, percentages, or performance claims unless they appear literally in the diff

## Multi-concern commits

When a commit (typically after squash) spans multiple distinct concerns, give each concern its own sentence in the body; use a blank-line-separated paragraph only when a concern needs additional explanation beyond one sentence. Do not bury a secondary concern as a trailing clause of another sentence -- a reviewer scanning the message will miss it. Order by significance: primary change first, then secondary changes, each clearly separated. Significance is judged by diff size and user-facing impact; when ambiguous, order by diff size.

## Inline Commit Procedure

Steps for any operation that commits inline (without delegating to the `committer` agent):

1. Stage files: `git add <file1> ...`
2. Run `git diff --staged` to review what will be committed
3. Draft message per the rules above (or use a provided message)
4. Sanitize and commit: write the message to `./tmp/commit-msg.txt` using the Write tool, then `~/.claude/skills/git/scripts/sanitize.sh --commit-msg ./tmp/commit-msg.txt && git commit -F ./tmp/commit-msg.txt`
5. If `sanitize.sh` rejects the message (exit 1 -- subject too long or empty), shorten and re-run step 4.
6. Error: pre-commit hook failure -- read the error output, fix the issue, re-stage, retry. MUST NOT use `--no-verify`.
7. Report: `git log -1 --oneline`
