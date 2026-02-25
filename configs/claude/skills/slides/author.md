# Author

Write or edit Slidev presentation content by delegating to the `slide-writer` agent.

## Instructions

1. **Locate talk**:
   Discover `talks/` directories in the slides workspace. If multiple talks exist, present them as AskUserQuestion options. Confirm the absolute path to the target talk directory.

2. **Gather requirements**:
   Determine what the user wants written or changed. If the request is vague, batch clarifying questions into a single AskUserQuestion (outline, topic, style, target audience). Do not ask one question at a time.

3. **Determine mode**:
   - `create` -- talk directory has no `slides.md` or only the starter template from the Create operation
   - `edit` -- talk directory has an existing `slides.md` with real content

4. **Delegate to `slide-writer`**:
   Invoke the `slide-writer` agent with:
   - `talk_path` -- absolute path to the talk directory
   - `mode` -- `create` or `edit`
   - `requirements` -- consolidated outline, topic, style notes, or specific edits

5. **Report result**:
   Relay the slide-writer's output: slide count, file path, and build status.
