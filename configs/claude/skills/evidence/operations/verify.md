# Verify

Gather supporting, contradicting, and inconclusive evidence for one or more claims and write a structured assessment to a markdown file in `tmp/`.

Input arrives pre-resolved — no $ARGUMENTS parsing needed. Claims were resolved by SKILL.md before this operation was dispatched.

## Instructions

1. **Plan investigation**: For each claim, identify which sources are relevant — codebase (grep, file reads), git history, runtime commands, or external documentation. SHOULD skip source types that clearly don't apply to a given claim. MUST list planned searches for both supporting and contradicting evidence before executing. MUST cap at 10 claims per invocation; if more were provided, verify the 10 most specific and note the remainder.

2. **Gather evidence**: Execute the plan for each claim. MUST search for both supporting and contradicting evidence — do not stop when supporting evidence is found. Cap findings at 20 items per claim; if more are found, keep the 20 most relevant and note how many were omitted. For each finding, record:
   - What was found (the fact or observation)
   - Source — `file:line`, command with its output, or URL
   - Whether it supports, contradicts, or is inconclusive

3. **Classify verdict**: For each claim, MUST assign one of:
   - **Supported** — evidence clearly backs the claim with no significant contradictions
   - **Partially Supported** — evidence leans toward the claim but has meaningful exceptions or gaps
   - **Unsupported** — evidence contradicts the claim or fails to support it
   - **Inconclusive** — insufficient evidence to determine either way

4. **Derive slug**: Derive `<slug>` as a short summary of the subject, max 40 chars. If `tmp/evidence-<slug>.md` already exists, append a numeric suffix (e.g., `-2`, `-3`) until the path is free.

5. **Write evidence file**: Write to `tmp/evidence-<slug>.md` relative to the working directory (never `/tmp/`).

   Format:
   ```
   # Evidence Review

   ## Claim: "<claim text>"

   ### Supporting Evidence
   - **<finding>**: <description>
     Source: <file:line | command | URL>

   ### Contradicting Evidence
   - **<finding>**: <description>
     Source: <file:line | command | URL>

   ### Verdict: <Supported | Partially Supported | Unsupported | Inconclusive>
   <1-2 sentence assessment>

   ## Claim: "<next claim>"
   ...

   ## Summary
   <Overall assessment of all claims verified>
   ```

   If no supporting evidence is found for a claim, write "No supporting evidence found" under that heading. Apply the same to contradicting evidence.

6. **Verify write**: MUST confirm `tmp/evidence-<slug>.md` exists and is non-empty. If the write fails, create the `tmp/` directory and retry. If still failing, print findings directly to the conversation instead.

7. **Report**: Tell the user the file path, each claim's verdict, and a 1-2 sentence overall summary.
