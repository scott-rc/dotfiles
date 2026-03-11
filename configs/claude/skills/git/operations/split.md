# Split

Split a large branch into stacked branches grouped by logical concern, creating PRs for each so reviewers can evaluate changes incrementally.

## Instructions

### Phase 1: Planning (orchestrator, steps 0–4)

0. **Resume detection**: Derive the branch directory by running `~/.claude/skills/git/scripts/branch-context-path.sh` and stripping the filename. Check for `split-state.json` in that directory. If the file exists with `"status": "in-progress"`, ask via AskUserQuestion: "Found an in-progress split with N of M branches completed. Resume or start fresh?" — Resume regenerates the orchestrator prompt from the state file (step 4 template) and enters plan mode; Start fresh deletes the state file and continues to step 1.

1. **Gather state**: Record the current branch as the reference branch — it will become the last branch in the stack. Detect base branch per references/git-patterns.md (Base Branch Detection). Run:
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

   Notes to include: files may appear in multiple branches; each branch should compile independently (aspirational, not required); the reference branch is a guide, not an exact target; the last branch in the stack reuses the existing reference branch name. Ask the user to approve, modify, or reject. Apply modifications and confirm before proceeding.

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
         "reuses_reference_branch": false,
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

   The last entry in `stack` MUST have `"branch_name": "<reference_branch>"` and `"reuses_reference_branch": true`. All other entries have `"reuses_reference_branch": false` and use new branch names derived from the theme.

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
      a. Create or reset branch:
         - First branch: `git checkout -b <name> origin/<base>`
         - Middle branches (not first, not last): `git checkout -b <name> <previous-branch>`
         - Last branch (`reuses_reference_branch: true`): `git checkout <reference_branch>` then `git reset --hard <previous-branch>` to rebase it onto the previous stack branch
      b. Generate scoped reference diff: `git diff origin/<base>...<reference_branch> -- <relevant_files>` (triple-dot) as guidance. Check the line count of this diff. If it exceeds ~4000 lines (~4000 lines risks exceeding the code-writer's effective context budget; the Explore subagent in Phase 1 has a higher ~10,000-line threshold because it reads selectively), do NOT pass the full diff to code-writer. Instead, pass `git diff --stat origin/<base>...<reference_branch> -- <relevant_files>` as a summary, and include in the code-writer task: "The full reference diff is too large to pass directly. Use `git diff origin/<base>...<reference_branch> -- <file>` to read individual file diffs as needed."
      c. Delegate to code-writer — one or more sequential delegations per branch. A single code-writer handles a focused subset of the changes; if the branch has many files or distinct sub-concerns (e.g., rename across 50 files, then wire up a new module), split into multiple sequential code-writer calls on the same branch. Each call picks up where the previous left off. Do not try to cram everything into one delegation.
         - `mode`: apply
         - `files`: the subset of relevant_files for this delegation (all of them if using a single call)
         - `task`: "Implement the following changes for the '<theme>' concern: <changes bullets for this delegation>. Use the reference diff below as a guide — implement the changes naturally, do not copy mechanically. If this branch's changes affect test snapshots or generated files, update them as part of implementation — do not leave them for manual fixup. Reference diff: <scoped diff or stat summary>"
         - `constraints`: "Branch <N> of <M> in a stacked split. Previous branches implemented: <summary of earlier themes>. Do not re-implement or undo their work. Only modify files in this branch's relevant_files list — do not touch files scoped to other branches. Verify your changes compile and pass lint before finishing — run the project's type checker and linter. If the branch adds a new command, module, or public API, ensure all registration and wiring points are updated (e.g., command registries, export maps, help text groups). If tests use snapshots, run the test suite with the snapshot update flag to regenerate them after your changes."
      d. Verify the branch: run the project's type checker, linter, and tests for affected files (e.g., `tsc --noEmit`, `lint`, `test <affected files>`). If verification fails, re-delegate to code-writer using the same parameters as step c, appending the verification errors to the task. Do NOT fix issues inline — always re-delegate. If verification still reports errors after two code-writer re-delegations, ask the user via AskUserQuestion: retry / skip verification / stop.
      e. Write the branch context file (path per references/git-patterns.md "Branch Context File") for EVERY branch, including the last (reused) branch. Write 1-3 sentences of purpose/motivation that naturally incorporate the theme and stack position (e.g., "Branch 2 of 4 in a stacked split. <theme purpose>. Review focus: <review_focus>.").
      f. Delegate to committer: "Commit all staged and unstaged changes. This is branch N of M in a stacked split. Theme: <theme>." MUST NOT pass a pre-written commit message.
      g. Update state file: set branch status to `committed`.
   3. Push each committed branch and create PRs:
      - `git push -u origin <branch-name>` (for the last branch, this force-pushes the reused reference branch — use `git push --force-with-lease -u origin <branch-name>`)
      - Delegate to pr-writer per references/pr-writer-rules.md:
        - `mode`: create
        - `base_branch`: previous stack branch (or `origin/<base>` for the first)
        - `commit_messages`: run `git log <base-for-this-branch>..HEAD --format=%B` and pass the captured output verbatim
        - `branch_context`: contents of this branch's context file
        - `context`: "Branch N of M in a stacked split. Base is <previous-stack-branch>. The reference branch (<reference_branch>) shows the original combined intent."
      - Update state file: set status to `pr-created`, record PR URL.
   4. Verify: `git diff --stat <second-to-last-stack-branch> <last-stack-branch>`. Since the last stack branch IS the reference branch, this shows what the final branch adds on top of the previous one. This is informational.
   5. Report all branches with PR URLs, themes, sizes, and the verification summary. The working branch is already the last stack branch (the reference branch) — no additional checkout needed.

   # Cross-Branch Compatibility

   When branch N changes an interface or type that code inherited from earlier in the stack still uses (e.g., removing fields from a struct that later-branch code calls with), the branch must remain compilable. Include in the code-writer's `task` a note: "This branch inherits code that later branches will modify. If you change a public interface, keep backward-compatible signatures (optional/deprecated fields) so inherited callers still compile." Do not fix this inline — re-delegate to the code-writer if compilation fails due to interface changes.

   # Error Handling

   - If code-writer fails after retries, ask the user: retry / skip / stop.
   - Update the state file on every status change so the split can be resumed.
   - If the code-writer doesn't apply expected snapshot or generated file changes, re-delegate with explicit instruction to apply those file changes directly from the reference diff — regeneration may not be possible if required services aren't running.
   - If code-writer's output is missing expected changes (e.g., registration in a command group, snapshot updates, wiring in entry points), re-delegate with explicit instruction targeting the specific gap — do not fix inline in the orchestrator.
   ```

### Phase 2: Execution (plan-mode orchestrator prompt)

The orchestrator prompt above drives execution. It reads the state file for all context — no prior conversation knowledge is assumed. See the Requirements section in the prompt for full step details.
