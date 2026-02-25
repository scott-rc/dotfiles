# Commit Operation

Commit outstanding changes with a well-formatted message.

## Instructions

1. **Check current branch**:
   Check main branch protection per [git-patterns.md](git-patterns.md). If on main/master and not dotfiles, present branch options via AskUserQuestion. If a branch is chosen, create and switch to it before committing.

2. **Delegate to the `committer` agent**. Pass no additional prompt — the agent gathers context, drafts a message, stages, and commits autonomously.

3. **If the agent returns `needs-user-input`** (mixed concerns): present the groups from `## Cohesion` as AskUserQuestion options. Then re-invoke the agent with: "Stage and commit only these files: `<file list>`".

4. **Report**: show the commit hash and title from the agent's `## Commit` section.

See [git-patterns.md](git-patterns.md) for dotfiles exception and main branch protection patterns.
