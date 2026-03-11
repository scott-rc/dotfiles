# Claude Session Transcripts

When the user references a "Claude session" with a hash-like string (e.g., "why did this session do X? c0432bb4"), treat that string as a **Claude session ID or prefix** — NOT a git commit hash. Do not run `git show` or `git log` on it.

## Using `claude-transcripts`

Use the `claude-transcripts` script at `~/Code/personal/dotfiles/tools/claude-transcripts` to inspect sessions. It parses JSONL transcripts from `~/.claude/projects/`.

Key subcommands:

- `claude-transcripts show <session-id>` — show a transcript; flags: `--full`, `--thinking`, `--all`
- `claude-transcripts search <pattern>` — search across transcripts
- `claude-transcripts list` — list recent sessions; flags: `--no-subagents`, `--no-sidechains`, `-p <project>`, `-n <limit>`
- `claude-transcripts stats` — aggregate statistics
