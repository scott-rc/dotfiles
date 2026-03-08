/**
 * Git Skill Behavioral Specification
 *
 * Verifies delegation correctness, routing completeness, step-sequence
 * invariants, composition safety, and reference coverage for the git skill.
 *
 * Run:
 *   alloy exec -f -o /tmp/alloy-output \
 *     configs/claude/skills/git/specs/git.als
 */
module git


-- ═══ Domain ══════════════════════════════════════════════════

/** Artifact types the skill operates on */
abstract sig ArtifactType {}
one sig CommitArt, PRArt, GitHubTextArt extends ArtifactType {}

/** Agents that operations delegate work to */
abstract sig DelegateAgent {}
one sig Committer, PRWriter, GitHubWriter, CITriager, FixSubagent, ExploreSubagent extends DelegateAgent {}

/**
 * References provide domain knowledge (declarative or procedural)
 * consumed by operations. They are structurally separate from Operation:
 * no step bindings, no agent routing, no artifact lifecycle.
 * Scripts are modeled as Reference instances — same sig type.
 */
abstract sig Reference {
    consumedBy: some Operation
}


-- ═══ Operations ══════════════════════════════════════════════

/**
 * Each operation declares what it produces, who it delegates to,
 * whether it mutates persistent artifacts, and which other operations
 * it can invoke as sub-operations.
 *
 * - produces:     artifact types this operation outputs (empty for read-only ops)
 * - delegatesTo:  which delegate agents handle subwork
 * - mutates:      artifact types modified on disk or remote (empty = read-only)
 * - invokes:      other operations this operation can trigger inline
 */
abstract sig Operation {
    produces:    set ArtifactType,
    delegatesTo: set DelegateAgent,
    mutates:     set ArtifactType,
    invokes:     set Operation
}

one sig Commit extends Operation {} {
    produces    = CommitArt + PRArt
    delegatesTo = Committer + PRWriter
    mutates     = CommitArt + PRArt
    invokes     = none
}

one sig Squash extends Operation {} {
    produces    = CommitArt
    delegatesTo = Committer
    mutates     = CommitArt
    invokes     = Push
}

one sig Rebase extends Operation {} {
    produces    = CommitArt
    delegatesTo = none
    mutates     = CommitArt
    invokes     = none
}

one sig Push extends Operation {} {
    produces    = CommitArt + PRArt
    delegatesTo = Committer + PRWriter
    mutates     = CommitArt + PRArt
    invokes     = Commit
}

-- Correct propagates a user correction to all affected artifacts.
-- Commit message amend is inlined (orchestrator already has the text).
-- Delegates to PRWriter (update description).
-- Also directly edits branch context and changeset files (not domain artifacts).
one sig Correct extends Operation {} {
    produces    = CommitArt + PRArt
    delegatesTo = PRWriter
    mutates     = CommitArt + PRArt
    invokes     = none
}

-- Fix unifies CI failure repair (FixCI), review feedback (FixReview), and
-- PR comment replies (Reply). CITriager is GitHub-Actions-specific; for
-- Buildkite the triager is bypassed and FixSubagent handles everything.
one sig Fix extends Operation {} {
    produces    = CommitArt + GitHubTextArt
    delegatesTo = CITriager + FixSubagent + ExploreSubagent + GitHubWriter + Committer
    mutates     = CommitArt
    invokes     = none
}


-- ═══ References ═════════════════════════════════════════════

-- Named reference instances. Each constrains its consumedBy set.

one sig GitPatterns extends Reference {} {
    -- Consumed by all operations.
    consumedBy = Commit + Squash + Rebase + Push + Correct + Fix
}

one sig GitHubText extends Reference {} {
    consumedBy = Commit + Push + Correct + Fix
}

-- PRWriterRules provides delegation context and commit-forwarding rules for
-- callers that spawn the pr-writer agent. Commit messages are forwarded as
-- supplementary hints only -- the diff is the source of truth. Commit messages
-- describe intermediate states (fixups, reverts, mid-PR bugs) that must not
-- appear in the final PR description.
one sig PRWriterRules extends Reference {} {
    consumedBy = Commit + Push + Correct
}

one sig BulkThreads extends Reference {} {
    consumedBy = Fix
}

one sig BuildkiteHandling extends Reference {} {
    consumedBy = Fix
}

one sig CommitMessageFormat extends Reference {} {
    consumedBy = Commit + Squash + Correct + Fix
}

-- Scripts modeled as Reference instances

one sig SafeText extends Reference {} {
    -- Indirect: ops delegate to Committer/PRWriter agents, which call the script.
    consumedBy = Commit + Squash + Push + Correct + Fix
}

one sig GetPRComments extends Reference {} {
    consumedBy = Fix
}

one sig GetFailedRuns extends Reference {} {
    consumedBy = Fix
}


-- ═══ Routing ═════════════════════════════════════════════════

/**
 * User intents map to one or more operations.
 * Combined intents (e.g. "commit and push") route to multiple ops.
 */
abstract sig Intent {
    routesTo: some Operation
}

one sig IntCommit extends Intent {} {
    routesTo = Commit
}
one sig IntCommitAndPush extends Intent {} {
    routesTo = Commit + Push
}
one sig IntSquash extends Intent {} {
    routesTo = Squash
}
one sig IntSquashAndPush extends Intent {} {
    routesTo = Squash + Push
}
one sig IntPush extends Intent {} {
    routesTo = Push
}
one sig IntRebase extends Intent {} {
    routesTo = Rebase
}
one sig IntCorrect extends Intent {} {
    routesTo = Correct
}
one sig IntFix extends Intent {} {
    routesTo = Fix
}
one sig IntFixAndPush extends Intent {} {
    routesTo = Fix + Push
}


-- ═══ Step Sequences (Predicate Encoding) ════════════════════

/**
 * Step kinds that matter for invariant checking.
 *
 * Step sequences are encoded as ground-truth predicates rather than
 * existentially-quantified StepBinding atoms. This eliminates the
 * combinatorial explosion that caused the solver to hang on assertions
 * involving nested quantifiers over 70+ binding atoms.
 */
abstract sig StepKind {}
one sig GatherK, ConfirmK, DelegateK, WriteK, PublishK, ReportK, LoopK, VerifyK extends StepKind {}

/**
 * hasStep[op, k, p] — operation op has a step of kind k at position p.
 * maxPos[op, p] — position p is the highest step position for operation op.
 *
 * These predicates encode the same information that was previously modeled
 * as StepBinding sig atoms with per-operation facts. The assertions now
 * reference these predicates directly, making all checks instantaneous.
 */
pred hasStep[op: Operation, k: StepKind, p: Int] {
    -- Commit: gather(0) -> write(1) -> delegate(2) -> confirm(3) -> delegate(4) -> publish(5) -> delegate(6) -> report(7)
    -- write(1) = inline commit (simple path); delegate(2) = committer (complex path);
    -- confirm(3) = amend or update description needed?;
    -- delegate(4) = committer amend (message rewrite) or pr-writer description update;
    -- publish(5) = force-push if amend path taken; delegate(6) = pr-writer update.
    (op = Commit and k = GatherK   and p = 0) or
    (op = Commit and k = WriteK    and p = 1) or
    (op = Commit and k = DelegateK and p = 2) or
    (op = Commit and k = ConfirmK  and p = 3) or
    (op = Commit and k = DelegateK and p = 4) or
    (op = Commit and k = PublishK  and p = 5) or
    (op = Commit and k = DelegateK and p = 6) or
    (op = Commit and k = ReportK   and p = 7) or

    -- Squash: gather(0) -> delegate(1) -> write(2) -> verify(3) -> confirm(4) -> delegate(5) -> report(6)
    -- delegate(1) = optional commit of uncommitted changes; write(2) = rebase;
    -- verify(3) = scope check; confirm(4) = squash?;
    -- delegate(5) = committer squash.
    (op = Squash and k = GatherK   and p = 0) or
    (op = Squash and k = DelegateK and p = 1) or
    (op = Squash and k = WriteK    and p = 2) or
    (op = Squash and k = VerifyK   and p = 3) or
    (op = Squash and k = ConfirmK  and p = 4) or
    (op = Squash and k = DelegateK and p = 5) or
    (op = Squash and k = ReportK   and p = 6) or

    -- Rebase: gather(0) -> write(1) -> verify(2) -> report(3)
    -- No delegation -- rebase is inline. verify(2) = scope check.
    (op = Rebase and k = GatherK  and p = 0) or
    (op = Rebase and k = WriteK   and p = 1) or
    (op = Rebase and k = VerifyK  and p = 2) or
    (op = Rebase and k = ReportK  and p = 3) or

    -- Push: gather(0) -> publish(1) -> verify(2) -> delegate(3) -> confirm(4) -> delegate(5) -> report(6)
    -- publish(1) = git push to remote; verify(2) = check PR state;
    -- delegate(3) = pr-writer creates/updates PR; confirm(4) = update description?;
    -- delegate(5) = pr-writer rewrites description.
    (op = Push and k = GatherK   and p = 0) or
    (op = Push and k = PublishK  and p = 1) or
    (op = Push and k = VerifyK   and p = 2) or
    (op = Push and k = DelegateK and p = 3) or
    (op = Push and k = ConfirmK  and p = 4) or
    (op = Push and k = DelegateK and p = 5) or
    (op = Push and k = ReportK   and p = 6) or

    -- Correct: gather(0) -> write(1) -> write(2) -> delegate(3) -> confirm(4) -> publish(5) -> report(6)
    -- gather(0) = understand correction, detect base, scan all artifacts;
    -- write(1) = fix branch context and changeset files directly;
    -- write(2) = inline amend of commit message;
    -- delegate(3) = pr-writer updates description;
    -- confirm(4) = force push offer; publish(5) = force push if accepted.
    (op = Correct and k = GatherK   and p = 0) or
    (op = Correct and k = WriteK    and p = 1) or
    (op = Correct and k = WriteK    and p = 2) or
    (op = Correct and k = DelegateK and p = 3) or
    (op = Correct and k = ConfirmK  and p = 4) or
    (op = Correct and k = PublishK  and p = 5) or
    (op = Correct and k = ReportK   and p = 6) or

    -- Fix: gather(0) -> report(1) -> confirm(2) -> delegate(3) -> delegate(4) -> verify(5) -> write(6) -> publish(7) -> report(8)
    -- gather(0) = detect CI failures and review threads in parallel;
    -- report(1) = summarize what was found; confirm(2) = classify threads, approve plan;
    -- delegate(3) = CITriager (GitHub Actions) or ExploreSubagent (bulk threads);
    -- delegate(4) = FixSubagent applies code fixes; verify(5) = run linter/tests;
    -- write(6) = inline commit of fixes; publish(7) = GitHubWriter posts replies.
    (op = Fix and k = GatherK   and p = 0) or
    (op = Fix and k = ReportK   and p = 1) or
    (op = Fix and k = ConfirmK  and p = 2) or
    (op = Fix and k = DelegateK and p = 3) or
    (op = Fix and k = DelegateK and p = 4) or
    (op = Fix and k = VerifyK   and p = 5) or
    (op = Fix and k = WriteK    and p = 6) or
    (op = Fix and k = PublishK  and p = 7) or
    (op = Fix and k = ReportK   and p = 8)
}

pred maxPos[op: Operation, p: Int] {
    (op = Commit  and p = 7) or
    (op = Squash  and p = 6) or
    (op = Rebase  and p = 3) or
    (op = Push    and p = 6) or
    (op = Fix     and p = 8) or
    (op = Correct and p = 6)
}


-- ═══ Invariants (Assertions) ═════════════════════════════════

-- INV-D1: Committer agent only used by ops that produce/mutate CommitArt
assert committerDelegatedCorrectly {
    all op: Operation |
        Committer in op.delegatesTo implies
            CommitArt in (op.produces + op.mutates)
}

-- INV-D2: PRWriter agent only used by ops that produce/mutate PRArt
assert prWriterDelegatedCorrectly {
    all op: Operation |
        PRWriter in op.delegatesTo implies
            PRArt in (op.produces + op.mutates)
}

-- INV-D3: GitHubWriter agent only used by ops that produce GitHubTextArt
assert githubWriterDelegatedCorrectly {
    all op: Operation |
        GitHubWriter in op.delegatesTo implies
            GitHubTextArt in op.produces
}

-- INV-D4: ExploreSubagent only used by ops that consume BulkThreads reference
assert exploreSubagentMatchesBulkThreads {
    all op: Operation |
        ExploreSubagent in op.delegatesTo implies
            op in BulkThreads.consumedBy
}

-- INV-D5: Every mutated artifact type is also produced
assert mutatesSubsetOfProduces {
    all op: Operation | op.mutates in op.produces
}

-- INV-D6: CITriager delegation always implies FixSubagent delegation
-- (triager never acts alone -- it only triages, then hands off to fix)
assert ciTriagerImpliesFixSubagent {
    all op: Operation |
        CITriager in op.delegatesTo implies
            FixSubagent in op.delegatesTo
}

-- INV-P1: PublishK always preceded by GatherK in the same operation
assert publishPrecededByGather {
    all op: Operation, pp: Int |
        hasStep[op, PublishK, pp] implies
            some gp: Int | hasStep[op, GatherK, gp] and gp < pp
}

-- INV-P2: ConfirmK precedes PublishK when both present in the same operation.
-- Push is exempt: its PublishK (git push) and ConfirmK (update description?)
-- guard independent concerns — the push is the primary action, not a
-- side-effect that needs user gating.
assert confirmPrecedesPublish {
    all op: Operation, cp: Int, pp: Int |
        (op != Push and hasStep[op, ConfirmK, cp] and hasStep[op, PublishK, pp])
        implies cp < pp
}

-- INV-SM-1: Every operation starts with GatherK at position 0
assert allOpsStartWithGather {
    all op: Operation | hasStep[op, GatherK, 0]
}

-- INV-SM-2: Every operation ends with ReportK at its highest position
assert allOpsEndWithReport {
    all op: Operation |
        some p: Int | maxPos[op, p] and hasStep[op, ReportK, p]
}

-- INV-SM-3: Every ConfirmK must have at least one WriteK or DelegateK after it
-- (weakened from "all ConfirmK < all WriteK" to support operations
-- where confirm appears between writes, and broadened to include DelegateK
-- for operations like Fix where the action after confirm is delegation,
-- not a write)
assert confirmPrecedesAction {
    all op: Operation, cp: Int |
        hasStep[op, ConfirmK, cp] implies
            (some wp: Int | hasStep[op, WriteK, wp] and cp < wp) or
            (some dp: Int | hasStep[op, DelegateK, dp] and cp < dp) or
            (some pp: Int | hasStep[op, PublishK, pp] and cp < pp)
}

-- INV-SM-4: Operations that delegate must have an action step
assert delegationImpliesActionStep {
    all op: Operation |
        some op.delegatesTo implies
            some p: Int | hasStep[op, DelegateK, p] or hasStep[op, PublishK, p] or hasStep[op, LoopK, p]
}

-- INV-SM-5: Operations that mutate must have an action step
assert mutationImpliesActionStep {
    all op: Operation |
        some op.mutates implies
            some p: Int |
                hasStep[op, WriteK, p] or hasStep[op, DelegateK, p] or
                hasStep[op, PublishK, p] or hasStep[op, LoopK, p]
}

-- INV-SM-6: Domain-empty operations must not delegate
-- Domain-empty ops (no produces, no delegatesTo, no mutates) should have
-- no DelegateK or LoopK steps. WriteK is allowed for local utility writes.
-- ConfirmK is allowed for user interaction. PublishK is allowed for external
-- triggers. VerifyK is allowed for post-action checks.
assert domainEmptyOpsNoDelegation {
    all op: Operation |
        (no op.produces and no op.delegatesTo and no op.mutates) implies
            all k: StepKind, p: Int |
                hasStep[op, k, p] implies k not in (DelegateK + LoopK)
}

-- INV-SM-7: VerifyK must be preceded by an action step
-- (DelegateK, WriteK, PublishK, or LoopK) in the same operation
assert verifyFollowsAction {
    all op: Operation, vp: Int |
        hasStep[op, VerifyK, vp] implies
            some ap: Int |
                (hasStep[op, DelegateK, ap] or hasStep[op, WriteK, ap] or
                 hasStep[op, PublishK, ap] or hasStep[op, LoopK, ap]) and
                ap < vp
}

-- INV-ROUTE-1: Every operation is reachable from at least one intent
assert allOpsReachable {
    all op: Operation | some i: Intent | op in i.routesTo
}

-- INV-ROUTE-2: Every delegate agent is used by at least one operation
assert allAgentsUsed {
    all a: DelegateAgent | some op: Operation | a in op.delegatesTo
}

-- INV-COMP-1: Every invoked operation must be reachable from at least one intent
assert invokedOpsReachable {
    all op1, op2: Operation |
        op2 in op1.invokes implies
            some i: Intent | op2 in i.routesTo
}

-- INV-REF-1: References are leaves — all references have non-empty consumedBy.
-- The type system enforces this (consumedBy: some Operation), but we assert it
-- explicitly to document the architectural intent.
assert referencesAreLeaves {
    all r: Reference | some r.consumedBy
}

-- INV-REF-2: GitPatterns is consumed by all operations.
assert gitPatternsMostOps {
    all op: Operation | op in GitPatterns.consumedBy
}

-- INV-REF-3: GitHubText consumed by all ops that produce GitHubTextArt or PRArt
assert githubTextMatchesProduction {
    all op: Operation |
        (GitHubTextArt in op.produces or PRArt in op.produces) implies
            op in GitHubText.consumedBy
}


-- ═══ Verification ════════════════════════════════════════════

-- Delegation
check committerDelegatedCorrectly        for 5 but 4 Int
check prWriterDelegatedCorrectly         for 5 but 4 Int
check githubWriterDelegatedCorrectly     for 5 but 4 Int
check exploreSubagentMatchesBulkThreads  for 5 but 4 Int
check mutatesSubsetOfProduces            for 5 but 4 Int
check ciTriagerImpliesFixSubagent        for 5 but 4 Int

-- Publish safety
check publishPrecededByGather            for 5 but 4 Int
check confirmPrecedesPublish             for 5 but 4 Int

-- State machines
check allOpsStartWithGather              for 5 but 4 Int
check allOpsEndWithReport                for 5 but 4 Int
check confirmPrecedesAction              for 5 but 4 Int
check delegationImpliesActionStep        for 5 but 4 Int
check mutationImpliesActionStep          for 5 but 4 Int
check domainEmptyOpsNoDelegation         for 5 but 4 Int
check verifyFollowsAction                for 5 but 4 Int

-- Routing
check allOpsReachable                    for 5 but 4 Int
check allAgentsUsed                      for 5 but 4 Int

-- Composition
check invokedOpsReachable                for 5 but 4 Int

-- References
check referencesAreLeaves                for 5 but 4 Int
check gitPatternsMostOps                 for 5 but 4 Int
check githubTextMatchesProduction        for 5 but 4 Int
