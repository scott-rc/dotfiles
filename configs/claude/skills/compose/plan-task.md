# Plan Task

Decompose a large task into ordered chunks with orchestrated subagent execution, producing a plan directory with a master plan and chunk files, and delivering the orchestrator prompt via Claude Code plan mode.

## Instructions

1. **Gather requirements**:
   Interview the user about:
   - The overall goal and desired end state
   - Natural phase boundaries or milestones (if they have any in mind)
   - The codebase location and key directories
   - Build and test commands
   - Any prior decisions, constraints, or failed approaches

   MUST batch questions into a single message. MUST skip any questions the user's initial request already answered. SHOULD ask follow-ups only if answers are ambiguous or incomplete.

2. **Load coding preferences** (conditional):
   If the task involves writing or modifying code:
   - MUST invoke the code skill to load coding preferences: `skill: "code"` (no routing args -- loading references only)
   - Read from the code skill's References section: general-guidelines and the applicable language-specific guidelines (TypeScript, Go, or Shell)
   - Read test-environment for test runner detection, file placement, and build/test commands
   - Apply these during decomposition:
     - Default to TDD structure (red-green-refactor) for chunks adding testable behavior
     - Chunks that are pure refactoring, config, or glue code do not need TDD structure
     - Include build and test commands in chunk context

   If no code changes are involved, MUST skip this step.

3. **Explore codebase context**:
   If a codebase is relevant, spawn a Task subagent (type: Explore, model: sonnet) to gather context. The subagent MUST:
   - Map key file paths, directory structure, and architecture
   - Identify naming conventions, types, and patterns the chunks will reference
   - Stay focused on what the task actually needs -- do not map the entire codebase
   - Note specific function names, type signatures, or modules each chunk will touch
   - Return a concise summary of findings organized by area

   If no codebase applies, MUST skip this step entirely.

4. **Confirm understanding**:
   - MUST summarize the goal, scope, and codebase context in 3-5 sentences
   - MUST present the summary and ask for confirmation via AskUserQuestion with options: "Looks good", "Needs changes" (description: "I'll describe what to adjust"), "Start over" (description: "Re-gather requirements from scratch")
   - If the user selects "Needs changes", ask what to adjust, update understanding, and re-confirm with the same options
   - If the user selects "Start over", return to step 1
   - MUST NOT proceed to decomposition until the user selects "Looks good"

5. **Decompose into chunks**:
   Identify 2-6 chunks that partition the work. Present the chunk list with one-line descriptions for user approval.

   MUST read [plan-template.md](plan-template.md) and follow the Chunking Guidelines before decomposing.

   MUST present the chunk list and ask for approval via AskUserQuestion with options: "Approve chunks", "Request changes" (description: "I'll describe what to adjust"), "Add/remove chunks" (description: "I'll specify which chunks to add or remove")
   If the user selects "Request changes" or "Add/remove chunks", ask what to adjust, revise the list, and re-present with the same options. MUST NOT proceed to writing chunk files until the user selects "Approve chunks".

6. **Write chunk files via subagents**:
   For each approved chunk, spawn a Task tool subagent (type: chunk-writer) to write the chunk file. This keeps context manageable and ensures each chunk file gets focused attention.

   For each chunk, launch a chunk-writer subagent with only the chunk-specific fields:
   - Chunk number, title, slug, and one-line description
   - Summary (2-4 sentences from your decomposition)
   - Dependency (prior chunk file name, or "None")
   - High-level steps from your decomposition
   - Codebase context: file paths, function names, types, and patterns discovered during exploration
   - Build and test commands
   - Output file path: `./tmp/<plan-name>/chunk-NN-<slug>.md`

   Run chunk writer subagents in parallel when chunks have no dependency on each other's files. Run sequentially only when a chunk's writer prompt references content from an earlier chunk file.

   Decision point: If all chunks receive their full context from the parent prompt (the typical case), launch all subagents in parallel. If a chunk's writer prompt says "read chunk-NN for context," that chunk must wait for chunk-NN's subagent to finish.

   After all subagents complete, read each chunk file and verify it meets these requirements:
   - Has a "Depends on" line naming its prerequisite chunk (or "None")
   - Has a "What and Why" section with enough context for a fresh Claude Code session
   - Has an "Implementation Steps" section with numbered sub-step groups and `- [ ]` checkboxes
   - Has a "Verification" section with `- [ ]` checkboxes for build, test, and manual checks
   - Has ~15-25 total checkboxes (split the chunk if it exceeds 25)
   - Includes specific file paths, function names, and shell commands -- not vague descriptions
   - Chunks adding testable behavior use TDD structure: step groups named "Red: ...", "Green: ...", "Refactor" with explicit test-run checkboxes confirming failure then success

   If a chunk file fails validation, provide feedback and re-run the subagent.

7. **Write master plan**:
   Create `./tmp/<plan-name>/plan.md` using the Master Plan Template from [plan-template.md](plan-template.md).

8. **Validate**:
   - MUST verify all chunk file paths in plan.md resolve to actual files
   - MUST verify dependency links between chunks are correct and acyclic
   - MUST verify chunk numbering is sequential with no gaps
   - MUST verify every chunk has at least one checkbox in both Implementation Steps and Verification
   - MUST scan all files for non-ASCII characters and replace with ASCII equivalents

9. **Deliver via plan mode**:
   Enter plan mode via `EnterPlanMode` if not already in it. Write the orchestrator prompt to the plan file specified by the plan mode system message, using the Orchestrator Prompt Template from [plan-template.md](plan-template.md) with all `<...>` placeholders filled in. MUST scan the orchestrator prompt content for non-ASCII characters and replace with ASCII equivalents before writing.

   Call `ExitPlanMode` to present the plan for user approval. MUST also list all created chunk files with their paths.
