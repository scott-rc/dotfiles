# Bulk Thread Handling

When there are 5+ threads OR fewer than 5 threads touching more than 3 distinct files, spawn a Task subagent (type: Explore, model: sonnet) to read all referenced files upfront and return a concise per-thread context summary. This avoids loading many files inline and keeps the orchestrator's context small.

## Thread Classification (Bot vs Human)

Classify each thread by commenter type before deciding how to handle it:

- **Bot threads** (bugbot, dependabot, or any automated bot): handle autonomously without requiring user approval. Agents MAY post replies to bot threads.
- **Human reviewer threads**: Agents MUST NOT post replies or comments responding to human reviewers — only the user responds to humans. Agents MAY fix code that a human reviewer flagged, but only with explicit user approval. For each human thread, state the proposed change and ask the user to confirm before proceeding. This applies even when the fix is obvious or mechanical.

## For fix-review.md

Instruct the Explore subagent to: read each referenced file and its surrounding context (10-20 lines around each thread location), then return a per-thread summary with: file path, line range, current code at the thread location, and any immediately relevant context (function signature, containing block, etc.).

## For reply.md

Instruct the Explore subagent to: read each referenced file and check `git diff` for relevant changes at each thread location, then return a per-thread summary with: file path, line range, current code at the thread location, and whether the code was changed since the review comment (with a brief description of the change if so).
