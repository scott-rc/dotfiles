# Background Agents and Worktrees

## Background Agents

- Run subagents in **foreground** (default) when you need results before continuing
- If you use `run_in_background: true`, use `TaskOutput` with `block: true` to wait — do NOT attempt to `resume` a running agent (it errors with "Cannot resume agent: it is still running")
- `TaskOutput` timeout means the result wasn't delivered in time — not that the subagent failed. MUST NOT treat a delivery timeout as evidence that delegation doesn't work; continue delegating normally
- If the same delegation fails on retry, inspect the subagent's output or error before delegating again — don't loop blindly

## Worktrees

- MUST NOT use `isolation: worktree` in agent configs or pass `isolation: "worktree"` to the Task tool. The user manages worktrees manually.
