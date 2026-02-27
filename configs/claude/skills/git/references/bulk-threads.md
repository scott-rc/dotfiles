# Bulk Thread Handling

When there are 5+ threads OR fewer than 5 threads touching more than 3 distinct files, spawn a Task subagent (type: Explore, model: sonnet) to read all referenced files upfront and return a concise per-thread context summary. This avoids loading many files inline and keeps the orchestrator's context small.

## For review.md

Instruct the Explore subagent to: read each referenced file and its surrounding context (10-20 lines around each thread location), then return a per-thread summary with: file path, line range, current code at the thread location, and any immediately relevant context (function signature, containing block, etc.).

## For reply.md

Instruct the Explore subagent to: read each referenced file and check `git diff` for relevant changes at each thread location, then return a per-thread summary with: file path, line range, current code at the thread location, and whether the code was changed since the review comment (with a brief description of the change if so).
