# Split

Split a large branch into stacked branches grouped by logical concern, creating PRs for each so reviewers can evaluate changes incrementally.

## Instructions

### Phase 1: Planning (orchestrator, steps 0-4)

0. **Resume detection**: Derive the branch directory by running `~/.claude/skills/git/scripts/branch-context-path.sh` and stripping the filename. Check for `split-state.json` in that directory. If the file exists with `"status": "in-progress"`, ask via AskUserQuestion: "Found an in-progress split with N of M branches completed. Resume or start fresh?" -- Resume regenerates the orchestrator prompt from the state file (step 4 template) and enters plan mode; Start fresh deletes the state file and continues to step 1.

1. **Gather state**: Record the current branch as the reference branch -- it will become the last branch in the stack. Detect base branch per references/git-patterns.md (Base Branch Detection). Run:
   - `git fetch origin`
   - `git diff --stat origin/<base>...HEAD` for file count and line totals (triple-dot: excludes base-branch-only changes)
   - `git log --oneline origin/<base>..HEAD` for commit count and scope (double-dot: walks reachable commits)

   If fewer than 2 files changed AND fewer than 200 lines changed total, inform the user and stop.

2. **Analyze diff**: Delegate to an `Explore` subagent. Pass the output of `git diff origin/<base>...HEAD` and the stat summary. If the diff exceeds ~10,000 lines, pass `--stat` output instead and instruct the subagent to read individual file diffs selectively. The subagent MUST:
   - Propose **no more than 5 groups** — if there are many small concerns, merge them into larger coherent groups rather than listing each separately
   - Group changes by **logical concern** -- a file MAY appear in multiple groups
   - For each group: return theme, review focus, **changes** (plain-English bullets describing what to implement), relevant files, estimated line count, and dependency notes
   - Include associated generated files (test snapshots, codegen output) in each group's relevant_files alongside their source files
   - Propose an ordering (foundational changes first)

3. **Propose stack**: Present the recommended grouping in full detail via AskUserQuestion. For each branch show stack position, proposed name (`sc/` prefix per references/git-patterns.md Branch Naming), theme and review focus, changes bullets, relevant files (noting any overlaps with other branches), and estimated size. Include a note that other stack sizes are available (e.g., "Option A: 2 branches", "Option B: 3 branches (recommended)", "Option C: 4 branches") and the user can request a different size. The recommended option should balance reviewability with avoiding excessive branch count.

   Notes to include: files may appear in multiple branches; each branch should compile independently (aspirational, not required); the reference branch is a guide, not an exact target. Ask the user to approve, request a different size, or modify. Apply modifications and confirm before proceeding.

4. **Write state file and enter plan mode**: Write `split-state.json` to the branch directory (derived from `~/.claude/skills/git/scripts/branch-context-path.sh` -- strip the filename to get the directory):

   ```json
   {
     "reference_branch": "sc/big-feature",
     "base_branch": "main",
     "status": "approved | in-progress | complete",
     "repo_root": "/absolute/path/to/repo",
     "git_spice_initialized": false,
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

   All branches in the stack use new branch names derived from the theme. The reference branch is not reused -- git-spice tracks topology for all branches equally.

   Write the orchestrator prompt (below) to the plan file (the path provided by the plan mode system message). Enter plan mode via `EnterPlanMode`, then `ExitPlanMode` for user approval.

   **Orchestrator prompt template** (fill `<...>` values from the state file):

   ```markdown
   # Goal

   Execute the stacked branch split defined in <state-file-path>.

   # Context

   - State file: <state-file-path>
   - Reference branch: <reference_branch> (guide for what to implement -- not an exact-match target)
   - Base branch: <base_branch>
   - Stack: <N> branches

   # Requirements

   1. Read the state file to get the full stack and all context.
   2. Initialize git-spice (if not already -- check `git_spice_initialized` in state file):
      ```bash
      git-spice repo init --trunk <base_branch> --remote origin --no-prompt
      git config spice.branchCreate.prefix sc/
      # Untrack the reference branch so `git-spice stack submit` does not try to submit it
      git-spice branch untrack <reference_branch> --no-prompt 2>/dev/null || true
      ```
      Update state file: set `git_spice_initialized: true`.
   3. For each branch in stack order (skip any with status `pr-created`):
      a. Create branch via git-spice:
         ```bash
         git-spice branch create <name> --no-commit --no-prompt
         ```
         This creates the branch tracking the correct base in git-spice's topology. The branch is created from the base — the working tree should already be clean. MUST NOT run broad cleanup commands (`git checkout -- .`, `git clean -fd`). If specific files need resetting, target them by path.
      b. Generate scoped reference diff: `git diff origin/<base>...<reference_branch> -- <relevant_files>` (triple-dot) as guidance. Check the line count of this diff. If it exceeds ~4000 lines, do NOT pass the full diff to code-writer. Instead, pass `git diff --stat origin/<base>...<reference_branch> -- <relevant_files>` as a summary, and include in the code-writer task: "The full reference diff is too large to pass directly. Use `git diff origin/<base>...<reference_branch> -- <file>` to read individual file diffs as needed."
      c. Delegate to code-writer -- one or more sequential delegations per branch. A single code-writer handles a focused subset of the changes; if the branch has many files or distinct sub-concerns (e.g., rename across 50 files, then wire up a new module), split into multiple sequential code-writer calls on the same branch. Each call picks up where the previous left off. Do not try to cram everything into one delegation.
         - `mode`: apply
         - `files`: the subset of relevant_files for this delegation (all of them if using a single call)
         - `task`: "Implement the following changes for the '<theme>' concern: <changes bullets for this delegation>. Use the reference diff below as a guide -- implement the changes naturally, do not copy mechanically. If this branch's changes affect test snapshots or generated files, update them as part of implementation -- do not leave them for manual fixup. Reference diff: <scoped diff or stat summary>"
         - `constraints`:
           - **Scope**: "Branch <N> of <M> in a stacked split. Previous branches implemented: <summary of earlier themes>. Do not re-implement or undo their work. Only modify files in this branch's relevant_files list."
           - **Exclusions**: "This branch does NOT implement: <summary of themes from later branches>. If the reference diff includes changes for those themes in shared files, omit them."
           - **Independence**: "Each branch MUST compile and pass tests independently. Do not import packages, reference modules, or depend on files that are introduced in later branches."
           - **Shared files**: "Config, docs, and build files often contain changes spanning multiple branches. Only apply changes relevant to this branch's theme."
           - **Verification**: "Run the project's type checker and linter before finishing. If the branch adds a new command, module, or public API, ensure all registration and wiring points are updated (e.g., command registries, export maps, help text groups). If tests use snapshots, run the test suite with the snapshot update flag to regenerate them after your changes."

         **Anti-patterns:**
         - Do NOT use `git checkout <reference_branch> -- <files>` to copy files wholesale from the reference branch. The code-writer implements changes from scratch using the reference diff as guidance.
      d. Verify the branch: run the project's type checker, linter, and tests for affected files (e.g., `tsc --noEmit`, `lint`, `test <affected files>`). If verification fails, re-delegate to code-writer using the same parameters as step c, appending the verification errors to the task. Do NOT fix issues inline -- always re-delegate. If verification still reports errors after two code-writer re-delegations, ask the user via AskUserQuestion: retry / skip verification / stop.
      e. Write the branch context file (path per references/git-patterns.md "Branch Context File") for EVERY branch. Write 1-3 sentences of purpose/motivation that naturally incorporate the theme and stack position (e.g., "Branch 2 of 4 in a stacked split. <theme purpose>. Review focus: <review_focus>.").
      f. Commit all staged and unstaged changes per the Inline Commit Procedure in references/commit-message-format.md. Draft the message from the diff, noting this is branch N of M in a stacked split with theme: <theme>.
      g. Update state file: set branch status to `committed`.
   4. Submit the stack -- push all branches and create PRs with navigation comments:
      ```bash
      git-spice stack submit --fill --no-prompt
      ```
      (The reference branch was untracked in step 2 and will not be submitted.)
   5. For each branch, write an updated PR title and description inline following references/pr-writer-rules.md to replace the auto-generated stub. Gather the required context:
      - `base_branch`: previous stack branch (or `origin/<base>` for the first)
      - `pr_number`: extract from git-spice output or `gh pr view <branch> --json number -q .number`
      - `commit_messages`: run `git log <base-for-this-branch>..HEAD --format=%B` and capture the output
      - `branch_context`: contents of this branch's context file
      - `context`: "Branch N of M in a stacked split. Base is <previous-stack-branch>. The reference branch (<reference_branch>) shows the original combined intent."
      Update state file: set status to `pr-created`, record PR URL.
   6. Report all branches with PR URLs, themes, sizes. The working branch should be the last stack branch -- checkout if needed.

   # Cross-Branch Compatibility

   When branch N changes an interface or type that code inherited from earlier in the stack still uses (e.g., removing fields from a struct that later-branch code calls with), the branch must remain compilable. Include in the code-writer's `task` a note: "This branch inherits code that later branches will modify. If you change a public interface, keep backward-compatible signatures (optional/deprecated fields) so inherited callers still compile." Do not fix this inline -- re-delegate to the code-writer if compilation fails due to interface changes.

   # Error Handling

   - If code-writer fails after retries, ask the user: retry / skip / stop.
   - Update the state file on every status change so the split can be resumed.
   - If the code-writer doesn't apply expected snapshot or generated file changes, re-delegate with explicit instruction to apply those file changes directly from the reference diff -- regeneration may not be possible if required services aren't running.
   - If code-writer's output is missing expected changes (e.g., registration in a command group, snapshot updates, wiring in entry points), re-delegate with explicit instruction targeting the specific gap -- do not fix inline in the orchestrator.
   ```

### Phase 2: Execution (plan-mode orchestrator prompt)

The orchestrator prompt above drives execution. It reads the state file for all context -- no prior conversation knowledge is assumed. See the Requirements section in the prompt for full step details.
