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
   - Branch diff: merge-base with the base branch. Run `fish -c 'gbb' 2>/dev/null || git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||'` to detect it. Use `git merge-base origin/<base> HEAD`.
   - Single commit: `<commit>~1`
   - Commit range: use the range directly
   - File-only (no commit/branch): diff against the default branch merge-base

3. **Gather commit context**:
   Spawn the `github-context` agent with: ref_range set to `<base>..HEAD` (or the resolved range), include_pr true, include_issues true.
   The agent returns: commit list, PR title/motivation, referenced issue context, and a motivation summary.

4. **Get the diff**:
   - `git diff --stat <base>...<target>` for overview
   - `git diff <base>...<target>` for full diff (add `-- <files>` if file arguments were given)
   - Note the total added + removed lines from `git diff --stat` for threshold classification

5. **Read changed files**:
   Read full files (not just diff hunks) to understand surrounding context.
   - **Small diffs** (under 500 diff lines): read each changed file inline using Read tool
   - **Large diffs** (500+ diff lines): group files by theme (e.g., package, feature area, test vs. production), spawn one Task subagent per group to read files and summarize changes, then synthesize results. See large diff strategy in [explain-patterns.md](explain-patterns.md).

6. **Compose explanation**:
   Produce three layers adapted to diff size (see thresholds in [explain-patterns.md](explain-patterns.md)):
   - **Why** — motivation and context: what problem is being solved, what triggered the change
   - **What** — concrete changes grouped by theme: new files, modified behavior, removed code
   - **How** — implementation details: key algorithms, patterns used, non-obvious decisions

   Apply the depth strategy from [explain-patterns.md](explain-patterns.md) based on diff size.

7. **Verify completeness**:
   Check that the explanation covers all changed files from the diff stat. If any files are missing, add them. Verify the Why traces back to at least one source (PR body, issue, or commit message).

8. **Print**:
   Output the explanation directly to the terminal. MUST NOT offer to save, copy, or share.
