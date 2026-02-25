# Commit Operation

Commit outstanding changes with a well-formatted message.

## Instructions

1. **Check current branch**:
   Check main branch protection per [git-patterns.md](git-patterns.md). If on main/master and not dotfiles, present branch options via AskUserQuestion. If a branch is chosen, create and switch to it before committing.

2. **Analyze changes** by spawning the `commit-analyzer` agent. Pass no additional prompt — the agent gathers its own context via git commands. It returns a structured summary with `## Changes`, `## Cohesion`, `## Commit Message`, and `## Staging` sections.

3. **Stage files** based on the agent's `## Staging` section:
   - If "all": `git add -A`
   - If "selective": present the groups from `## Cohesion` as AskUserQuestion options, then `git add <files>` for the chosen group

4. **Create commit** using the draft from `## Commit Message`, refined if needed, following [commit-guidelines.md](commit-guidelines.md) (including its Shell-Safe Application rules).

5. **If commit fails due to a pre-commit hook**: read the error output, fix the issue, re-stage changes, and retry the commit. MUST NOT use `--no-verify`.

6. **Report**: show the commit hash and title (`git log -1 --oneline`).

See [git-patterns.md](git-patterns.md) for dotfiles exception and main branch protection patterns.
