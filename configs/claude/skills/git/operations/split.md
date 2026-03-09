# Split

Split a large branch into stacked branches grouped by logical concern, creating PRs for each so reviewers can evaluate changes incrementally.

## Instructions

1. **Gather state**: Detect base branch per references/git-patterns.md. Record the current branch as the reference branch -- it is kept untouched throughout. Run:
   - `git fetch origin`
   - `git diff --stat origin/<base>...HEAD` to see file count and line totals
   - `git log --oneline origin/<base>..HEAD` to see commit count and scope

   If the branch has fewer than 2 files changed AND fewer than 200 lines changed total, inform the user the branch is small enough to review as-is and stop.

2. **Analyze diff** (delegate to `Explore` subagent): Spawn an `Explore` subagent to analyze the full diff. Pass: the output of `git diff origin/<base>...HEAD` and the file stat summary from step 1. If the diff exceeds ~10,000 lines, pass `--stat` output instead and instruct the subagent to selectively read individual file diffs as needed. The subagent MUST:
   - Group files by logical concern (e.g., schema changes, infra, client refactor, tests, frontend, config)
   - Check cross-group imports and dependencies to identify coupling constraints -- coupled files prefer the same group, but a later branch can fix compilation issues introduced by an earlier branch, so perfect isolation is not required
   - For each group: list files, estimate line count, suggest a theme name and review focus
   - Propose an ordering that respects dependencies (foundational changes first, dependent changes after)
   - Return structured output: ordered list of groups with files, theme names, estimated sizes, and dependency notes

3. **Propose stack** (confirm): Present the proposed stack to the user via AskUserQuestion. For each branch in stack order show:
   - Stack position (e.g., "Branch 1 of 3")
   - Proposed branch name using the `sc/` prefix convention from references/git-patterns.md
   - Theme and review focus
   - File list
   - Estimated size (lines changed)

   Notes to include in the prompt:
   - Each stacked branch MUST compile and pass CI independently
   - Splits don't need to be perfectly clean -- a later branch can change or undo work from an earlier branch, since the reference branch guarantees the final combined state

   Ask the user to approve the proposed stack, modify it (e.g., rename a branch, merge two groups, reorder), or reject it.

   If the user modifies the stack, apply their changes and confirm the revised plan before proceeding.

4. **Validate file assignments and create/commit each branch** (write): Before creating any branches, validate that no file appears in more than one group. If duplicates exist, report which files are assigned to multiple groups and ask the user to resolve the conflict before proceeding.

   Then, for each branch in stack order:
   - Check out the branch: `git checkout -b <branch-name> origin/<base-branch>` for the first branch in the stack; `git checkout -b <branch-name> <previous-stack-branch>` for subsequent branches
   - Pull files for that branch from the reference: `git checkout <reference-branch> -- <file1> <file2> ...`
   - Write the branch context file (path per references/git-patterns.md "Branch Context File") with the theme and review focus for that branch -- this MUST be written before delegating to `committer`, as the committer reads it
   - Delegate to the `committer` agent with: "Commit all staged and unstaged changes. This is branch N of M in a stacked split for review. Theme: <theme>. Branch context: <one-line description>." MUST NOT pass a pre-written commit message -- the committer drafts it from the diff

5. **Push and create PRs** (publish): For each branch in stack order:
   - `git push -u origin <branch-name>`
   - Spawn `pr-writer` with `mode: create` per references/pr-writer-rules.md. Key fields:
     - `base_branch`: the previous branch in the stack (or `origin/<base-branch>` for the first)
     - `commit_messages`:
       - First branch: `git log origin/<base>..HEAD --format=%B`
       - Subsequent branches: `git log <previous-stack-branch>..HEAD --format=%B`
     - `branch_context`: contents of the branch context file written in step 4
     - `context`: "Branch N of M in a stacked split. The reference branch (<reference-branch>) holds the final combined state. This branch isolates: <theme>."
     - See references/pr-writer-rules.md for the full delegation field spec.

   Record the PR URL returned for each branch.

6. **Verify** (verify): Run:
   ```
   git diff <last-stack-branch> <reference-branch>
   ```
   If the diff is empty, the split is complete and correct.

   If the diff is non-empty, also run `git diff --stat <last-stack-branch> <reference-branch>` for a quick summary of what differs. Report the discrepancy to the user: list the files that differ and their change counts. Do NOT attempt auto-repair -- stop and let the user decide how to proceed.

7. **Report**: Check out the reference branch: `git checkout <reference-branch>`. Report:
   - All created branches with their PR URLs, estimated sizes, and review focus areas
   - Verification result (complete and correct, or discrepancy details)
   - Reminder that each stacked branch must compile and pass CI, but later branches can refine earlier ones -- the reference branch is the authoritative final state
