# Explain Operation

Parse arguments, gather context, read full changed files, and produce a layered explanation of code changes.

## Dynamic Context

Current branch and recent history:

!`git branch --show-current`

!`git log --oneline -20`

## Instructions

1. **Parse arguments**:
   Classify each argument in `$ARGUMENTS` using the rules in [explain-patterns.md](explain-patterns.md).
   - No arguments = explain full branch diff against base
   - Ambiguous argument = present possible interpretations as AskUserQuestion options
   - Multiple arguments compose per the combination rules in explain-patterns.md

2. **Detect base**:
   Determine the comparison base:
   - Branch diff: merge-base with the base branch. Run `fish -c 'gbb'` to detect it. Use `git merge-base origin/(gbb) HEAD`.
   - Single commit: `<commit>~1`
   - Commit range: use the range directly
   - File-only (no commit/branch): diff against the default branch merge-base

3. **Gather commit context**:
   Spawn a Task subagent (type: general-purpose, model: haiku) to gather all commit context. The subagent MUST:
   - Run `git log --format='%h %s%n%n%b' <base>..HEAD` (or the resolved range)
   - If a GitHub PR exists for the branch: `gh pr view --json title,body,url` -- extract motivation from the description
   - If the PR body or commit messages reference issues (e.g., `#123`, `fixes #456`): fetch each with `gh issue view <number> --json title,body` and extract relevant context
   - Return a concise summary: PR title/motivation, commit list with subjects, and relevant issue context. If no PR exists, return commit messages only.

4. **Get the diff**:
   - `git diff --stat <base>...<target>` for overview
   - `git diff <base>...<target>` for full diff (add `-- <files>` if file arguments were given)
   - Note the total line count of the diff for threshold classification

5. **Read changed files**:
   Read full files (not just diff hunks) to understand surrounding context.
   - **Small diffs** (under 500 diff lines): read each changed file inline using Read tool
   - **Large diffs** (500+ diff lines): group files by theme (e.g., package, feature area, test vs. production), spawn one Task subagent per group to read files and summarize changes, then synthesize results. See large diff strategy in [explain-patterns.md](explain-patterns.md).

6. **Compose explanation**:
   Produce three layers adapted to diff size (see thresholds in [explain-patterns.md](explain-patterns.md)):
   - **Why** — motivation and context: what problem is being solved, what triggered the change
   - **What** — concrete changes grouped by theme: new files, modified behavior, removed code
   - **How** — implementation details: key algorithms, patterns used, non-obvious decisions

   Adapt depth per diff size:
   - Trivial (< 20 lines): one paragraph combining all layers
   - Small (20–200 lines): short Why, bulleted What, brief How
   - Medium (200–500 lines): full three-layer treatment
   - Large (500+ lines): full treatment with themed sub-sections under What

7. **Print**:
   Output the explanation directly to the terminal. MUST NOT offer to save, copy, or share.
