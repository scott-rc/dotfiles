# Commit

Commit outstanding changes with a well-formatted message.

## Instructions

1. **Check branch protection**: MUST follow main branch protection per references/git-patterns.md. If on main/master and not dotfiles, present branch options via AskUserQuestion. If chosen, create and switch to the branch before committing. Branch name MUST follow the `sc/` prefix convention defined there.
2. **Branch context file (orchestrator -- do NOT delegate)**: If on main/master, skip this step. Otherwise, check if the branch context file exists (path per references/git-patterns.md "Branch Context File"). If missing, MUST run the set-branch-context operation **before** proceeding to step 3. This step requires user interaction and MUST complete at the orchestrator level -- the committer subagent cannot prompt the user.
3. **Determine scope**: MUST identify the session file set -- files modified in this conversation. Skip this step if the user asked to "commit all", "commit everything", or provided their own file list. If `git status` shows modified files outside the session file set, ask the user which files to include before proceeding. If the user says to skip the extra files, proceed with only the session file set.
4. **Evaluate complexity**: A commit is **simple** when ALL are true:
   - Session file set is known (from this conversation) or user provided a file list
   - 5 or fewer files
   - `git diff --stat <files>` shows 100 or fewer lines changed total
   If any condition is false, the commit is **complex**.
5. **Commit (simple -- inline)**: Stage files, draft message, and commit per the Inline Commit Procedure in references/commit-message-format.md.
6. **Commit (complex -- delegate)**: Delegate to the `committer` agent. If a session file set was determined, pass: "Stage and commit only these files: `<file list>`". Otherwise, pass no additional prompt -- the agent gathers context, drafts a message, stages, and commits autonomously. MUST NOT pass branch context, summaries, or change descriptions to the committer -- it reads the diff itself.
7. **If the agent returns `needs-user-input`** (mixed concerns): present the groups from `## Cohesion` as AskUserQuestion options. Re-invoke the agent with: "Stage and commit only these files: `<file list>`".
8. **Report**: show the commit hash and title from the agent's `## Commit` section (complex path) or from `git log -1 --oneline` (simple path, already reported in step 5).
