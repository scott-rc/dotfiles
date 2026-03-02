# Set Branch Context

Read or create the branch context file that captures the "why" for the current branch.

## Instructions

1. **Check branch**: If on main/master, inform user that branch context is for feature branches and stop.

2. **Check for existing file**: Check if the branch context file exists (path per references/git-patterns.md "Branch Context File"). If it exists, display its contents and ask if the user wants to update it. If they decline, stop.

3. **Assess conversation context**: Before prompting the user, assess whether the current conversation already contains enough information to draft a meaningful branch context — problem discussed, motivation clear, relevant links shared. If the conversation lacks sufficient context (e.g., user invoked set-branch-context directly at the start of a session with no prior discussion), fall through to step 4.

4. **Draft from conversation**: If context is sufficient, draft a branch context using the same content rules as step 7: 1-3 sentences of purpose/motivation, related links if discussed, no headers/change lists/implementation details. Cross-check any factual claims about before/after states against `git diff origin/<base>..HEAD` — the diff is the source of truth for what the code looked like. Then skip to step 8 (confirm with user).

5. **Gather context**: Prompt via AskUserQuestion -- "What's the purpose of this branch?" with exactly these three options (MUST NOT substitute domain-specific alternatives -- they are intentionally domain-agnostic so they work consistently across all repos and contexts):
   - **"I know"** — user provides the purpose directly. Optionally ask "Any related links (issues, PRs, Slack)?" with a "Skip" option. If their description includes factual claims about before/after states, cross-check against the diff before writing (same as "Help me articulate it").
   - **"Help me articulate it"** — proceed to step 6.
   - **"Skip"** — write `N/A` as the file content (same path and `mkdir` logic as below) and proceed to step 9 (skip confirmation).

6. **Ask targeted questions**: Ask via AskUserQuestion: "What problem are you solving or what triggered this work?". Then ask "What's the expected outcome when this branch merges?". Then ask "Any related issues, PRs, or links?" with a "Skip" option.

7. **Synthesize and cross-check**: Synthesize the answers into a concise purpose statement (1-3 sentences) plus any links provided. Cross-check any factual claims about before/after states (types, signatures, behavior) against `git diff origin/<base>..HEAD` — the diff is the source of truth for what the code looked like. Proceed to step 8 to write the file.

8. **Write the file**: `mkdir -p ./tmp/branches` and write to `./tmp/branches/<sanitized-branch>.md`. The file MUST contain only:
   - 1-3 sentences of purpose/motivation (the "why")
   - Related links, if given (each on its own line)

   Do NOT include headers, change lists, implementation details, or what files were modified -- the diff is the source of truth for "what". Keep the user's original phrasing where possible.

9. **Confirm with user**: Show the written content and ask via AskUserQuestion -- "Does this accurately capture the purpose?" with options:
   - **"Looks good"** — proceed to report.
   - **"Needs changes"** — user provides corrections; update the file and re-confirm.

10. **Report**: Confirm the file was written and show its contents.
