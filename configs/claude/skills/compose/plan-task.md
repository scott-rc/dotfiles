# Plan Task

Decompose a large task into ordered chunks with orchestrated subagent execution, producing a plan directory with a master plan, chunk files, and a ready-to-paste orchestrator prompt.

## Instructions

1. **Gather requirements**:
   Interview the user about:
   - The overall goal and desired end state
   - Natural phase boundaries or milestones (if they have any in mind)
   - The codebase location and key directories
   - Build and test commands
   - Any prior decisions, constraints, or failed approaches

   MUST batch questions into a single message. MUST skip any questions the user's initial request already answered. SHOULD ask follow-ups only if answers are ambiguous or incomplete.

2. **Explore codebase context**:
   If a codebase is relevant:
   - MUST explore key file paths, directory structure, and architecture
   - SHOULD identify naming conventions, types, and patterns the chunks will reference
   - MUST stay focused on what the task actually needs -- do not map the entire codebase
   - SHOULD note specific function names, type signatures, or modules each chunk will touch

   If no codebase applies, MUST skip this step entirely.

3. **Confirm understanding**:
   - MUST summarize the goal, scope, and codebase context in 3-5 sentences
   - MUST NOT proceed to decomposition until the user confirms the summary is accurate
   - If the user corrects anything, update understanding and re-confirm

4. **Decompose into chunks**:
   Identify 2-6 chunks that partition the work. Present the chunk list with one-line descriptions for user approval.

   MUST read [plan-template.md](plan-template.md) and follow the Chunking Guidelines before decomposing.

   MUST NOT proceed to writing chunk files until the user approves the chunk list. If the user requests changes, revise and re-present.

5. **Write chunk files**:
   Create `./tmp/<plan-name>/chunk-NN-<slug>.md` for each approved chunk using the Chunk File Template from [plan-template.md](plan-template.md).

   Each chunk file MUST have:
   - A "Depends on" line naming its prerequisite chunk (or "None")
   - A "What and Why" section with enough context for a fresh agent session
   - An "Implementation Steps" section with numbered sub-step groups and `- [ ]` checkboxes
   - A "Verification" section with `- [ ]` checkboxes for build, test, and manual checks
   - ~15-25 total checkboxes (split the chunk if it exceeds 25)

   MUST include specific file paths, function names, and shell commands -- not vague descriptions. Each checkbox SHOULD be completable in a single focused action.

6. **Write master plan**:
   Create `./tmp/<plan-name>/plan.md` using the Master Plan Template from [plan-template.md](plan-template.md).

   The orchestrator prompt code block MUST use the Orchestrator Prompt Template from plan-template.md, with all `<...>` placeholders filled in using details from the interview and exploration steps.

7. **Validate**:
   - MUST verify all chunk file paths in plan.md resolve to actual files
   - MUST verify dependency links between chunks are correct and acyclic
   - MUST verify chunk numbering is sequential with no gaps
   - MUST verify every chunk has at least one checkbox in both Implementation Steps and Verification
   - MUST scan all files for non-ASCII characters and replace with ASCII equivalents

8. **Deliver**:
   - MUST print the orchestrator prompt inside a markdown code block
   - MUST copy the orchestrator prompt to the clipboard via `pbcopy`
   - MUST list all created files with their paths
   - MUST tell the user the prompt is copied and ready to paste into a new session
