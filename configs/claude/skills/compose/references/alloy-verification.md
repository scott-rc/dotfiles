# Alloy Verification

Procedure for verifying a skill against its Alloy behavioral specification. Skills MAY include Alloy specs in a `specs/` directory; when present, verification is mandatory.

## Formal Check

Run the Alloy analyzer for each `.als` file in the skill's `specs/` directory:

1. Run `alloy exec -f -o /tmp/alloy-output <spec-path>`
2. UNSAT on all checks = the formal model is internally consistent
3. SAT on any check = counterexample found, a behavioral invariant is violated. Read the output, identify which assertion failed, and fix the skill files or spec before proceeding. Re-run until all checks return UNSAT.

## Structural Conformance

The formal check verifies the Alloy model's internal consistency, but it does not verify that the markdown files match the model. After the formal check passes, verify correspondence:

- **State machines**: Each operation file's numbered steps MUST map to the StepBinding facts in the spec. Verify that gather, confirm, write, review, report, and deliver steps appear in the order the spec declares.
- **Delegation**: Each operation's agent delegation (skill-writer, rules-writer, skill-reviewer, rules-reviewer) MUST match the spec's `writesThrough` and `reviewsWith` fields.
- **Routing**: SKILL.md's Combined Operations section MUST match the spec's Intent routing (each intent maps to the correct operation(s)).
- **Perspectives**: Operations that include review MUST use all three perspectives (Sonnet, Opus, Haiku) when the spec declares `perspectives = Sonnet + Opus + Haiku`.

If any structural conformance check fails, fix the skill files (or update the spec if the intent changed) and re-verify.
