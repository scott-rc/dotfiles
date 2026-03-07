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
one sig CommitArt, PRArt, GitHubTextArt, WatchStateArt extends ArtifactType {}

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
    produces    = CommitArt
    delegatesTo = Committer
    mutates     = CommitArt
    invokes     = SetBranchContext  -- partial: steps 3-4 only (gather+write, skipping report)
}

one sig Amend extends Operation {} {
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

-- CheckCI is the status-only mode: gather state, report results.
-- No delegation, no mutation, no artifacts produced.
one sig CheckCI extends Operation {} {
    produces    = none
    delegatesTo = none
    mutates     = none
    invokes     = none
}

-- CITriager delegation is GitHub-Actions-specific; for Buildkite,
-- the triager is bypassed and FixSubagent handles everything directly.
one sig FixCI extends Operation {} {
    produces    = CommitArt
    delegatesTo = CITriager + FixSubagent
    mutates     = CommitArt
    invokes     = Push
}

-- Rerun triggers a CI re-run via gh; no local artifacts or delegation.
one sig Rerun extends Operation {} {
    produces    = none
    delegatesTo = none
    mutates     = none
    invokes     = Watch
}

-- Watch is a meta-operation: its loop internally invokes fix and commit
-- patterns. The outer structure is gather -> report -> loop -> report.
-- CITriager delegation is GitHub-Actions-specific; for Buildkite,
-- the triager is bypassed and FixSubagent handles everything directly.
one sig Watch extends Operation {} {
    produces    = WatchStateArt + CommitArt + GitHubTextArt
    delegatesTo = CITriager + FixSubagent + Committer + GitHubWriter
    mutates     = WatchStateArt + CommitArt
    invokes     = none
}

one sig FixReview extends Operation {} {
    produces    = CommitArt
    delegatesTo = FixSubagent + ExploreSubagent
    mutates     = CommitArt
    invokes     = none
}

one sig Reply extends Operation {} {
    produces    = GitHubTextArt
    delegatesTo = GitHubWriter + ExploreSubagent
    mutates     = none
    invokes     = none
}

one sig UpdateDescription extends Operation {} {
    produces    = PRArt
    delegatesTo = PRWriter
    mutates     = PRArt
    invokes     = none
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

-- SetBranchContext reads or creates the branch context file.
-- No delegation -- purely inline. Produces nothing (the context file
-- is consumed by other operations, not an artifact in our domain model).
one sig SetBranchContext extends Operation {} {
    produces    = none
    delegatesTo = none
    mutates     = none
    invokes     = none
}


-- ═══ References ═════════════════════════════════════════════

-- Named reference instances. Each constrains its consumedBy set.

one sig GitPatterns extends Reference {} {
    -- Consumed by most operations. Rerun and CheckCI are intentional
    -- exceptions: Rerun uses only gh CLI directly; CheckCI only uses
    -- gh CLI directly for status gathering.
    -- SetBranchContext is also excluded: it only checks the branch name and
    -- writes a context file -- no base-branch detection, scope verification,
    -- or script patterns are used.
    consumedBy = Commit + Amend + Squash + Rebase + Push
                 + FixCI + Watch
                 + FixReview + Reply + UpdateDescription + Correct
}

one sig GitHubText extends Reference {} {
    consumedBy = Push + Amend + Reply + UpdateDescription + Watch + Correct
}

-- PRWriterRules provides delegation context and commit-forwarding rules for
-- callers that spawn the pr-writer agent. Commit messages are forwarded as
-- supplementary hints only -- the diff is the source of truth. Commit messages
-- describe intermediate states (fixups, reverts, mid-PR bugs) that must not
-- appear in the final PR description.
one sig PRWriterRules extends Reference {} {
    consumedBy = Push + Amend + UpdateDescription + Correct
}

one sig BulkThreads extends Reference {} {
    consumedBy = FixReview + Reply
}

one sig BuildkiteHandling extends Reference {} {
    consumedBy = Watch + FixCI
}

one sig WatchSubops extends Reference {} {
    consumedBy = Watch
}

one sig CommitMessageFormat extends Reference {} {
    consumedBy = Commit + Squash + Correct + FixReview + FixCI
}

-- Scripts modeled as Reference instances

one sig GetPRComments extends Reference {} {
    consumedBy = FixReview + Reply + Watch
}

one sig PollPRStatus extends Reference {} {
    consumedBy = Watch
}

one sig GetFailedRuns extends Reference {} {
    consumedBy = FixCI + Watch
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
one sig IntAmend extends Intent {} {
    routesTo = Amend
}
one sig IntAmendAndPush extends Intent {} {
    routesTo = Amend + Push
}
one sig IntSquash extends Intent {} {
    routesTo = Squash
}
one sig IntSquashAndPush extends Intent {} {
    routesTo = Squash + Push
}
one sig IntSquashAndUpdateDescription extends Intent {} {
    routesTo = Squash + UpdateDescription
}
one sig IntPush extends Intent {} {
    routesTo = Push
}
one sig IntRebase extends Intent {} {
    routesTo = Rebase
}
one sig IntCheckCI extends Intent {} {
    routesTo = CheckCI
}
one sig IntFixCI extends Intent {} {
    routesTo = FixCI
}
one sig IntRerun extends Intent {} {
    routesTo = Rerun
}
one sig IntRerunAndWatch extends Intent {} {
    routesTo = Rerun + Watch
}
one sig IntWatch extends Intent {} {
    routesTo = Watch
}
one sig IntPushAndWatch extends Intent {} {
    routesTo = Push + Watch
}
one sig IntFixReview extends Intent {} {
    routesTo = FixReview
}
one sig IntUpdateDescription extends Intent {} {
    routesTo = UpdateDescription
}
one sig IntReply extends Intent {} {
    routesTo = Reply
}
one sig IntFixReviewAndPush extends Intent {} {
    routesTo = FixReview + Push
}
one sig IntSetBranchContext extends Intent {} {
    routesTo = SetBranchContext
}
one sig IntCorrect extends Intent {} {
    routesTo = Correct
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
    -- Commit: gather(0) -> write(1) -> delegate(2) -> confirm(3) -> delegate(4) -> report(5)
    -- write(1) = inline commit (simple path); delegate(2) = committer (complex path);
    -- confirm(3) = cohesion choice if mixed concerns;
    -- delegate(4) = committer re-invocation with user's file selection.
    (op = Commit and k = GatherK   and p = 0) or
    (op = Commit and k = WriteK    and p = 1) or
    (op = Commit and k = DelegateK and p = 2) or
    (op = Commit and k = ConfirmK  and p = 3) or
    (op = Commit and k = DelegateK and p = 4) or
    (op = Commit and k = ReportK   and p = 5) or

    -- Amend: gather(0) -> write(1) -> verify(2) -> confirm(3) -> publish(4) -> delegate(5) -> report(6)
    -- write(1) = inline no-edit amend; verify(2) = compare file sets;
    -- confirm(3) = message update?; publish(4) = force-push;
    -- delegate(5) = pr-writer update (or committer for message rewrite).
    (op = Amend and k = GatherK   and p = 0) or
    (op = Amend and k = WriteK    and p = 1) or
    (op = Amend and k = VerifyK   and p = 2) or
    (op = Amend and k = ConfirmK  and p = 3) or
    (op = Amend and k = PublishK  and p = 4) or
    (op = Amend and k = DelegateK and p = 5) or
    (op = Amend and k = ReportK   and p = 6) or

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

    -- Push: gather(0) -> publish(1) -> verify(2) -> delegate(3) -> report(4)
    -- publish(1) = git push to remote; verify(2) = check PR state;
    -- delegate(3) = pr-writer creates/updates PR.
    (op = Push and k = GatherK   and p = 0) or
    (op = Push and k = PublishK  and p = 1) or
    (op = Push and k = VerifyK   and p = 2) or
    (op = Push and k = DelegateK and p = 3) or
    (op = Push and k = ReportK   and p = 4) or

    -- CheckCI: gather(0) -> report(1)
    (op = CheckCI and k = GatherK and p = 0) or
    (op = CheckCI and k = ReportK and p = 1) or

    -- FixCI: gather(0) -> delegate(1) -> delegate(2) -> write(3) -> report(4)
    -- delegate(1) = ci-triager triages failures; delegate(2) = fix subagent applies fixes;
    -- write(3) = inline commit of fixes.
    (op = FixCI and k = GatherK   and p = 0) or
    (op = FixCI and k = DelegateK and p = 1) or
    (op = FixCI and k = DelegateK and p = 2) or
    (op = FixCI and k = WriteK    and p = 3) or
    (op = FixCI and k = ReportK   and p = 4) or

    -- Rerun: gather(0) -> publish(1) -> verify(2) -> report(3)
    -- publish(1) = gh run rerun; verify(2) = check new status.
    (op = Rerun and k = GatherK  and p = 0) or
    (op = Rerun and k = PublishK and p = 1) or
    (op = Rerun and k = VerifyK  and p = 2) or
    (op = Rerun and k = ReportK  and p = 3) or

    -- Watch: gather(0) -> report(1) -> loop(2) -> report(3)
    -- ReportK appears at both position 1 (initial status) and position 3 (final summary).
    (op = Watch and k = GatherK and p = 0) or
    (op = Watch and k = ReportK and p = 1) or
    (op = Watch and k = LoopK   and p = 2) or
    (op = Watch and k = ReportK and p = 3) or

    -- FixReview: gather(0) -> report(1) -> confirm(2) -> delegate(3) -> verify(4) -> write(5) -> report(6)
    -- confirm(2) = classify threads; human reviewer threads require user approval.
    -- delegate(3) = fix subagent applies fixes; verify(4) = run linter/tests and re-read code;
    -- write(5) = inline commit of fixes.
    -- ReportK appears at both position 1 (summary) and position 6 (final report).
    (op = FixReview and k = GatherK   and p = 0) or
    (op = FixReview and k = ReportK   and p = 1) or
    (op = FixReview and k = ConfirmK  and p = 2) or
    (op = FixReview and k = DelegateK and p = 3) or
    (op = FixReview and k = VerifyK   and p = 4) or
    (op = FixReview and k = WriteK    and p = 5) or
    (op = FixReview and k = ReportK   and p = 6) or

    -- Reply: gather(0) -> report(1) -> confirm(2) -> publish(3) -> report(4)
    -- ReportK appears at both position 1 (present drafts) and position 4 (summary).
    -- PublishK (not DelegateK) at position 3: GitHub posting is modeled as PublishK
    -- to enforce INV-P2 (confirm-before-publish).
    (op = Reply and k = GatherK  and p = 0) or
    (op = Reply and k = ReportK  and p = 1) or
    (op = Reply and k = ConfirmK and p = 2) or
    (op = Reply and k = PublishK and p = 3) or
    (op = Reply and k = ReportK  and p = 4) or

    -- Update Description: gather(0) -> delegate(1) -> confirm(2) -> publish(3) -> report(4)
    -- delegate(1) = pr-writer rewrites description; confirm(2) = check for unpushed
    -- history rewrite (amend/squash); publish(3) = force-push if user accepts.
    (op = UpdateDescription and k = GatherK   and p = 0) or
    (op = UpdateDescription and k = DelegateK and p = 1) or
    (op = UpdateDescription and k = ConfirmK  and p = 2) or
    (op = UpdateDescription and k = PublishK  and p = 3) or
    (op = UpdateDescription and k = ReportK   and p = 4) or

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

    -- Set Branch Context: gather(0) -> confirm(1) -> write(2) -> report(3)
    -- gather(0) = check branch + check for existing file;
    -- confirm(1) = prompt user for purpose; write(2) = write context file.
    (op = SetBranchContext and k = GatherK  and p = 0) or
    (op = SetBranchContext and k = ConfirmK and p = 1) or
    (op = SetBranchContext and k = WriteK   and p = 2) or
    (op = SetBranchContext and k = ReportK  and p = 3)
}

pred maxPos[op: Operation, p: Int] {
    (op = Commit             and p = 5) or
    (op = Amend              and p = 6) or
    (op = Squash             and p = 6) or
    (op = Rebase             and p = 3) or
    (op = Push               and p = 4) or
    (op = CheckCI            and p = 1) or
    (op = FixCI              and p = 4) or
    (op = Rerun              and p = 3) or
    (op = Watch              and p = 3) or
    (op = FixReview          and p = 6) or
    (op = Reply              and p = 4) or
    (op = UpdateDescription  and p = 4) or
    (op = Correct            and p = 6) or
    (op = SetBranchContext   and p = 3)
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

-- INV-P2: ConfirmK precedes PublishK when both present in the same operation
assert confirmPrecedesPublish {
    all op: Operation, cp: Int, pp: Int |
        (hasStep[op, ConfirmK, cp] and hasStep[op, PublishK, pp])
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
-- for operations like FixReview where the action after confirm is delegation,
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
-- no DelegateK or LoopK steps. WriteK is allowed for local utility writes
-- (e.g. SetBranchContext writes a context file that is not a domain artifact).
-- ConfirmK is allowed for user interaction. PublishK is allowed for external
-- triggers (e.g. Rerun). VerifyK is allowed for post-action checks.
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

-- INV-REF-2: GitPatterns is consumed by most operations.
-- Rerun, CheckCI, and SetBranchContext are intentional exceptions.
assert gitPatternsMostOps {
    all op: Operation |
        (op not in (Rerun + CheckCI + SetBranchContext)) implies
            op in GitPatterns.consumedBy
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
