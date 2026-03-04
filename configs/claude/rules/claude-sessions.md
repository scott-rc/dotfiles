# Claude Session Transcripts

When the user references a "Claude session" with a hash-like string (e.g., "why did this session do X? c0432bb4"), treat that string as a **Claude session ID or prefix** — NOT a git commit hash. Do not run `git show` or `git log` on it.

## Locating a Session

Session transcripts live at:

```
~/.claude/projects/<project-dir>/<session-id>.jsonl
```

Project dirs encode the absolute path with slashes replaced by dashes:

- `/Users/scott/Code/gadget/gadget` → `-Users-scott-Code-gadget-gadget`
- `/Users/scott/Code/personal/dotfiles` → `-Users-scott-Code-personal-dotfiles`

To find a session by ID prefix:

```
find ~/.claude/projects -name "<prefix>*.jsonl"
```

Or browse by project:

```
ls ~/.claude/projects/<project-dir>/
```

## Reading the Transcript

The file is JSONL — one JSON event per line. To understand what the session did, look at:

- `content` fields for assistant messages and tool results
- `type: "tool_use"` entries with `name: "Bash"` for commands run
- `type: "tool_use"` entries with `name: "Write"` or `name: "Edit"` for file changes
