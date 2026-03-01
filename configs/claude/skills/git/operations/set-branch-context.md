# Set Branch Context

Read or create the branch context file that captures the "why" for the current branch.

## Instructions

1. **Check branch**: If on main/master, inform user that branch context is for feature branches and stop.

2. **Check for existing file**: Sanitize the branch name (replace `/` with `--`) and check if `tmp/branches/<sanitized-branch>.md` exists. If it exists, display its contents and ask if the user wants to update it. If they decline, stop.

3. **Gather context**: Prompt via AskUserQuestion -- "What's the purpose of this branch?" with exactly these three options (MUST NOT substitute domain-specific alternatives -- they are intentionally domain-agnostic so they work consistently across all repos and contexts):
   - **"I know"** — user provides the purpose directly. Optionally ask "Any related links (issues, PRs, Slack)?" with a "Skip" option.
   - **"Help me articulate it"** — run a quick inline exploration. Ask 2-3 targeted questions via AskUserQuestion: "What problem are you solving or what triggered this work?", "What's the expected outcome when this branch merges?", and optionally "Any related issues, PRs, or links?". Synthesize the answers into a concise purpose statement (1-3 sentences) plus links.
   - **"Skip"** — stop without creating the file.

4. **Write the file**: `mkdir -p tmp/branches` and write to `tmp/branches/<sanitized-branch>.md`. The file MUST contain only:
   - 1-3 sentences of purpose/motivation (the "why")
   - Related links, if given (each on its own line)

   Do NOT include headers, change lists, implementation details, or what files were modified -- the diff is the source of truth for "what". Keep the user's original phrasing where possible.

5. **Report**: Confirm the file was written and show its contents.
