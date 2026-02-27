# Investigate

Research a question or topic using available tools and write structured findings to a markdown file in `tmp/`.

Input arrives pre-resolved — no $ARGUMENTS parsing needed. The question or topic was resolved by SKILL.md before this operation was dispatched.

## Instructions

1. **Plan approach**: Identify which sources are relevant — codebase, runtime, or external references. SHOULD list planned investigation steps before executing. SHOULD skip source types that clearly don't apply.

2. **Collect evidence**: Execute the plan. SHOULD seek evidence on multiple sides of the question — do not stop at the first pattern that emerges. SHOULD cap findings at 20 items max; if more are found, keep the 20 most relevant and note how many were omitted. For each finding, record:
   - What was found (the fact or observation)
   - Source — `file:line`, command with its output, or URL
   - How it relates to the question

3. **Derive slug**: Derive `<slug>` as a short summary of the topic, max 40 chars. If `tmp/evidence-<slug>.md` already exists, append a numeric suffix (e.g., `-2`, `-3`) until the path is free.

4. **Write evidence file**: Write to `tmp/evidence-<slug>.md` relative to the working directory (never `/tmp/`).

   Format:
   ```
   # Evidence: <question>

   ## Findings

   - **<finding title>**: <description>
     Source: <file:line | command | URL>

   - **<finding title>**: <description>
     Source: <file:line | command | URL>

   ## Summary

   <2-3 sentence synthesis of what the evidence shows>
   ```

5. **Verify write**: MUST confirm `tmp/evidence-<slug>.md` exists and is non-empty. If the write fails, create the `tmp/` directory and retry. If still failing, print findings directly to the conversation instead.

6. **Report**: Tell the user the file path and summarize the key findings in 2-3 sentences.
