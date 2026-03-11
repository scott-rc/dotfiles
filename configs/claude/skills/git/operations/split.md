# Split

Split a large branch into stacked branches grouped by logical concern, creating PRs for each so reviewers can evaluate changes incrementally.

## Instructions

### Phase 1: Planning (orchestrator, steps 0–4)

0. **Resume detection**: Derive the branch directory by running `~/.claude/skills/git/scripts/branch-context-path.sh` and stripping the filename. Check for `split-state.json` in that directory. If the file exists with `"status": "in-progress"`, ask via AskUserQuestion: "Found an in-progress split with N of M branches completed. Resume or start fresh?" — Resume regenerates the orchestrator prompt from the state file (step 4 template) and enters plan mode; Start fresh deletes the state file and continues to step 1.

1. **Gather state**: Record the current branch as the reference branch — it stays untouched throughout. Detect base branch per references/git-patterns.md (Base Branch Detection). Run:
   - `git fetch origin`
   - `git diff --stat origin/<base>...HEAD` for file count and line totals (triple-dot: excludes base-branch-only changes)
   - `git log --oneline origin/<base>..HEAD` for commit count and scope (double-dot: walks reachable commits)

   If fewer than 2 files changed AND fewer than 200 lines changed total, inform the user and stop.

2. **Analyze diff**: Delegate to an `Explore` subagent. Pass the output of `git diff origin/<base>...HEAD` and the stat summary. If the diff exceeds ~10,000 lines, pass `--stat` output instead and instruct the subagent to read individual file diffs selectively. The subagent MUST:
   - Group changes by **logical concern** — a file MAY appear in multiple groups
   - For each group: return theme, review focus, **changes** (plain-English bullets describing what to implement), relevant files, estimated line count, and dependency notes
   - Include associated generated files (test snapshots, codegen output) in each group's relevant_files alongside their source files
   - Propose an ordering (foundational changes first)

3. **Propose stack**: Present the stack via AskUserQuestion. For each branch show:
   - Stack position, proposed name (`sc/` prefix per references/git-patterns.md Branch Naming), theme and review focus
   - Changes bullets, relevant files (noting any overlaps with other branches), estimated size

   Notes to include: files may appear in multiple branches; each branch should compile independently (aspirational, not required); the reference branch is a guide, not an exact target. Ask the user to approve, modify, or reject. Apply modifications and confirm before proceeding.

4. **Write state file and enter plan mode**: Write `split-state.json` to the branch directory (derived from `~/.claude/skills/git/scripts/branch-context-path.sh` — strip the filename to get the directory):

   ```json
   {
     "reference_branch": "sc/big-feature",
     "base_branch": "main",
     "status": "approved | in-progress | complete",
     "repo_root": "/absolute/path/to/repo",
     "stack": [
       {
         "position": 1,
         "branch_name": "sc/big-feature-schema",
         "theme": "Database schema changes",
         "review_focus": "Migration safety, index coverage",
         "changes": [
           "Add user_preferences table with JSON column",
           "Add index on users.email for new lookup pattern"
         ],
         "relevant_files": ["db/migrations/001.sql", "db/schema.prisma"],
         "estimated_lines": 45,
         "status": "pending | writing | committed | pushed | pr-created | failed | skipped",
         "pr_url": null,
         "error": null
       }
     ]
   }
   ```

   Write the orchestrator prompt (below) to the plan file (the path provided by the plan mode system message). Enter plan mode via `EnterPlanMode`, then `ExitPlanMode` for user approval.

   **Orchestrator prompt template** (fill `<...>` values from the state file):

   ```markdown
   # Goal

   Execute the stacked branch split defined in <state-file-path>.

   # Context

   - State file: <state-file-path>
   - Reference branch: <reference_branch> (guide for what to implement — not an exact-match target)
   - Base branch: <base_branch>
   - Stack: <N> branches

   # Requirements

   1. Read the state file to get the full stack and all context.
   2. For each branch in stack order (skip any with status `pr-created`):
      a. Create branch: `git checkout -b <name> origin/<base>` for the first; `git checkout -b <name> <previous-branch>` for subsequent.
      b. Generate scoped reference diff: `git diff origin/<base>...<reference_branch> -- <relevant_files>` (triple-dot) as guidance.
      c. Delegate to code-writer with `mode: apply`, `task`: "Implement the following changes for the '<theme>' concern: <changes bullets>. Use the reference diff below as a guide — implement the changes naturally, do not copy mechanically. Reference diff: <scoped diff>", `files`: relevant_files, `constraints`: "Branch <N> of <M> in a stacked split. Previous branches implemented: <summary of earlier themes>. Do not re-implement or undo their work. Only modify files in this branch's relevant_files list — do not touch files scoped to other branches. Branch should compile and lint independently."
      d. Write the branch context file (path per references/git-patterns.md "Branch Context File") with theme and review focus.
      e. Delegate to committer: "Commit all staged and unstaged changes. This is branch N of M in a stacked split. Theme: <theme>." MUST NOT pass a pre-written commit message.
      f. Update state file: set branch status to `committed`.
   3. Push each committed branch and create PRs:
      - `git push -u origin <branch-name>`
      - Delegate to pr-writer per references/pr-writer-rules.md:
        - `mode`: create
        - `base_branch`: previous stack branch (or `origin/<base>` for the first)
        - `commit_messages`: run `git log <base-for-this-branch>..HEAD --format=%B` and pass the captured output verbatim
        - `branch_context`: contents of this branch's context file
        - `context`: "Branch N of M in a stacked split. Base is <previous-stack-branch>. The reference branch (<reference_branch>) shows the original combined intent."
      - Update state file: set status to `pr-created`, record PR URL.
   4. Verify: `git diff --stat <last-stack-branch> <reference_branch>`. This is informational — divergence is expected since agents implement changes naturally. Empty diff means exact match; non-empty shows the summary.
   5. Check out the reference branch. Report all branches with PR URLs, themes, sizes, and the verification summary.

   # Cross-Branch Compatibility

   When branch N changes an interface or type that code inherited from earlier in the stack still uses (e.g., removing fields from a struct that later-branch code calls with), the branch must remain compilable. Include in the code-writer's `task` a note: "This branch inherits code that later branches will modify. If you change a public interface, keep backward-compatible signatures (optional/deprecated fields) so inherited callers still compile." Do not fix this inline — re-delegate to the code-writer if compilation fails due to interface changes.

   # Error Handling

   - If code-writer fails after retries, ask the user: retry / skip / stop.
   - Update the state file on every status change so the split can be resumed.
   - If the code-writer doesn't apply expected snapshot or generated file changes, re-delegate with explicit instruction to apply those file changes directly from the reference diff — regeneration may not be possible if required services aren't running.
   ```

### Phase 2: Execution (plan-mode orchestrator prompt)

The orchestrator prompt above drives execution. It reads the state file for all context — no prior conversation knowledge is assumed. See the Requirements section in the prompt for full step details.
