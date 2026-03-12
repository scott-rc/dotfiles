# Claude Session Transcripts

When the user references a "Claude session" with a hash-like string (e.g., "why did this session do X? c0432bb4"), treat that string as a **Claude session ID or prefix** — NOT a git commit hash. MUST NOT run `git show` or `git log` on it.

## Using `claude-transcripts`

Use the `claude-transcripts` script at `~/Code/personal/dotfiles/tools/claude-transcripts` to inspect sessions. It parses JSONL transcripts from `~/.claude/projects/`.

Key subcommands:

- `claude-transcripts show <session-id>` — show a transcript; flags: `--full`, `--thinking`, `--all`
- `claude-transcripts search <pattern>` — search across transcripts
- `claude-transcripts list` — list recent sessions; flags: `--no-subagents`, `--no-sidechains`, `-p <project>`, `-n <limit>`
- `claude-transcripts stats` — aggregate statistics

## Reading Transcripts

MUST NOT use the Read tool on raw transcript JSONL files or their persisted-output redirects — they routinely exceed size limits and cascade into multiple "too large" failures. Use `claude-transcripts` subcommands via Bash instead:

- **Overview**: `claude-transcripts show <id>` (truncated by default)
- **Full content in chunks**: `claude-transcripts show <id> --full | head -n <N>` or `| tail -n +<N>` to page through large transcripts
- **Targeted extraction**: `claude-transcripts search <pattern>` to find specific content across sessions
- **Filtering**: pipe output through `grep` to locate relevant sections
