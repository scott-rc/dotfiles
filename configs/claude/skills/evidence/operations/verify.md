# Verify

Gather supporting, contradicting, and inconclusive evidence for one or more claims and write a structured assessment to a markdown file in `tmp/`.

Input arrives pre-resolved — no $ARGUMENTS parsing needed. Claims were resolved by SKILL.md before this operation was dispatched.

## Instructions

1. **Plan investigation**: For each claim, decompose it into its constituent sub-claims before planning any searches. A compound claim like "apps on the hot-sauce connection saturated the curds shard" contains distinct assertions — that "hot-sauce" maps to the curds shard, and that those apps caused saturation — each requiring independent verification. Naming and mapping claims ("X is Y", "X maps to Y", "X means Z") MUST be verified against authoritative sources, not merely confirmed to exist in data. Then identify which sources are relevant — codebase (grep, file reads), git history, runtime commands, MCP tools (monitoring APIs, databases, internal services), prior conversation context (tool results from earlier in the same session), or external documentation. SHOULD skip source types that clearly don't apply to a given claim. MUST list planned searches for both supporting and contradicting evidence before executing. MUST cap at 10 claims per invocation; if more were provided, verify the 10 most specific and note the remainder.

2. **Gather evidence**: Execute the plan for each claim. MUST search for both supporting and contradicting evidence — do not stop when supporting evidence is found. When a claim asserts or implies causation (X caused Y, X led to Y, the root cause is X), MUST actively search for alternative explanations — recent config changes, deployments, scaling events, infrastructure changes, or other environmental shifts in the same time window. A causal claim is not supported merely because the proposed cause and effect both occurred; competing causes MUST be investigated and ruled out before the claim can be marked Supported. When the conversation already contains tool results that serve as evidence (query results, API responses, command output from earlier in the session), cite those directly rather than re-running the same queries. Only re-query when: (a) the original data was incomplete, (b) a fresh query would verify from a different angle, or (c) the original result is no longer in context. Cap findings at 20 items per claim; if more are found, keep the 20 most relevant and note how many were omitted. For each finding, record:
   - What was found (the fact or observation)
   - Source — `file:line`, command with its output, MCP tool call with key parameters, or URL
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
     Source: <file:line | command | MCP tool(params) | URL>

   ### Contradicting Evidence
   - **<finding>**: <description>
     Source: <file:line | command | MCP tool(params) | URL>

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
