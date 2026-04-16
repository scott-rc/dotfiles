# Create Handoff

Write a self-contained task description that captures the current session's context and the user's stated forward-looking intent, then deliver it via plan mode so the user can accept and continue in a fresh context.

## Instructions

1. **Gather intent from the conversation**:
   Read the conversation to extract:
   - What the user said they want the next session to do (take this verbatim -- do not decompose or pre-research it)
   - What has been completed or decided in this session
   - Any constraints, gotchas, or blockers already discovered

   If the user provided no context at all (e.g., just `/compose handoff` with nothing else), ask the user: what should the next session accomplish? MUST batch all questions into a single prompt. MUST skip anything the conversation history already answers.

   MUST NOT investigate, explore, or research on behalf of the next session. If the user said "figure out how X works", write that as-is -- do not figure it out now.

2. **Collect ambient state** (conditional):
   If the task involves a codebase and the information is already available in the conversation, note:
   - Current branch and uncommitted changes (run `git status --short` and `git branch --show-current` if not already known -- these are quick reads, not investigation)

   MUST NOT run Explore agents, read source files, or gather file lists. If the next session needs to explore the codebase, say so in Remaining Work.

3. **Deliver via plan mode**:
   Enter plan mode via `EnterPlanMode` if not already in it. Write the handoff content to the plan file specified by the plan mode system message.

   Use this structure (MUST omit sections that don't apply):

   - **Goal** -- what the next session should accomplish, in 1-2 sentences
   - **Background** -- why this work is happening, prior decisions, and context the next session needs to understand the task
   - **Current State** -- what has been completed, current branch, uncommitted changes, any in-progress work
   - **Remaining Work** -- numbered list preserving the user's stated intent verbatim. Write "Figure out how X works" if that's what the user said -- not a pre-researched breakdown of X. Add concrete file paths, commands, or specifics only when already known from this session.
   - **Constraints** -- boundaries, anti-requirements, things to avoid, gotchas discovered during this session
   - **Key Files** -- files already identified as relevant, with brief role descriptions. MUST NOT include files discovered by pre-investigation.
   - **Commands** -- build, test, lint, and other relevant commands already known

   Content rules:
   - The next agent starts a fresh session with no memory of this conversation. MUST write as if the reader has zero prior context. Every file path, function name, and decision MUST be stated explicitly -- never "the file we discussed" or "as mentioned earlier"
   - MUST preserve the user's intent in Remaining Work rather than decomposing or researching it. If they said "figure out X", write "Figure out X"
   - SHOULD keep the content under ~80 lines. If it would exceed that, the task may need Plan Task instead
   - MUST scan for non-ASCII characters and replace with ASCII equivalents before writing

4. **Verify plan completeness**:
   Verify the plan file contains at least a Goal and Remaining Work section (both are always required for a handoff). If either is missing, add it before proceeding.

5. **Present for approval**:
   Call `ExitPlanMode` to present the handoff for user approval.
