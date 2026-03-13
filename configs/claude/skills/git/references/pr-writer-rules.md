# PR Title and Description Writing Rules

Rules for writing PR titles and descriptions inline.

## Required Context

Gather these before writing a PR title and description:

- `mode`: `create` or `update`
- `base_branch`: detect per references/git-patterns.md Base Branch Detection
- `pr_number`: required for `update` mode
- `commit_messages`: all branch commit messages verbatim (see Commit Message Forwarding below)
- `branch_context` (optional): contents of the branch context file (path per references/git-patterns.md "Branch Context File") -- purpose statement and links captured at branch creation. When present, this is the primary source for the PR's motivation/narrative; commit messages remain supplementary hints.
- `context` (optional): one sentence describing what changed in this particular update. When `branch_context` is present, do NOT restate the branch purpose here -- instead describe what's new (e.g., "addresses review feedback on error handling", "adds test coverage for edge cases"). If nothing substantive changed (e.g., post-squash cleanup), omit this field entirely.

## Commit Message Forwarding

- MUST read all branch commit messages (`git log origin/<base>..HEAD --format=%B`) and use them as supplementary context
- Commit messages are supplementary hints -- they provide context but are NOT the source of truth; the diff is
- Commit messages often describe intermediate states (fixups, reverts, mid-PR bugs that were later corrected) that MUST NOT appear in the PR description
- After a squash this is a single message; with multiple commits it is the full set

## Cardinal Rules

These override everything else. Every title and description MUST follow them.

1. **The diff is the source of truth.** Every factual claim in the description MUST be verifiable against `git diff`. With multiple commits, commit messages often describe intermediate states -- fixups, reverts, mid-PR bugs that were later corrected -- and MUST NOT drive the description. After a squash, the single commit message is the author's deliberate summary of the final state; use it as a structural template (its organization, grouping, and emphasis) while still verifying every claim against the diff.
2. **Describe the net change, not the journey.** If a bug was introduced in commit 1 and fixed in commit 3, do NOT mention the bug. The PR describes the end state, not intermediate steps.
3. **Branch context provides the "why" -- not the "what".** When `branch_context` is present, it is the primary source for motivation and narrative framing. However, branch context may contain inaccurate factual claims about before/after states (types, signatures, behavior). The diff is the sole source of truth for what the code looked like before and after. If branch context claims "X was raw strings" but the diff shows branded types, the diff wins.

## Title

- Imperative mood, under 70 characters
- Specific: name the feature, fix, or change
- Derived from the diff, not copied from commit subjects

## Description

- **Prose by default, structure when earned**: A typical PR is readable paragraphs. When a PR addresses 3+ distinct problems or concerns, use a numbered list so each is clearly delineated -- each item should be a concise problem/solution pair (2-3 sentences). Use bullets for ancillary items (secondary fixes, cleanups) that don't warrant their own paragraph. Semicolons joining 4+ independent clauses in one sentence are always worse than bullets.
- **No boilerplate headers**: Do NOT use `## Summary`, `## Test plan`, `## Changes`, or similar generic headers. For multi-concern PRs, an introductory sentence plus a numbered list provides enough structure without headers.
- **MUST NOT wrap lines**: Do NOT wrap text to 72 characters. Write each thought as one continuous line. GitHub handles wrapping.
- **Focus on the "why"**: Explain motivation and reasoning, not just what changed.
- **Testing and verification woven into the narrative**: Mention test coverage inline as part of the prose. Do NOT put it in a separate section or checklist. If the PR adds CI workflows, deployment pipelines, or build configuration, state how to verify locally (e.g., build command) and what triggers in CI (e.g., "deploys on push to main when docs/ changes").
- **Link issues**: Use "Fixes #123" to auto-close; use "Related to #456" for referenced-but-not-fixed issues.
- **Be concise**: A few sentences for a typical PR. For multi-concern PRs, prefer tighter items over more paragraphs -- a numbered list of 3 two-sentence items is more readable than 3 dense paragraphs. The diff has the details; the description orients the reviewer.
- **Don't enumerate artifacts**: Do NOT list every type, helper, module, function, or file path introduced. Name only what a reviewer needs to orient themselves.
- **Reviewer-first**: The description helps reviewers understand why the change exists and how to approach reviewing it. It is not a changelog or API reference.

## ASCII and Posting

See references/github-text.md for ASCII-only rules and sanitize script usage. Apply those rules to all PR title and body content before posting.

## Workflow

1. **Gather diff context**:

   In update mode, first resolve the head branch from the PR:

   ```bash
   HEAD_BRANCH=$(gh pr view <pr_number> --json headRefName -q .headRefName)
   ```

   Then diff against that branch (update mode) or HEAD (create mode):

   ```bash
   # Update mode
   git diff --stat origin/<base_branch>...$HEAD_BRANCH
   git diff origin/<base_branch>...$HEAD_BRANCH

   # Create mode
   git diff --stat origin/<base_branch>...HEAD
   git diff origin/<base_branch>...HEAD
   ```

   If the diff is large (>500 lines), use `--stat` for overview and read selectively. Ensure every file group visible in `--stat` is represented in the description -- do not silently omit categories (e.g., tooling configs, CI workflows, project rules) because they seem ancillary to the primary change.

2. **Draft title and body**:
   Write the title and body following the rules above. If `branch_context` is present, use it as the primary narrative source for motivation and framing. If `context` is present, incorporate it naturally.

3. **Create or update**:

   Write the body and title to PR-specific temp file paths to avoid clobbering when updating multiple PRs.

   **Update mode** -- use the PR number as the suffix:

   ```bash
   mkdir -p ./tmp && cat <<'EOF' > ./tmp/pr-<pr_number>-body.txt
   ...
   EOF
   ~/.claude/skills/git/scripts/sanitize.sh ./tmp/pr-<pr_number>-body.txt
   ~/.claude/skills/git/scripts/sanitize.sh --title ./tmp/pr-<pr_number>-title.txt
   TITLE=$(cat ./tmp/pr-<pr_number>-title.txt)
   ```

   - Fetch current body: `gh pr view <pr_number> --json body -q .body`
   - If the existing body contains bot-appended content (sections not in your new description, e.g., Cursor BugBot, Dependabot), append it to the new body
   - **Before posting**, verify every factual claim in your new draft against the diff. For claims about before/after states (types, signatures, behavior), find the corresponding `-` and `+` lines in the diff and confirm they match. Remove or correct claims that don't match the net change (e.g., "removed from both call sites" when only one existed, "raw strings" when the diff shows branded types, or journey language like "was flaky" for code that is entirely new in the PR). Do not trust branch context or commit messages for before/after facts -- only the diff.

   ```bash
   gh pr edit <pr_number> --title "$TITLE" --body-file ./tmp/pr-<pr_number>-body.txt
   ```

   **Create mode** -- use the base branch name (slashes replaced with dashes) as the suffix:

   ```bash
   BRANCH_SLUG=$(echo "<base_branch>" | tr '/' '-')
   mkdir -p ./tmp && cat <<'EOF' > ./tmp/pr-${BRANCH_SLUG}-body.txt
   ...
   EOF
   ~/.claude/skills/git/scripts/sanitize.sh ./tmp/pr-${BRANCH_SLUG}-body.txt
   ~/.claude/skills/git/scripts/sanitize.sh --title ./tmp/pr-${BRANCH_SLUG}-title.txt
   TITLE=$(cat ./tmp/pr-${BRANCH_SLUG}-title.txt)
   gh pr create --title "$TITLE" --base <base_branch> --body-file ./tmp/pr-${BRANCH_SLUG}-body.txt
   ```

   Clean up the PR-specific temp files after posting.

## Stacked PR Batch Updates

When updating descriptions for multiple PRs in a stack:

- Use PR-specific temp file paths (with PR number or branch slug) to avoid clobbering when writing multiple descriptions
- After all updates complete, verify descriptions are distinct: fetch titles and first body lines via `gh pr view <number> --json title,body` for each PR and confirm they are not identical
- If duplicates are found, re-draft and re-post the affected descriptions
