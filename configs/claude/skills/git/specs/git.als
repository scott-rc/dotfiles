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
one sig CommitArt, PRArt, GitHubTextArt, WorktreeArt, WatchStateArt extends ArtifactType {}

/** Agents that operations delegate work to */
abstract sig DelegateAgent {}
one sig Committer, PRWriter, GitHubWriter, CITriager, FixSubagent, ExploreSubagent extends DelegateAgent {}

/**
 * References provide domain knowledge (declarative or procedural)
 * consumed by operations. They are structurally separate from Operation:
 * no StepBindings, no agent routing, no artifact lifecycle.
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
    invokes     = none
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

one sig Worktree extends Operation {} {
    produces    = WorktreeArt
    delegatesTo = none
    mutates     = WorktreeArt
    invokes     = none
}

one sig CleanWorktrees extends Operation {} {
    produces    = WorktreeArt
    delegatesTo = none
    mutates     = WorktreeArt
    invokes     = none
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
    invokes     = none
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

one sig Review extends Operation {} {
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

one sig SubmitReview extends Operation {} {
    produces    = GitHubTextArt
    delegatesTo = GitHubWriter
    mutates     = none
    invokes     = none
}

one sig UpdateDescription extends Operation {} {
    produces    = PRArt
    delegatesTo = PRWriter
    mutates     = PRArt
    invokes     = none
}


-- ═══ References ═════════════════════════════════════════════

-- Named reference instances. Each constrains its consumedBy set.

one sig GitPatterns extends Reference {} {
    -- Consumed by most operations. Rerun, SubmitReview, and CheckCI are
    -- intentional exceptions: Rerun uses only gh CLI directly; SubmitReview
    -- delegates entirely to github-writer with no branch/scope/script patterns;
    -- CheckCI only uses gh CLI directly for status gathering.
    consumedBy = Commit + Amend + Squash + Rebase + Push + Worktree
                 + CleanWorktrees + FixCI + Watch
                 + Review + Reply + UpdateDescription
}

one sig GitHubText extends Reference {} {
    consumedBy = Push + Amend + Reply + SubmitReview + UpdateDescription + Watch
}

one sig PRWriterRules extends Reference {} {
    consumedBy = Push + Amend + UpdateDescription
}

one sig BulkThreads extends Reference {} {
    consumedBy = Review + Reply
}

one sig BuildkiteHandling extends Reference {} {
    consumedBy = Watch
}

one sig WatchSubops extends Reference {} {
    consumedBy = Watch
}

-- Scripts modeled as Reference instances

one sig GetPRComments extends Reference {} {
    consumedBy = Review + Reply + Watch
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
one sig IntReview extends Intent {} {
    routesTo = Review
}
one sig IntUpdateDescription extends Intent {} {
    routesTo = UpdateDescription
}
one sig IntReply extends Intent {} {
    routesTo = Reply
}
one sig IntSubmitReview extends Intent {} {
    routesTo = SubmitReview
}
one sig IntReviewAndPush extends Intent {} {
    routesTo = Review + Push
}
one sig IntWorktree extends Intent {} {
    routesTo = Worktree
}
one sig IntCleanWorktrees extends Intent {} {
    routesTo = CleanWorktrees
}


-- ═══ State Machines ══════════════════════════════════════════

/** Step kinds that matter for invariant checking */
abstract sig StepKind {}
one sig GatherK, ConfirmK, DelegateK, WriteK, PublishK, ReportK, LoopK, VerifyK extends StepKind {}

/**
 * StepBinding ties an operation to its ordered step kinds.
 * position encodes the sequence (lower = earlier).
 *
 * We model only the step kinds relevant to invariant checking,
 * not every substep (e.g. "fetch", "detect base", "check protection"
 * fold into GatherK).
 */
sig StepBinding {
    forOp:    one Operation,
    kind:     one StepKind,
    position: one Int
} {
    position >= 0
    position <= 7
}

-- Per-operation step sequences.
-- Each fact uses cardinality + quantifiers to fully constrain its steps.
-- Operations with duplicate StepKinds use `some disj` for uniqueness.

-- Commit: gather(0) -> delegate(1) -> confirm(2) -> delegate(3) -> report(4)
-- delegate(1) = committer; confirm(2) = cohesion choice if mixed concerns;
-- delegate(3) = committer re-invocation with user's file selection.
fact commitSteps {
    #{ sb: StepBinding | sb.forOp = Commit } = 5
    one sb: StepBinding | sb.forOp = Commit and sb.kind = GatherK   and sb.position = 0
    some disj sb1, sb2: StepBinding {
        sb1.forOp = Commit and sb1.kind = DelegateK and sb1.position = 1
        sb2.forOp = Commit and sb2.kind = DelegateK and sb2.position = 3
    }
    one sb: StepBinding | sb.forOp = Commit and sb.kind = ConfirmK  and sb.position = 2
    one sb: StepBinding | sb.forOp = Commit and sb.kind = ReportK   and sb.position = 4
}

-- Amend: gather(0) -> delegate(1) -> verify(2) -> confirm(3) -> publish(4) -> delegate(5) -> report(6)
-- delegate(1) = committer amend; verify(2) = compare file sets;
-- confirm(3) = message update?; publish(4) = force-push;
-- delegate(5) = pr-writer update.
fact amendSteps {
    #{ sb: StepBinding | sb.forOp = Amend } = 7
    one sb: StepBinding | sb.forOp = Amend and sb.kind = GatherK   and sb.position = 0
    some disj sb1, sb2: StepBinding {
        sb1.forOp = Amend and sb1.kind = DelegateK and sb1.position = 1
        sb2.forOp = Amend and sb2.kind = DelegateK and sb2.position = 5
    }
    one sb: StepBinding | sb.forOp = Amend and sb.kind = VerifyK   and sb.position = 2
    one sb: StepBinding | sb.forOp = Amend and sb.kind = ConfirmK  and sb.position = 3
    one sb: StepBinding | sb.forOp = Amend and sb.kind = PublishK  and sb.position = 4
    one sb: StepBinding | sb.forOp = Amend and sb.kind = ReportK   and sb.position = 6
}

-- Squash: gather(0) -> delegate(1) -> write(2) -> verify(3) -> confirm(4) -> delegate(5) -> report(6)
-- delegate(1) = optional commit of uncommitted changes; write(2) = rebase;
-- verify(3) = scope check; confirm(4) = squash?;
-- delegate(5) = committer squash.
fact squashSteps {
    #{ sb: StepBinding | sb.forOp = Squash } = 7
    one sb: StepBinding | sb.forOp = Squash and sb.kind = GatherK   and sb.position = 0
    some disj sb1, sb2: StepBinding {
        sb1.forOp = Squash and sb1.kind = DelegateK and sb1.position = 1
        sb2.forOp = Squash and sb2.kind = DelegateK and sb2.position = 5
    }
    one sb: StepBinding | sb.forOp = Squash and sb.kind = WriteK    and sb.position = 2
    one sb: StepBinding | sb.forOp = Squash and sb.kind = VerifyK   and sb.position = 3
    one sb: StepBinding | sb.forOp = Squash and sb.kind = ConfirmK  and sb.position = 4
    one sb: StepBinding | sb.forOp = Squash and sb.kind = ReportK   and sb.position = 6
}

-- Rebase: gather(0) -> write(1) -> verify(2) -> report(3)
-- No delegation -- rebase is inline. verify(2) = scope check.
fact rebaseSteps {
    #{ sb: StepBinding | sb.forOp = Rebase } = 4
    one sb: StepBinding | sb.forOp = Rebase and sb.kind = GatherK  and sb.position = 0
    one sb: StepBinding | sb.forOp = Rebase and sb.kind = WriteK   and sb.position = 1
    one sb: StepBinding | sb.forOp = Rebase and sb.kind = VerifyK  and sb.position = 2
    one sb: StepBinding | sb.forOp = Rebase and sb.kind = ReportK  and sb.position = 3
}

-- Push: gather(0) -> publish(1) -> verify(2) -> delegate(3) -> report(4)
-- publish(1) = git push to remote; verify(2) = check PR state;
-- delegate(3) = pr-writer creates/updates PR.
fact pushSteps {
    #{ sb: StepBinding | sb.forOp = Push } = 5
    one sb: StepBinding | sb.forOp = Push and sb.kind = GatherK   and sb.position = 0
    one sb: StepBinding | sb.forOp = Push and sb.kind = PublishK  and sb.position = 1
    one sb: StepBinding | sb.forOp = Push and sb.kind = VerifyK   and sb.position = 2
    one sb: StepBinding | sb.forOp = Push and sb.kind = DelegateK and sb.position = 3
    one sb: StepBinding | sb.forOp = Push and sb.kind = ReportK   and sb.position = 4
}

-- Worktree: gather(0) -> write(1) -> confirm(2) -> write(3) -> report(4)
-- write(1) = gwt; confirm(2) = branch exists?; write(3) = gwt --force or retry.
fact worktreeSteps {
    #{ sb: StepBinding | sb.forOp = Worktree } = 5
    one sb: StepBinding | sb.forOp = Worktree and sb.kind = GatherK  and sb.position = 0
    some disj sb1, sb2: StepBinding {
        sb1.forOp = Worktree and sb1.kind = WriteK and sb1.position = 1
        sb2.forOp = Worktree and sb2.kind = WriteK and sb2.position = 3
    }
    one sb: StepBinding | sb.forOp = Worktree and sb.kind = ConfirmK and sb.position = 2
    one sb: StepBinding | sb.forOp = Worktree and sb.kind = ReportK  and sb.position = 4
}

-- Clean Worktrees: gather(0) -> write(1) -> report(2) -> confirm(3) -> write(4) -> report(5)
-- write(1) = fetch/discover stale; report(2) = present stale worktrees;
-- confirm(3) = deletion?; write(4) = delete/prune; report(5) = summary.
fact cleanWorktreesSteps {
    #{ sb: StepBinding | sb.forOp = CleanWorktrees } = 6
    one sb: StepBinding | sb.forOp = CleanWorktrees and sb.kind = GatherK  and sb.position = 0
    some disj sb1, sb2: StepBinding {
        sb1.forOp = CleanWorktrees and sb1.kind = WriteK  and sb1.position = 1
        sb2.forOp = CleanWorktrees and sb2.kind = WriteK  and sb2.position = 4
    }
    some disj sb3, sb4: StepBinding {
        sb3.forOp = CleanWorktrees and sb3.kind = ReportK and sb3.position = 2
        sb4.forOp = CleanWorktrees and sb4.kind = ReportK and sb4.position = 5
    }
    one sb: StepBinding | sb.forOp = CleanWorktrees and sb.kind = ConfirmK and sb.position = 3
}

-- CheckCI: gather(0) -> report(1)
fact checkCISteps {
    #{ sb: StepBinding | sb.forOp = CheckCI } = 2
    one sb: StepBinding | sb.forOp = CheckCI and sb.kind = GatherK and sb.position = 0
    one sb: StepBinding | sb.forOp = CheckCI and sb.kind = ReportK and sb.position = 1
}

-- FixCI: gather(0) -> delegate(1) -> delegate(2) -> report(3)
-- delegate(1) = ci-triager triages failures; delegate(2) = fix subagent applies fixes.
-- Consistent with Review which also models fix subagent work as DelegateK.
fact fixCISteps {
    #{ sb: StepBinding | sb.forOp = FixCI } = 4
    one sb: StepBinding | sb.forOp = FixCI and sb.kind = GatherK and sb.position = 0
    some disj sb1, sb2: StepBinding {
        sb1.forOp = FixCI and sb1.kind = DelegateK and sb1.position = 1
        sb2.forOp = FixCI and sb2.kind = DelegateK and sb2.position = 2
    }
    one sb: StepBinding | sb.forOp = FixCI and sb.kind = ReportK and sb.position = 3
}

-- Rerun: gather(0) -> publish(1) -> verify(2) -> report(3)
-- publish(1) = gh run rerun; verify(2) = check new status.
fact rerunSteps {
    #{ sb: StepBinding | sb.forOp = Rerun } = 4
    one sb: StepBinding | sb.forOp = Rerun and sb.kind = GatherK  and sb.position = 0
    one sb: StepBinding | sb.forOp = Rerun and sb.kind = PublishK and sb.position = 1
    one sb: StepBinding | sb.forOp = Rerun and sb.kind = VerifyK  and sb.position = 2
    one sb: StepBinding | sb.forOp = Rerun and sb.kind = ReportK  and sb.position = 3
}

-- Watch: gather(0) -> report(1) -> loop(2) -> report(3)
-- ReportK appears at both position 1 (initial status) and position 3 (final summary).
fact watchSteps {
    #{ sb: StepBinding | sb.forOp = Watch } = 4
    one sb: StepBinding | sb.forOp = Watch and sb.kind = GatherK and sb.position = 0
    some disj sb1, sb2: StepBinding {
        sb1.forOp = Watch and sb1.kind = ReportK and sb1.position = 1
        sb2.forOp = Watch and sb2.kind = ReportK and sb2.position = 3
    }
    one sb: StepBinding | sb.forOp = Watch and sb.kind = LoopK and sb.position = 2
}

-- Review: gather(0) -> report(1) -> delegate(2) -> report(3)
-- No ConfirmK -- user invoked "fix review feedback" so intent is clear.
-- ReportK appears at both position 1 (summary) and position 3 (verify fixes).
fact reviewSteps {
    #{ sb: StepBinding | sb.forOp = Review } = 4
    one sb: StepBinding | sb.forOp = Review and sb.kind = GatherK   and sb.position = 0
    some disj sb1, sb2: StepBinding {
        sb1.forOp = Review and sb1.kind = ReportK and sb1.position = 1
        sb2.forOp = Review and sb2.kind = ReportK and sb2.position = 3
    }
    one sb: StepBinding | sb.forOp = Review and sb.kind = DelegateK and sb.position = 2
}

-- Reply: gather(0) -> report(1) -> confirm(2) -> publish(3) -> report(4)
-- ReportK appears at both position 1 (present drafts) and position 4 (summary).
-- PublishK (not DelegateK) at position 3: github-writer posting is modeled as
-- PublishK to enforce INV-P2 (confirm-before-publish). SubmitReview uses DelegateK
-- because the user's explicit verdict serves as implicit confirmation.
fact replySteps {
    #{ sb: StepBinding | sb.forOp = Reply } = 5
    one sb: StepBinding | sb.forOp = Reply and sb.kind = GatherK  and sb.position = 0
    some disj sb1, sb2: StepBinding {
        sb1.forOp = Reply and sb1.kind = ReportK and sb1.position = 1
        sb2.forOp = Reply and sb2.kind = ReportK and sb2.position = 4
    }
    one sb: StepBinding | sb.forOp = Reply and sb.kind = ConfirmK and sb.position = 2
    one sb: StepBinding | sb.forOp = Reply and sb.kind = PublishK and sb.position = 3
}

-- Submit Review: gather(0) -> confirm(1) -> delegate(2) -> report(3)
-- confirm(1) = verdict clarification if ambiguous.
fact submitReviewSteps {
    #{ sb: StepBinding | sb.forOp = SubmitReview } = 4
    one sb: StepBinding | sb.forOp = SubmitReview and sb.kind = GatherK   and sb.position = 0
    one sb: StepBinding | sb.forOp = SubmitReview and sb.kind = ConfirmK  and sb.position = 1
    one sb: StepBinding | sb.forOp = SubmitReview and sb.kind = DelegateK and sb.position = 2
    one sb: StepBinding | sb.forOp = SubmitReview and sb.kind = ReportK   and sb.position = 3
}

-- Update Description: gather(0) -> delegate(1) -> report(2)
fact updateDescriptionSteps {
    #{ sb: StepBinding | sb.forOp = UpdateDescription } = 3
    one sb: StepBinding | sb.forOp = UpdateDescription and sb.kind = GatherK   and sb.position = 0
    one sb: StepBinding | sb.forOp = UpdateDescription and sb.kind = DelegateK and sb.position = 1
    one sb: StepBinding | sb.forOp = UpdateDescription and sb.kind = ReportK   and sb.position = 2
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
    all op: Operation |
        (some sp: StepBinding | sp.forOp = op and sp.kind = PublishK) implies
            some sg: StepBinding |
                sg.forOp = op and sg.kind = GatherK and
                (all sp: StepBinding |
                    (sp.forOp = op and sp.kind = PublishK) implies
                        sg.position < sp.position)
}

-- INV-P2: ConfirmK precedes PublishK when both present in the same operation
assert confirmPrecedesPublish {
    all op: Operation |
        (some sc: StepBinding | sc.forOp = op and sc.kind = ConfirmK) and
        (some sp: StepBinding | sp.forOp = op and sp.kind = PublishK)
        implies
            all sc, sp: StepBinding |
                (sc.forOp = op and sc.kind = ConfirmK and
                 sp.forOp = op and sp.kind = PublishK)
                implies sc.position < sp.position
}

-- INV-SM-1: Every operation starts with GatherK at position 0
assert allOpsStartWithGather {
    all op: Operation |
        some sb: StepBinding | sb.forOp = op and sb.kind = GatherK and sb.position = 0
}

-- INV-SM-2: Every operation ends with ReportK at its highest position
assert allOpsEndWithReport {
    all op: Operation |
        some sb: StepBinding |
            sb.forOp = op and sb.kind = ReportK and
            no sb2: StepBinding |
                sb2.forOp = op and sb2.position > sb.position
}

-- INV-SM-3: Every ConfirmK must have at least one WriteK after it
-- (weakened from "all ConfirmK < all WriteK" to support operations
-- like Worktree and CleanWorktrees where confirm appears between writes)
assert confirmPrecedesWrite {
    all op: Operation |
        all sc: StepBinding |
            (sc.forOp = op and sc.kind = ConfirmK) implies
                (no sw: StepBinding | sw.forOp = op and sw.kind = WriteK) or
                (some sw: StepBinding |
                    sw.forOp = op and sw.kind = WriteK and
                    sc.position < sw.position)
}

-- INV-SM-4: Operations that delegate must have an action step
assert delegationImpliesActionStep {
    all op: Operation |
        some op.delegatesTo implies
            some sb: StepBinding |
                sb.forOp = op and sb.kind in (DelegateK + PublishK + LoopK)
}

-- INV-SM-5: Operations that mutate must have an action step
assert mutationImpliesActionStep {
    all op: Operation |
        some op.mutates implies
            some sb: StepBinding |
                sb.forOp = op and sb.kind in (WriteK + DelegateK + PublishK + LoopK)
}

-- INV-SM-6: Domain-empty operations have no internal mutation steps
-- PublishK and VerifyK are allowed — PublishK covers external triggers like
-- CI re-runs that don't produce local artifacts (e.g. Rerun); VerifyK is
-- a post-action check that reads but does not mutate.
assert domainEmptyOpsHaveNoMutationSteps {
    all op: Operation |
        (no op.produces and no op.delegatesTo and no op.mutates) implies
            all sb: StepBinding |
                sb.forOp = op implies sb.kind in (GatherK + ReportK + PublishK + VerifyK)
}

-- INV-SM-7: VerifyK must be preceded by an action step
-- (DelegateK, WriteK, PublishK, or LoopK) in the same operation
assert verifyFollowsAction {
    all op: Operation |
        all sv: StepBinding |
            (sv.forOp = op and sv.kind = VerifyK) implies
                some sa: StepBinding |
                    sa.forOp = op and
                    sa.kind in (DelegateK + WriteK + PublishK + LoopK) and
                    sa.position < sv.position
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
-- Rerun, SubmitReview, and CheckCI are intentional exceptions and are excluded.
-- The assert checks that every op except these three consumes GitPatterns.
assert gitPatternsMostOps {
    all op: Operation |
        (op not in (Rerun + SubmitReview + CheckCI)) implies op in GitPatterns.consumedBy
}

-- INV-REF-3: GitHubText consumed by all ops that produce GitHubTextArt or PRArt
assert githubTextMatchesProduction {
    all op: Operation |
        (GitHubTextArt in op.produces or PRArt in op.produces) implies
            op in GitHubText.consumedBy
}


-- ═══ Verification ════════════════════════════════════════════

-- Delegation
check committerDelegatedCorrectly        for 5 but exactly 69 StepBinding, 4 Int
check prWriterDelegatedCorrectly         for 5 but exactly 69 StepBinding, 4 Int
check githubWriterDelegatedCorrectly     for 5 but exactly 69 StepBinding, 4 Int
check exploreSubagentMatchesBulkThreads  for 5 but exactly 69 StepBinding, 4 Int
check mutatesSubsetOfProduces            for 5 but exactly 69 StepBinding, 4 Int
check ciTriagerImpliesFixSubagent        for 5 but exactly 69 StepBinding, 4 Int

-- Publish safety
check publishPrecededByGather            for 5 but exactly 69 StepBinding, 4 Int
check confirmPrecedesPublish             for 5 but exactly 69 StepBinding, 4 Int

-- State machines
check allOpsStartWithGather              for 5 but exactly 69 StepBinding, 4 Int
check allOpsEndWithReport                for 5 but exactly 69 StepBinding, 4 Int
check confirmPrecedesWrite               for 5 but exactly 69 StepBinding, 4 Int
check delegationImpliesActionStep        for 5 but exactly 69 StepBinding, 4 Int
check mutationImpliesActionStep          for 5 but exactly 69 StepBinding, 4 Int
check domainEmptyOpsHaveNoMutationSteps  for 5 but exactly 69 StepBinding, 4 Int
check verifyFollowsAction                for 5 but exactly 69 StepBinding, 4 Int

-- Routing
check allOpsReachable                    for 5 but exactly 69 StepBinding, 4 Int
check allAgentsUsed                      for 5 but exactly 69 StepBinding, 4 Int

-- Composition
check invokedOpsReachable                for 5 but exactly 69 StepBinding, 4 Int

-- References
check referencesAreLeaves                for 5 but exactly 69 StepBinding, 4 Int
check gitPatternsMostOps                 for 5 but exactly 69 StepBinding, 4 Int
check githubTextMatchesProduction        for 5 but exactly 69 StepBinding, 4 Int


-- ═══ Examples ════════════════════════════════════════════════

-- Show a valid instance of the full model
run showModel {} for 5 but exactly 69 StepBinding, 4 Int

-- Show all operations that mutate artifacts
run showMutatingOps {
    some op: Operation | some op.mutates
} for 5 but exactly 69 StepBinding, 4 Int

-- Show all operations that delegate to agents
run showDelegatingOps {
    some op: Operation | some op.delegatesTo
} for 5 but exactly 69 StepBinding, 4 Int

-- Show operations with confirm-before-publish safety
run showConfirmBeforePublish {
    some op: Operation |
        (some sb: StepBinding | sb.forOp = op and sb.kind = ConfirmK) and
        (some sb: StepBinding | sb.forOp = op and sb.kind = PublishK)
} for 5 but exactly 69 StepBinding, 4 Int
