/**
 * Compose Skill Behavioral Specification
 *
 * Verifies delegation correctness, review coverage, routing
 * completeness, and structural invariants for the compose skill.
 *
 * Run:
 *   alloy exec -f -o /tmp/alloy-output \
 *     configs/claude/skills/compose/specs/compose.als
 */
module compose


-- ═══ Domain ══════════════════════════════════════════════════

/** Artifact types the skill operates on */
abstract sig ArtifactType {}
one sig SkillArt, RulesArt, PromptArt, HandoffArt, PlanArt extends ArtifactType {}

/** Agents that write artifacts */
abstract sig WriterAgent {}
one sig SkillWriter, RulesWriter, CodeWriter extends WriterAgent {}

/** Agents that review artifacts */
abstract sig ReviewerAgent {}
one sig SkillReviewer, RulesReviewer extends ReviewerAgent {}

/**
 * References provide domain knowledge (declarative or procedural)
 * consumed by operations. They are structurally separate from Operation:
 * no StepBindings, no agent routing, no artifact lifecycle.
 */
sig Reference {
    consumedBy: some Operation
}

/** Review perspectives (multi-perspective review loop) */
abstract sig Perspective {}
one sig Sonnet, Opus, Haiku extends Perspective {}


-- ═══ Operations ══════════════════════════════════════════════

/**
 * Each operation declares what it produces, who it delegates to,
 * and whether it mutates persistent artifacts.
 *
 * - produces:      artifact types this operation outputs
 * - writesThrough: which writer agent handles file writes
 * - reviewsWith:   which reviewer agent evaluates quality
 * - perspectives:  which review perspectives are used (empty = no review)
 * - mutates:       artifact types modified on disk (empty = read-only/ephemeral)
 */
abstract sig Operation {
    produces:       some ArtifactType,
    writesThrough:  set WriterAgent,
    reviewsWith:    set ReviewerAgent,
    perspectives:   set Perspective,
    mutates:        set ArtifactType
}

one sig CreateSkill extends Operation {} {
    produces      = SkillArt
    writesThrough = SkillWriter
    reviewsWith   = SkillReviewer
    perspectives  = Sonnet + Opus + Haiku
    mutates       = SkillArt
}

one sig UpdateSkill extends Operation {} {
    produces      = SkillArt
    writesThrough = SkillWriter
    reviewsWith   = SkillReviewer
    perspectives  = Sonnet + Opus + Haiku
    mutates       = SkillArt
}

one sig ReviewSkill extends Operation {} {
    produces      = SkillArt
    writesThrough = SkillWriter
    reviewsWith   = SkillReviewer
    perspectives  = Sonnet + Opus + Haiku
    mutates       = SkillArt
}

one sig CreateRules extends Operation {} {
    produces      = RulesArt
    writesThrough = RulesWriter
    reviewsWith   = RulesReviewer
    perspectives  = Sonnet + Opus + Haiku
    mutates       = RulesArt
}

one sig ReviewRules extends Operation {} {
    produces      = RulesArt
    writesThrough = RulesWriter
    reviewsWith   = RulesReviewer
    perspectives  = Sonnet + Opus + Haiku
    mutates       = RulesArt
}

one sig CreatePrompt extends Operation {} {
    produces      = PromptArt
    writesThrough = none
    reviewsWith   = none
    perspectives  = none
    mutates       = none
}

-- ReviewPrompt step 5 offers a conditional rewrite if the user approves fixes.
-- WriteK/DeliverK are intentionally omitted: the rewrite is inline (no writer
-- agent) and produces no persistent artifact (mutates = none).
one sig ReviewPrompt extends Operation {} {
    produces      = PromptArt
    writesThrough = none
    reviewsWith   = none
    perspectives  = none
    mutates       = none
}

one sig CreateHandoff extends Operation {} {
    produces      = HandoffArt
    writesThrough = none
    reviewsWith   = none
    perspectives  = none
    mutates       = none
}

-- PlanTask writes chunk files to ./tmp/ via chunk-writer subagents and a master
-- plan directly.  writesThrough/mutates are none because chunk-writer is outside
-- the WriterAgent domain and plan artifacts are ephemeral (not review-protected
-- skill/rules files).
one sig PlanTask extends Operation {} {
    produces      = PlanArt
    writesThrough = none
    reviewsWith   = none
    perspectives  = none
    mutates       = none
}


-- ═══ Routing ═════════════════════════════════════════════════

/**
 * User intents map to one or more operations.
 * Combined intents (e.g. "create and review") route to multiple ops.
 */
abstract sig Intent {
    routesTo: some Operation
}

one sig IntCreateSkill extends Intent {} {
    routesTo = CreateSkill
}
one sig IntUpdateSkill extends Intent {} {
    routesTo = UpdateSkill
}
one sig IntReviewSkill extends Intent {} {
    routesTo = ReviewSkill
}
one sig IntImproveSkill extends Intent {} {
    routesTo = ReviewSkill
}
one sig IntCreateAndReviewSkill extends Intent {} {
    routesTo = CreateSkill + ReviewSkill
}
one sig IntCreateRules extends Intent {} {
    routesTo = CreateRules
}
one sig IntReviewRules extends Intent {} {
    routesTo = ReviewRules
}
one sig IntImproveRules extends Intent {} {
    routesTo = ReviewRules
}
one sig IntCreatePrompt extends Intent {} {
    routesTo = CreatePrompt
}
one sig IntReviewPrompt extends Intent {} {
    routesTo = ReviewPrompt
}
one sig IntCreateAndReviewPrompt extends Intent {} {
    routesTo = CreatePrompt + ReviewPrompt
}
one sig IntHandoff extends Intent {} {
    routesTo = CreateHandoff
}
one sig IntPlanTask extends Intent {} {
    routesTo = PlanTask
}


-- ═══ State Machines ══════════════════════════════════════════

/** Step kinds that matter for invariant checking */
abstract sig StepKind {}
one sig GatherK, WriteK, ReviewK, ReportK, ConfirmK, DeliverK extends StepKind {}

/**
 * StepBinding ties an operation to its ordered step kinds.
 * position encodes the sequence (lower = earlier).
 *
 * We model only the step kinds relevant to invariant checking,
 * not every substep (e.g. "locate", "read", "assess" fold into GatherK).
 */
sig StepBinding {
    forOp:    one Operation,
    kind:     one StepKind,
    position: one Int
} {
    position >= 0
    position <= 5
}

-- Per-operation step sequences.
-- Each fact uses cardinality + one-quantifiers to fully constrain its steps.

-- Create Skill: gather(0) -> confirm(1) -> write(2) -> review(3) -> report(4)
fact createSkillSteps {
    #{ sb: StepBinding | sb.forOp = CreateSkill } = 5
    one sb: StepBinding | sb.forOp = CreateSkill and sb.kind = GatherK  and sb.position = 0
    one sb: StepBinding | sb.forOp = CreateSkill and sb.kind = ConfirmK and sb.position = 1
    one sb: StepBinding | sb.forOp = CreateSkill and sb.kind = WriteK   and sb.position = 2
    one sb: StepBinding | sb.forOp = CreateSkill and sb.kind = ReviewK  and sb.position = 3
    one sb: StepBinding | sb.forOp = CreateSkill and sb.kind = ReportK  and sb.position = 4
}

-- Update Skill: gather(0) -> confirm(1) -> write(2) -> review(3) -> report(4)
fact updateSkillSteps {
    #{ sb: StepBinding | sb.forOp = UpdateSkill } = 5
    one sb: StepBinding | sb.forOp = UpdateSkill and sb.kind = GatherK  and sb.position = 0
    one sb: StepBinding | sb.forOp = UpdateSkill and sb.kind = ConfirmK and sb.position = 1
    one sb: StepBinding | sb.forOp = UpdateSkill and sb.kind = WriteK   and sb.position = 2
    one sb: StepBinding | sb.forOp = UpdateSkill and sb.kind = ReviewK  and sb.position = 3
    one sb: StepBinding | sb.forOp = UpdateSkill and sb.kind = ReportK  and sb.position = 4
}

-- Review Skill: gather(0) -> review(1) -> write(2) -> report(3)
-- The review-then-write pattern models the fix step in the review loop.
fact reviewSkillSteps {
    #{ sb: StepBinding | sb.forOp = ReviewSkill } = 4
    one sb: StepBinding | sb.forOp = ReviewSkill and sb.kind = GatherK  and sb.position = 0
    one sb: StepBinding | sb.forOp = ReviewSkill and sb.kind = ReviewK  and sb.position = 1
    one sb: StepBinding | sb.forOp = ReviewSkill and sb.kind = WriteK   and sb.position = 2
    one sb: StepBinding | sb.forOp = ReviewSkill and sb.kind = ReportK  and sb.position = 3
}

-- Create Rules: gather(0) -> confirm(1) -> write(2) -> review(3) -> report(4)
fact createRulesSteps {
    #{ sb: StepBinding | sb.forOp = CreateRules } = 5
    one sb: StepBinding | sb.forOp = CreateRules and sb.kind = GatherK  and sb.position = 0
    one sb: StepBinding | sb.forOp = CreateRules and sb.kind = ConfirmK and sb.position = 1
    one sb: StepBinding | sb.forOp = CreateRules and sb.kind = WriteK   and sb.position = 2
    one sb: StepBinding | sb.forOp = CreateRules and sb.kind = ReviewK  and sb.position = 3
    one sb: StepBinding | sb.forOp = CreateRules and sb.kind = ReportK  and sb.position = 4
}

-- Review Rules: gather(0) -> review(1) -> write(2) -> report(3)
fact reviewRulesSteps {
    #{ sb: StepBinding | sb.forOp = ReviewRules } = 4
    one sb: StepBinding | sb.forOp = ReviewRules and sb.kind = GatherK  and sb.position = 0
    one sb: StepBinding | sb.forOp = ReviewRules and sb.kind = ReviewK  and sb.position = 1
    one sb: StepBinding | sb.forOp = ReviewRules and sb.kind = WriteK   and sb.position = 2
    one sb: StepBinding | sb.forOp = ReviewRules and sb.kind = ReportK  and sb.position = 3
}

-- Create Prompt: gather(0) -> confirm(1) -> write(2) -> deliver(3)
fact createPromptSteps {
    #{ sb: StepBinding | sb.forOp = CreatePrompt } = 4
    one sb: StepBinding | sb.forOp = CreatePrompt and sb.kind = GatherK  and sb.position = 0
    one sb: StepBinding | sb.forOp = CreatePrompt and sb.kind = ConfirmK and sb.position = 1
    one sb: StepBinding | sb.forOp = CreatePrompt and sb.kind = WriteK   and sb.position = 2
    one sb: StepBinding | sb.forOp = CreatePrompt and sb.kind = DeliverK and sb.position = 3
}

-- Review Prompt: gather(0) -> review(1) -> report(2)
fact reviewPromptSteps {
    #{ sb: StepBinding | sb.forOp = ReviewPrompt } = 3
    one sb: StepBinding | sb.forOp = ReviewPrompt and sb.kind = GatherK  and sb.position = 0
    one sb: StepBinding | sb.forOp = ReviewPrompt and sb.kind = ReviewK  and sb.position = 1
    one sb: StepBinding | sb.forOp = ReviewPrompt and sb.kind = ReportK  and sb.position = 2
}

-- Create Handoff: gather(0) -> confirm(1) -> write(2) -> deliver(3)
fact createHandoffSteps {
    #{ sb: StepBinding | sb.forOp = CreateHandoff } = 4
    one sb: StepBinding | sb.forOp = CreateHandoff and sb.kind = GatherK  and sb.position = 0
    one sb: StepBinding | sb.forOp = CreateHandoff and sb.kind = ConfirmK and sb.position = 1
    one sb: StepBinding | sb.forOp = CreateHandoff and sb.kind = WriteK   and sb.position = 2
    one sb: StepBinding | sb.forOp = CreateHandoff and sb.kind = DeliverK and sb.position = 3
}

-- Plan Task: gather(0) -> confirm(1) -> write(2) -> review(3) -> deliver(4)
fact planTaskSteps {
    #{ sb: StepBinding | sb.forOp = PlanTask } = 5
    one sb: StepBinding | sb.forOp = PlanTask and sb.kind = GatherK  and sb.position = 0
    one sb: StepBinding | sb.forOp = PlanTask and sb.kind = ConfirmK and sb.position = 1
    one sb: StepBinding | sb.forOp = PlanTask and sb.kind = WriteK   and sb.position = 2
    one sb: StepBinding | sb.forOp = PlanTask and sb.kind = ReviewK  and sb.position = 3
    one sb: StepBinding | sb.forOp = PlanTask and sb.kind = DeliverK and sb.position = 4
}


-- ═══ Invariants (Assertions) ═════════════════════════════════

-- INV-D1: Skill artifacts written only through SkillWriter
assert skillWritesDelegatedCorrectly {
    all op: Operation |
        SkillArt in op.mutates implies
            op.writesThrough = SkillWriter
}

-- INV-D2: Rules artifacts written only through RulesWriter
assert rulesWritesDelegatedCorrectly {
    all op: Operation |
        RulesArt in op.mutates implies
            op.writesThrough = RulesWriter
}

-- INV-D3: CodeWriter never used by any operation
assert codeWriterNeverUsed {
    no op: Operation | CodeWriter in op.writesThrough
}

-- INV-D4: Skill review uses SkillReviewer, Rules review uses RulesReviewer
assert reviewerMatchesArtifact {
    all op: Operation | (
        (SkillArt in op.mutates and some op.reviewsWith)
            implies op.reviewsWith = SkillReviewer
    ) and (
        (RulesArt in op.mutates and some op.reviewsWith)
            implies op.reviewsWith = RulesReviewer
    )
}

-- INV-D5: Non-mutating operations must not use writer or reviewer agents
assert nonMutatingOpsAreClean {
    all op: Operation |
        no op.mutates implies
            (no op.writesThrough and no op.reviewsWith)
}

-- INV-R1: Operations that mutate skill or rules MUST include review
assert mutatingOpsHaveReview {
    all op: Operation |
        (SkillArt in op.mutates or RulesArt in op.mutates) implies
            some op.perspectives
}

-- INV-R5: When review is present, all three perspectives MUST be used
assert reviewUsesAllPerspectives {
    all op: Operation |
        some op.perspectives implies
            op.perspectives = Sonnet + Opus + Haiku
}

-- INV-ROUTE-1: Every operation is reachable from at least one intent
assert allOpsReachable {
    all op: Operation | some i: Intent | op in i.routesTo
}

-- INV-ROUTE-2: Every artifact type is produced by at least one operation
assert allArtifactsCovered {
    all a: ArtifactType | some op: Operation | a in op.produces
}

-- INV-SM-1: Every mutating operation has a write step
assert mutatingOpsHaveWriteStep {
    all op: Operation |
        some op.mutates implies
            some sb: StepBinding | sb.forOp = op and sb.kind = WriteK
}

-- INV-SM-2: In create/update operations, write comes before review.
-- Review operations (ReviewSkill, ReviewRules) are exempt: they use a
-- review-first-then-fix pattern where review precedes the write (fix) step.
assert writeBeforeReviewInCreateOps {
    all op: Operation |
        (some op.mutates and op not in ReviewSkill + ReviewRules) implies
            all sw, sr: StepBinding |
                (sw.forOp = op and sw.kind = WriteK and
                 sr.forOp = op and sr.kind = ReviewK) implies
                    sw.position < sr.position
}

-- INV-SM-3: In review operations, review comes before write (fix step)
assert reviewBeforeWriteInReviewOps {
    all op: ReviewSkill + ReviewRules |
        all sr, sw: StepBinding |
            (sr.forOp = op and sr.kind = ReviewK and
             sw.forOp = op and sw.kind = WriteK) implies
                sr.position < sw.position
}

-- INV-SM-4: Every operation starts with gather (position 0)
assert allOpsStartWithGather {
    all op: Operation |
        some sb: StepBinding | sb.forOp = op and sb.kind = GatherK and sb.position = 0
}

-- INV-SM-5: Confirm step (user gate) comes before write step
assert confirmBeforeWrite {
    all op: Operation |
        (some sc: StepBinding | sc.forOp = op and sc.kind = ConfirmK) implies
            all sc, sw: StepBinding |
                (sc.forOp = op and sc.kind = ConfirmK and
                 sw.forOp = op and sw.kind = WriteK) implies
                    sc.position < sw.position
}

-- INV-REF-1: References are leaves — they consume operations, not other references.
-- The type system enforces this (consumedBy: some Operation), but we assert it
-- explicitly to document the architectural intent: references provide domain
-- knowledge (declarative or procedural) without workflow phases or agent routing.
-- StepBinding.forOp is typed to Operation, so references structurally cannot have
-- step sequences, agent delegation, or artifact lifecycles.
assert referencesAreLeaves {
    all r: Reference | some r.consumedBy
}


-- ═══ Verification ════════════════════════════════════════════

-- Delegation
check skillWritesDelegatedCorrectly       for 5 but 39 StepBinding, 4 Int
check rulesWritesDelegatedCorrectly       for 5 but 39 StepBinding, 4 Int
check codeWriterNeverUsed                 for 5 but 39 StepBinding, 4 Int
check reviewerMatchesArtifact             for 5 but 39 StepBinding, 4 Int

check nonMutatingOpsAreClean              for 5 but 39 StepBinding, 4 Int

-- Review coverage
check mutatingOpsHaveReview               for 5 but 39 StepBinding, 4 Int
check reviewUsesAllPerspectives           for 5 but 39 StepBinding, 4 Int

-- Routing
check allOpsReachable                     for 5 but 39 StepBinding, 4 Int
check allArtifactsCovered                 for 5 but 39 StepBinding, 4 Int

-- State machines
check mutatingOpsHaveWriteStep            for 5 but 39 StepBinding, 4 Int
check writeBeforeReviewInCreateOps        for 5 but 39 StepBinding, 4 Int
check reviewBeforeWriteInReviewOps        for 5 but 39 StepBinding, 4 Int
check allOpsStartWithGather               for 5 but 39 StepBinding, 4 Int
check confirmBeforeWrite                  for 5 but 39 StepBinding, 4 Int

-- References
check referencesAreLeaves                 for 5 but 39 StepBinding, 4 Int


-- ═══ Examples ════════════════════════════════════════════════

-- Show a valid instance of the full model
run showModel {} for 5 but 39 StepBinding, 4 Int

-- Show all operations that mutate artifacts
run showMutatingOps {
    some op: Operation | some op.mutates
} for 5 but 39 StepBinding, 4 Int
