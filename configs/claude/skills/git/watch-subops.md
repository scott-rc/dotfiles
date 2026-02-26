# Watch State Reference

State file format for the watch loop. Referenced by [watch.md](watch.md). Context management rules and behavioral instructions live in watch.md.

## State File Format

```markdown
# CI Watch State: PR #<number>

## Metadata
- head_sha: <sha>
- last_push_time: <iso timestamp>
- iteration: <n>
- started_at: <iso timestamp>
- sleep_interval: <seconds> — AIMD poll interval (10–120); see watch.md step 5

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

