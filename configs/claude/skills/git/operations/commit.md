# Commit

Commit outstanding changes with a well-formatted message.

## Instructions

1. **Check branch protection**: MUST follow main branch protection per references/git-patterns.md. If on main/master and not dotfiles, present branch options via AskUserQuestion. If chosen, create and switch to the branch before committing. Branch name MUST follow the `sc/` prefix convention defined there.
2. **Determine scope**: identify the session file set -- files modified in this conversation. Skip this step if the user asked to "commit all", "commit everything", or provided their own file list. If `git status` shows modified files outside the session file set, ask the user which files to include before proceeding.
3. **Delegate to the `committer` agent**: MUST delegate -- do not commit inline. If a session file set was determined, pass: "Stage and commit only these files: `<file list>`". Otherwise, pass no additional prompt -- the agent gathers context, drafts a message, stages, and commits autonomously.
4. **If the agent returns `needs-user-input`** (mixed concerns): present the groups from `## Cohesion` as AskUserQuestion options. Re-invoke the agent with: "Stage and commit only these files: `<file list>`".
5. **Report**: show the commit hash and title from the agent's `## Commit` section.
