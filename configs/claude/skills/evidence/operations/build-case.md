# Build Case

Collect supporting and contradicting evidence for a claim and write a balanced assessment to a markdown file in `tmp/`.

Input arrives pre-resolved — no $ARGUMENTS parsing needed. The claim was resolved by SKILL.md before this operation was dispatched.

## Instructions

1. **Plan investigation**: Identify what evidence would support the claim AND what evidence would contradict it. SHOULD list searches for both sides before executing.

2. **Collect supporting evidence**: Search for facts, code paths, outputs, or documentation that support the claim. SHOULD cap findings at 20 items max; if more are found, keep the 20 most relevant and note how many were omitted. Record each finding with its source (`file:line`, command output, or URL).

3. **Collect contradicting evidence**: MUST apply equal rigor to contradicting evidence — do not shortcut the opposing side. Search for facts, code paths, outputs, or documentation that weaken or contradict the claim. Apply the same 20-item cap. If no contradicting evidence is found, record "No contradicting evidence found" in that section and note this in the assessment.

4. **Derive slug**: Derive `<slug>` as a short summary of the claim, max 40 chars. If `tmp/case-<slug>.md` already exists, append a numeric suffix (e.g., `-2`, `-3`) until the path is free.

5. **Write evidence file**: Write to `tmp/case-<slug>.md` relative to the working directory (never `/tmp/`).

   Format:
   ```
   # Case: <claim>

   ## Supporting Evidence

   - **<finding>**: <description>
     Source: <source>

   ## Contradicting Evidence

   - **<finding>**: <description>
     Source: <source>

   ## Assessment

   <2-3 sentence balanced assessment of the claim based on evidence found>
   Confidence: <high | medium | low>
   ```

6. **Verify write**: MUST confirm `tmp/case-<slug>.md` exists and is non-empty. If the write fails, create the `tmp/` directory and retry. If still failing, print findings directly to the conversation instead.

7. **Report**: Tell the user the file path, the assessment sentence(s), and the confidence level.
