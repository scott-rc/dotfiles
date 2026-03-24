# Plan Task

Decompose a large task into ordered chunks with orchestrated subagent execution, producing a plan directory with a master plan and chunk files, and delivering the orchestrator prompt via Claude Code plan mode.

## Instructions

1. **Gather requirements** (conditional):
   Follow the interview pattern from references/content-patterns.md. Skip the interview only when the user's request explicitly provides the overall goal, codebase location, and any constraints or failed approaches. Interview when any of these is missing:
   - The overall goal and desired end state
   - Natural phase boundaries or milestones (if they have any in mind)
   - The codebase location and key directories
   - Build and test commands
   - Any prior decisions, constraints, or failed approaches

2. **Explore codebase context**:
   If a codebase is relevant, spawn a Task subagent (type: Explore, model: sonnet) to gather context. The subagent MUST:
   - Map key file paths, directory structure, and architecture
   - Identify naming conventions, types, and patterns the chunks will reference
   - Stay focused on what the task actually needs -- do not map the entire codebase
   - Note specific function names, type signatures, or modules each chunk will touch
   - Return a concise summary of findings organized by area

   If no codebase applies, MUST skip this step entirely.

3. **Confirm understanding** (conditional):
   Confirm when the agent synthesized information from multiple sources or made non-obvious inferences. Skip when the user's request was already complete and codebase exploration (step 2) added no unexpected context.
   When this step runs:
   - MUST summarize the goal, scope, and codebase context in 3-5 sentences
   - MUST present the summary and ask for confirmation via AskUserQuestion with options: "Looks good", "Needs changes" (description: "I'll describe what to adjust"), "Start over" (description: "Re-gather requirements from scratch")
   - If the user selects "Needs changes", ask what to adjust via AskUserQuestion, update understanding, and re-confirm with the same options
   - If the user selects "Start over", return to step 1
   - MUST NOT proceed to decomposition until the user selects "Looks good"

4. **Decompose into chunks**:
   Spawn a Task subagent (type: Plan, model: sonnet) to propose the chunk decomposition. Supply the subagent with:
   - The approved requirements from step 3 (or the user's original request if step 3 was skipped)
   - The Explore agent's codebase analysis from step 2 (if it ran)

   The Plan subagent MUST:
   - Identify 2-6 chunks that partition the work
   - For each chunk, provide: a one-line description, the list of files it will touch, which prior chunk it depends on (or "None"), a 2-4 sentence summary, and verification steps
   - Follow these chunking guidelines:
     - **Refactor first** -- if the task requires new abstractions or restructuring, make chunk 01 a pure refactor with no behavior change.
     - **One feature per chunk** -- each chunk should add exactly one user-visible capability or complete one logical unit of work. MUST NOT mix unrelated changes.
     - **Buildable after each** -- the codebase MUST build and pass tests after every chunk completes.
     - **~15-25 checkboxes per chunk** -- enough for meaningful progress, few enough to complete in one Claude Code session.
     - **Declare dependencies** -- each chunk's "Depends on" line names the chunk file it requires.
     - **Test first when testable** -- for chunks adding testable behavior, structure step groups as red-green-refactor.
     - **Docs and cleanup last** -- put documentation updates, README changes, and cleanup in the final chunk.
     - **Independently verifiable** -- each chunk's Verification section should confirm its work without relying on later chunks.
   - Return the proposed chunk list with all details

   Present the Plan subagent's proposed decomposition to the user. MUST use AskUserQuestion with options: "Approve chunks", "Request changes" (description: "I'll describe what to adjust"), "Add/remove chunks" (description: "I'll specify which chunks to add or remove")
   If the user selects "Request changes" or "Add/remove chunks", ask what to adjust via AskUserQuestion, revise the decomposition (re-running the Plan subagent if the changes are substantial), and re-present with the same options. MUST NOT proceed to writing chunk files until the user selects "Approve chunks".

5. **Write chunk files**:
   For each approved chunk, write a chunk file at `./tmp/<plan-name>/chunk-NN-<slug>.md` using the format from references/chunk-format.md. Expand the high-level steps into concrete checkboxes using the codebase context for specific file paths, function names, and commands. Use TDD step groups when steps involve testable behavior; use plain step groups for refactoring, config, or glue.

   After writing all chunk files, spawn a Task subagent (type: Explore, model: haiku) to read them all and verify each has the correct structure (sections, checkboxes, TDD structure where appropriate). The subagent MUST return a pass/fail summary per chunk. Split any chunk exceeding 25 checkboxes.

6. **Write master plan**:
   Create `./tmp/<plan-name>/plan.md` using the Master Plan Template from references/plan-template.md.

7. **Validate structure**:
   - MUST verify all chunk file paths in plan.md resolve to actual files
   - MUST verify dependency links between chunks are correct and acyclic
   - MUST verify chunk numbering is sequential with no gaps
   - MUST verify every chunk has at least one checkbox in both Implementation Steps and Verification
   - MUST scan all files for non-ASCII characters and replace with ASCII equivalents

8. **Deliver via plan mode**:
   Enter plan mode via `EnterPlanMode` if not already in it. Write the orchestrator prompt to the plan file specified by the plan mode system message, using the Orchestrator Prompt Template from references/plan-template.md with all `<...>` placeholders filled in. MUST scan the orchestrator prompt content for non-ASCII characters and replace with ASCII equivalents before writing.

   Call `ExitPlanMode` to present the plan for user approval. MUST also list all created chunk files with their paths.
