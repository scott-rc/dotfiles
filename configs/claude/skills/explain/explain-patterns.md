# Explain Patterns Reference

Shared patterns for argument classification, diff thresholds, and output format.

## Argument Classification

Classify each argument by testing in this order:

1. **File** — path exists on disk (`test -e <arg>`) or matches a file in `git diff --name-only`
2. **Commit range** — contains `..` (e.g., `abc123..def456`, `HEAD~3..HEAD`)
3. **Commit** — `git rev-parse --verify <arg>^{commit}` succeeds and `git show-ref --verify refs/heads/<arg>` fails
4. **Branch** — `git show-ref --verify refs/heads/<arg>` or `git show-ref --verify refs/remotes/origin/<arg>` succeeds

If none match, ask the user what they meant.

## Combination Rules

Multiple arguments compose as follows:

- **commit + file** — show only that file's changes in the commit
- **branch + file** — show only that file's changes across the branch diff
- **range + file** — show only that file's changes within the range
- **multiple files** — filter diff to those files
- **multiple commits** — explain each commit separately in chronological order
- **commit + branch** — ambiguous, ask the user

## Diff Size Thresholds

Measured by total added + removed lines in `git diff --stat`:

- **Trivial** — under 20 lines
- **Small** — 20–200 lines
- **Medium** — 200–500 lines
- **Large** — 500+ lines

## Large Diff Strategy

When a diff exceeds 500 lines:

1. Group changed files by theme (same package, same feature, test files together)
2. Spawn one Task subagent (subagent_type: `Explore`) per group with prompt: "Read these files and summarize what changed and why: `<file list>`"
3. Collect subagent summaries
4. Synthesize into a single three-layer explanation

## Output Structure

### Trivial / Small

```
## <title summarizing the change>

<Why + What + How in 1–3 paragraphs>
```

### Medium / Large

```
## <title summarizing the change>

### Why
<Motivation, linked issues, PR context>

### What
<Concrete changes grouped by theme>
- **<theme>**: <changes>
- **<theme>**: <changes>

### How
<Implementation details, key decisions, patterns>
```

Adapt freely within these structures — they are guidelines, not rigid templates. Shorter is better when the diff is straightforward.
