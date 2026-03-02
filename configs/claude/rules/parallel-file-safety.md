# Parallel Subagent File Safety

Parallel subagents share a working tree. Destructive git commands can silently clobber a sibling agent's in-progress edits.

## Rules

- MUST NOT run destructive git commands — `git checkout --`, `git restore`, `git reset`, `git clean`, `git stash` — on files outside the agent's own scope (files it created or edited in this session)
- `git stash` captures ALL dirty files, including siblings' changes — treat it as out-of-scope by default
- If `git diff` or `git status` shows unexpected changes in files the agent didn't touch, ignore them — they belong to a sibling agent running in parallel
- To discard only the agent's own changes, target files explicitly: `git restore <file>` on a file this agent modified
