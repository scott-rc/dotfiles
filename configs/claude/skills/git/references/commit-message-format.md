# Commit Message Format

Canonical source for inline commit paths. The committer agent maintains a synced copy (see agents/committer.md).

- Draft the message solely from the diff content
- MUST NOT run `git log` or reference previous commit messages
- Imperative mood, start with a capital letter, under 72 chars, explain _why_ not _what_
- No prefix conventions (no `type:`, `scope:`, `feat:`, etc.) -- just a plain sentence
- ASCII only: use `--` instead of em dashes, straight quotes instead of curly quotes, `...` instead of `…`
- Multi-line: write to a temp file and `git commit -F <file>` (not repeated `-m` args)
- No invented metrics: never cite specific numbers, percentages, or performance claims unless they appear literally in the diff

## Inline Commit Procedure

Steps for any operation that commits inline (without delegating to the `committer` agent):

1. Stage files: `git add <file1> ...`
2. Run `git diff --staged` to review what will be committed
3. Draft message per the rules above (or use a provided message)
4. Pipe message through `safe-text.sh --commit-msg --prefix commit-msg`, then `git commit -F <file>`
5. Error: pre-commit hook failure -- read the error output, fix the issue, re-stage, retry. MUST NOT use `--no-verify`.
6. Report: `git log -1 --oneline`
