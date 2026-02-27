# Plan Task

Decompose a large task into ordered chunks with orchestrated subagent execution, producing a plan directory with a master plan and chunk files, and delivering the orchestrator prompt via Claude Code plan mode.

## Instructions

1. **Gather requirements** (conditional):
   Follow the interview pattern from [content-patterns.md](references/content-patterns.md). Skip the interview only when the user's request explicitly provides the overall goal, codebase location, and any constraints or failed approaches. Interview when any of these is missing:
   - The overall goal and desired end state
   - Natural phase boundaries or milestones (if they have any in mind)
   - The codebase location and key directories
   - Build and test commands
   - Any prior decisions, constraints, or failed approaches

2. **Load coding preferences** (conditional):
   If the task involves code changes, invoke the code skill to load coding preferences. Otherwise, skip.

3. **Explore codebase context**:
   If a codebase is relevant, spawn a Task subagent (type: Explore, model: sonnet) to gather context. The subagent MUST:
   - Map key file paths, directory structure, and architecture
   - Identify naming conventions, types, and patterns the chunks will reference
   - Stay focused on what the task actually needs -- do not map the entire codebase
   - Note specific function names, type signatures, or modules each chunk will touch
   - Return a concise summary of findings organized by area

   If no codebase applies, MUST skip this step entirely.

4. **Confirm understanding** (conditional):
   Confirm when the agent synthesized information from multiple sources or made non-obvious inferences. Skip when the user's request was already complete and codebase exploration (step 3) added no unexpected context.
   When this step runs:
   - MUST summarize the goal, scope, and codebase context in 3-5 sentences
   - MUST present the summary and ask for confirmation via AskUserQuestion with options: "Looks good", "Needs changes" (description: "I'll describe what to adjust"), "Start over" (description: "Re-gather requirements from scratch")
   - If the user selects "Needs changes", ask what to adjust via AskUserQuestion, update understanding, and re-confirm with the same options
   - If the user selects "Start over", return to step 1
   - MUST NOT proceed to decomposition until the user selects "Looks good"

5. **Decompose into chunks**:
   Spawn a Task subagent (type: Plan, model: sonnet) to propose the chunk decomposition. Supply the subagent with:
   - The approved requirements from step 4 (or the user's original request if step 4 was skipped)
   - The Explore agent's codebase analysis from step 3 (if it ran)
   - The coding preferences from step 2 (if they were loaded)

   The Plan subagent MUST:
   - Identify 2-6 chunks that partition the work
   - For each chunk, provide: a one-line description, the list of files it will touch, which prior chunk it depends on (or "None"), a 2-4 sentence summary, and verification steps
   - Follow these chunking guidelines:
     1. **Refactor first** -- if the task requires new abstractions or restructuring, make chunk 01 a pure refactor with no behavior change. This gives later chunks a clean foundation.
     2. **One feature per chunk** -- each chunk should add exactly one user-visible capability or complete one logical unit of work. Do not mix unrelated changes.
     3. **Buildable after each** -- the codebase MUST build and pass tests after every chunk completes. Never leave the codebase in a broken intermediate state.
     4. **~15-25 checkboxes per chunk** -- enough for meaningful progress, few enough to complete in one Claude Code session. If a chunk exceeds 25, split it.
     5. **Declare dependencies** -- each chunk's "Depends on" line names the chunk file it requires. Chunk 01 depends on "None". Keep the dependency chain linear when possible.
     6. **Test first when testable** -- for chunks adding testable behavior, structure step groups as red-green-refactor: "Red" (write failing tests), "Green" (implement to pass), "Refactor" (clean up). Include explicit test-run checkboxes to confirm failure then success. Chunks that are pure refactoring, config, or glue code use plain step groups.
     7. **Docs and cleanup last** -- put documentation updates, README changes, and cleanup in the final chunk. Earlier chunks focus on implementation.
     8. **Independently verifiable** -- each chunk's Verification section should confirm its work without relying on later chunks. A reviewer should be able to check one chunk in isolation.
   - Return the proposed chunk list with all details

   Present the Plan subagent's proposed decomposition to the user. MUST use AskUserQuestion with options: "Approve chunks", "Request changes" (description: "I'll describe what to adjust"), "Add/remove chunks" (description: "I'll specify which chunks to add or remove")
   If the user selects "Request changes" or "Add/remove chunks", ask what to adjust via AskUserQuestion, revise the decomposition (re-running the Plan subagent if the changes are substantial), and re-present with the same options. MUST NOT proceed to writing chunk files until the user selects "Approve chunks".

6. **Write chunk files via subagents**:
   For each approved chunk, spawn a Task tool subagent (type: chunk-writer) to write the chunk file. This keeps context manageable and ensures each chunk file gets focused attention.

   Supply these four sections per chunk to the chunk-writer subagent:
   - **Chunk Details** -- number, title, slug, one-line description, dependency (prior chunk file name, or "None"), and summary (2-4 sentences from your decomposition)
   - **High-Level Steps** -- numbered list from the decomposition
   - **Codebase Context** -- file paths, function names, types, and patterns discovered during exploration
   - **Build and Test** -- build and test commands

   Output file path: `./tmp/<plan-name>/chunk-NN-<slug>.md`

   Run chunk writer subagents in parallel when chunks have no dependency on each other's files. Run sequentially only when a chunk's writer prompt references content from an earlier chunk file.

   Decision point: If all chunks receive their full context from the parent prompt (the typical case), launch all subagents in parallel. If a chunk's writer prompt says "read chunk-NN for context," that chunk must wait for chunk-NN's subagent to finish.

   After all subagents complete, spawn a Task subagent (type: Explore, model: haiku) to read all chunk files and verify each matches the chunk-writer Output Format (correct sections, checkboxes, TDD structure where appropriate). The subagent MUST return a pass/fail summary per chunk. Split any chunk exceeding 25 checkboxes. If a chunk file fails validation, provide feedback and re-run the subagent.

7. **Write master plan**:
   Create `./tmp/<plan-name>/plan.md` using the Master Plan Template from [plan-template.md](references/plan-template.md).

8. **Validate**:
   - MUST verify all chunk file paths in plan.md resolve to actual files
   - MUST verify dependency links between chunks are correct and acyclic
   - MUST verify chunk numbering is sequential with no gaps
   - MUST verify every chunk has at least one checkbox in both Implementation Steps and Verification
   - MUST scan all files for non-ASCII characters and replace with ASCII equivalents

9. **Deliver via plan mode**:
   Enter plan mode via `EnterPlanMode` if not already in it. Write the orchestrator prompt to the plan file specified by the plan mode system message, using the Orchestrator Prompt Template from [plan-template.md](references/plan-template.md) with all `<...>` placeholders filled in. MUST scan the orchestrator prompt content for non-ASCII characters and replace with ASCII equivalents before writing.

   Call `ExitPlanMode` to present the plan for user approval. MUST also list all created chunk files with their paths.
