# Parallel Subagent Safety

Parallel subagents share a working tree. Destructive git commands can silently clobber a sibling agent's in-progress edits.

## Rules

- MUST NOT run destructive git commands — `git checkout --`, `git reset`, `git clean`, `git stash` — broadly across the working tree; always target files explicitly by path and only on files the agent itself created or edited in this session
- `git stash` captures ALL dirty files, including siblings' changes — treat it as out-of-scope by default
- If `git diff` or `git status` shows unexpected changes in files the agent didn't touch, ignore them — they belong to a sibling agent running in parallel
- To discard only the agent's own changes, target files explicitly: `git restore <file>` on a file this agent modified

## Shared Mutable State

Parallel subagents share the entire working tree. Beyond git commands, watch for:

- **Temp files** — fixed-path temp files (e.g., `./tmp/pr-body.txt`) collide when multiple agents write to the same path. Use unique paths — include PR number, branch name, or an agent-specific suffix.
- **Git HEAD** — all agents resolve `HEAD` to the same ref. Commands like `git diff origin/main...HEAD` return identical results regardless of which agent runs them. Pass explicit branch names rather than relying on `HEAD`.
- **Working tree files** — one agent's uncommitted edits are visible to all siblings. MUST NOT read or depend on files another parallel agent is actively modifying.

When dispatching parallel agents, audit for shared mutable state: if two agents would read or write the same path or git ref, either serialize them or make the references unique.
