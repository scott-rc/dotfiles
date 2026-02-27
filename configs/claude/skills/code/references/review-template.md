# Review Template

Templates for review subagent prompts. Replace `<...>` placeholders with actual content.

## Review Subagent Prompt Template

```markdown
Review the following scope of code changes.

## Scope

- **Name**: <scope-name>
- **Files**:
<file-list, one per line>
- **Focus**: <what to pay attention to in this scope>
- **Scope-specific criteria**: <additional criteria beyond the standard checklist>

## Guidelines

Read and apply these guidelines:
- <path-to-general-guidelines.md>
- <path-to-testing-guidelines.md>
- <path-to-language-specific-guidelines.md, if applicable — omit if not applicable>

## Project Context

<repo root, key paths, conventions, patterns, and project-specific standards observed during guideline/context loading>

## Review Checklist

<contents of review-checklist.md>

## Output Format

Report findings grouped by severity (issues first, then suggestions, then nits). Each finding MUST include:
- `file_path:line_number`
- What the problem is (one sentence)
- A concrete fix or recommendation

If no findings in this scope, say so — do not manufacture issues.
```

## Consolidated Report Format

Structure for the merged findings report after all subagents complete.

```markdown
# Review Findings

## Issues

- **`<file_path>:<line>`** — <problem description>
  Fix: <concrete fix or recommendation>

## Suggestions

- **`<file_path>:<line>`** — <problem description>
  Fix: <concrete fix or recommendation>

## Nits

- **`<file_path>:<line>`** — <problem description>
  Fix: <concrete fix or recommendation>
```
