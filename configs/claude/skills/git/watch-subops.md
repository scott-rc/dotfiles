# Watch State Reference

State file format and context management rules for the watch loop. Referenced by [watch.md](watch.md).

All fixes MUST be delegated to subagents -- reading source files and attempting fixes inline exhausts the context window and causes the loop to lose track of its monitoring state. The orchestrator triages and dispatches; subagents debug and fix. Each subagent prompt MUST include the repository root path.

## Context Management

The state file (`./tmp/ci-watch-<pr-number>.md`) is your persistent memory across iterations. Follow these rules strictly:

- **Read state at the start of each iteration.** The file is the source of truth, not your recollection.
- **Write state at the end of each iteration.** Every mutation (new handled check, new handled thread, updated fix attempts, new action log entry) goes into the file.
- **Do NOT retain raw poll JSON or subagent results in your working memory after processing.** Once you have extracted the relevant data and written it to the state file, discard the details. The state file is your record.
- **Do NOT summarize or accumulate iteration history in conversation.** If you need to recall what happened, read the state file.

This discipline keeps the context window small across 90-iteration sessions.

## State File Format

```markdown
# CI Watch State: PR #<number>

## Metadata
- head_sha: <sha>
- last_push_time: <iso timestamp>
- iteration: <n>
- started_at: <iso timestamp>

## Handled Checks
- <check name 1>
- <check name 2>

## Handled Threads
- <thread id 1>
- <thread id 2>

## Fix Attempts
- <check name>: <count>

## Actions Log
- [<timestamp>] <action description>
- [<timestamp>] <action description>

## Latest Status
<one-line status from most recent poll>
```

Each section uses simple markdown list items for easy reading and writing. Empty sections should contain no list items (just the heading).

