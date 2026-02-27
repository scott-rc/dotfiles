# Create Handoff

Write a self-contained task description that captures the current session's context, decisions, and remaining work, then deliver it via plan mode so the user can accept and continue in a fresh context.

## Instructions

1. **Identify what to hand off**:
   Determine the scope from the conversation history:
   - What was the original goal or task?
   - What decisions have been made?
   - What has been completed so far?
   - What remains to be done?
   - What was tried and didn't work (if applicable)?

   If the conversation context is insufficient, interview the user via AskUserQuestion:
   - What should the next session accomplish?
   - Are there constraints or gotchas the next session needs to know?

   MUST use AskUserQuestion for all questions. MUST batch questions into a single message. MUST skip questions the conversation history already answers.

2. **Capture codebase state** (conditional):
   If the task involves a codebase, gather:
   - Current branch, uncommitted changes (`git status`, `git diff --stat`)
   - Key file paths and functions relevant to the remaining work
   - Build and test commands
   - Environment or setup requirements

   SHOULD use a Task subagent (type: Explore, model: haiku) for codebase exploration if more than 3 files are involved.

   If no codebase applies, MUST skip this step.

3. **Confirm scope** (conditional):
   Confirm when scope is ambiguous -- the goal, what's done, or what remains is unclear, or the handoff would omit decisions or constraints the template sections don't naturally capture. Skip when the conversation context clearly establishes all three.
   When this step runs:
   - MUST summarize what the handoff will contain in 2-4 sentences
   - MUST present the summary and ask for confirmation via AskUserQuestion with options: "Looks good", "Needs changes" (description: "I'll describe what to adjust")
   - If the user selects "Needs changes", ask what to adjust via AskUserQuestion, update, and re-confirm with the same options
   - MUST NOT proceed to writing until the user confirms

4. **Deliver via plan mode**:
   Enter plan mode via `EnterPlanMode` if not already in it. Write the handoff content to the plan file specified by the plan mode system message.

   Use this structure (MUST omit sections that don't apply):

   - **Goal** -- what the next session should accomplish, in 1-2 sentences
   - **Background** -- why this work is happening, prior decisions, and context the next session needs to understand the task
   - **Current State** -- what has been completed, current branch, uncommitted changes, any in-progress work
   - **Remaining Work** -- numbered list of concrete steps still to be done, with specific file paths, function names, and shell commands
   - **Constraints** -- boundaries, anti-requirements, things to avoid, gotchas discovered during this session
   - **Key Files** -- list of files the next session will need to read or modify, with brief descriptions of their role
   - **Commands** -- build, test, lint, and other relevant commands

   Content rules:
   - MUST write as if the reader has zero prior context. Every file path, function name, and decision MUST be stated explicitly -- never "the file we discussed" or "as mentioned earlier"
   - MUST use imperative voice for remaining work items ("Add error handling to...", "Update the test in...")
   - SHOULD keep the content under ~80 lines. If it would exceed that, the task may need Plan Task instead
   - MUST scan for non-ASCII characters and replace with ASCII equivalents before writing

5. **Verify plan completeness**:
   Verify the plan file contains at least a Goal and Remaining Work section (both are always required for a handoff). If either is missing, add it before proceeding.

6. **Present for approval**:
   Call `ExitPlanMode` to present the handoff for user approval.
