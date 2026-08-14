# Wealthsimple Observability Platform — Interview Prep

Staff Software Engineer, Observability Platform. Study guide: example questions with answers grounded in real Gadget work. Answers are written in first person so they can be delivered nearly verbatim, then adapted live.

For a plain-English explanation of the underlying concepts, start with [wealthsimple-observability-interview-prep-plain.md](wealthsimple-observability-interview-prep-plain.md).

## Read this first: what they're screening for

The JD telegraphs their worldview: Honeycomb-school observability (wide events, high cardinality, "questions engineers did not anticipate rather than pre-built dashboards"), open standards so vendors are swappable ("switching costs are a configuration change"), platform-as-product, and AI agents as first-class consumers of telemetry. Every answer should quietly reinforce one of four themes:

1. **You've built and operated the exact stack they want** — wide events in a columnar store (ClickHouse), OTel pipelines, high-throughput ingestion, at multi-tenant-platform scale.
2. **You migrate safely** — shadow comparison, flag ramps, staged decommission. Fintech will care deeply about this.
3. **You treat engineers (and agents) as customers** — golden paths, docs, CLIs, skills, MCP.
4. **You measure before you act** — the query-log archive story, EXPLAIN-based CI tests, p99-driven capacity tuning.

## Key numbers to have loaded

- Log store at cutover: **~97 billion rows / ~13 TB** in the v1 table when dropped; full traffic dual-written to v2 for weeks first.
- Query load: **~1.6–2M queries/hour** from log tailing alone — **78% of cluster CPU** — cut with 500ms-base exponential backoff to a 5s cap.
- The two-stage CTE fix: **4.65s OOM (2 GiB memory kill) → 0.77s** on the same production query.
- Tail health after tuning: ~3M tail queries per 6h window at a **0.009% timeout rate**.
- Cluster right-sizing: **120 GB → 96 GB → pinned at 64 GB per replica** (3 replicas), driven by 7-day p99s from a query-log archive table; one failed early attempt at 64 GB taught the lesson about autoscaling churn.
- Ingestion: ~80 Vector pods as a DaemonSet streaming into ClickHouse Buffer tables; fixed a 36–111 **minute** ingestion lag down to ~1s by tuning source read sizes.
- Shadow rollout: every production Loki query mirrored to ClickHouse with a **>20% result-divergence alarm**, per-environment LaunchDarkly ramp, then a 5-stage decommission (code → Vector sinks → Helm → Terraform → buckets).

---

## Part 1 — Event Foundation (wide events, columnar, scale)

### Q1. "Walk me through the architecture of an observability backend you've built. What were the key design decisions?"

**Answer.** At Gadget I led the migration of log storage and search from Loki to ClickHouse, and designed the trace schema on the same cluster. The shape: Vector runs as a DaemonSet on every node, transforming raw Kubernetes logs into OTel-shaped wide events in a single VRL pass — severity normalization, JSON body parsing, trace IDs and flags, resource attributes like host role. Those stream into ClickHouse through Buffer engine tables, which absorb high-frequency small inserts and flush in large parts. ClickHouse's preferred tool for that problem is async inserts, but async-inserted rows aren't visible to queries until the server-side flush lands — and we were replacing Loki, where logs you just wrote show up in a tail within a second. Buffer tables keep the in-memory layer queryable immediately, so we got MergeTree-friendly batching *and* sub-second read-your-writes for log tailing. Traces flow through an OTel collector with transform processors that normalize everything to current semantic conventions before it lands.

The three decisions I'd call load-bearing:

- **Sort key design.** `ORDER BY (EnvironmentId, toStartOfSecond(SystemIngestedAt), Timestamp)` — tenant first because every query is tenant-scoped, then time. Partitioned by *ingest* time rather than event time, because the query patterns (tailing, recent-window search) align with when data arrived, and it makes partition pruning honest even when clients send skewed timestamps.
- **Typed-path JSON for attributes.** Trace attributes live in JSON columns with dynamic paths disabled and an explicit typed path list for the semantic-convention attributes we query (`http.request.method`, `url.path`, `rpc.grpc.status_code`, tenant IDs). You get wide-event flexibility without unbounded schema cardinality, and the hot paths are real typed columns.
- **Search via a materialized column + token index.** A lowercased `FullBody` materialized column concatenating body, attributes, and trace/span IDs, with a text skip index. That gives full-text search over wide events — including "paste a trace ID into log search" — without a separate search system.

**Likely follow-up: "Why ClickHouse over Elasticsearch / Loki / Datadog?"** Loki's model indexes only labels, so it optimizes for pre-anticipated queries — exactly what this team's mission statement rejects. Elasticsearch inverts everything and pays for it in write amplification and storage; I also run an Elasticsearch ingestion pipeline at Gadget, so I've felt both cost models. ClickHouse gives columnar compression, high-cardinality tolerance, and *SQL over raw events* — you can ask questions nobody anticipated. Vendor SaaS pricing at our per-event volume was a non-starter, and OTel-shaped tables keep the exit door open.

### Q2. "Tell me about the hardest performance problem you've debugged in that system."

This is the flagship story; tell it as a diagnostic arc.

**Answer.** After we ramped ClickHouse log search to production, long historical searches started hitting the 2 GiB per-query memory kill. First layer: the query wrapped a Buffer table in `clusterAllReplicas`, and on shared-storage ClickHouse that meant each of three replicas independently scanned the *same* data — 3× read amplification — while the per-tenant dedup clause blocked LIMIT pushdown. I fixed that with a query-shape classifier: if the requested window is entirely older than the buffer boundary, query the base table only; if it straddles, a two-branch UNION with dedup only on the buffer side.

That got us from timeouts to a 4.6-second OOM, which was the interesting layer. I worked out the memory math from first principles: the reverse-time-order K-way merge across ~89 parts holds one granule per part in memory — 8,192 rows — and each row carried ~3 KB of JSON attributes. That's roughly 2 GiB of heavy columns buffered just to merge-sort, when the query only returns a few hundred rows. The fix is classic columnar **late materialization**: a first stage selects only sort-key columns — cheap, tiny rows — ordered by primary key with the LIMIT applied; a second stage collapses those candidates to a min/max time bound; the outer query then fetches the heavy JSON columns only for rows matching the candidate `(id, timestamp)` tuples, with PREWHERE so heavy columns decompress only for passing rows. Same query: 4.65 seconds OOM to 0.77 seconds.

Two things I'd emphasize about *how*: every hypothesis was checked against a query-log archive table I'd built earlier — ClickHouse's `system.query_log` archived into a durable ops database — so I could replay real failing queries and compare profile events, not guess. And the fixes are pinned by CI tests that run `EXPLAIN indexes=1` against a real ClickHouse and assert primary-key usage, granule pruning, and skip-index engagement, so a future refactor can't silently regress the query shape.

### Q3. "How do you think about cardinality?"

**Answer.** Three different regimes, and the mistake is treating them as one problem.

- **In metrics systems, cardinality is series count.** Every unique label combination is a new time series, so an unbounded label — user ID, pod name over time — is a storage explosion. Concrete example: in our Go request router I dual-emitted a renamed Prometheus label during a vocabulary migration, and the design note I wrote was explicit that adding a label *key* carrying the same value set creates zero new series — same cardinality. That's the level of precision the team needs to operate at, because metrics cardinality incidents are the classic observability outage.
- **In columnar stores, cardinality is a design input, not a threat.** ClickHouse is happy with a billion distinct trace IDs in a column; what matters is *where* cardinality sits in the sort key (low-to-high ordering so the primary index prunes effectively), using `LowCardinality` types for enum-ish columns, and partitioning on low-cardinality lifecycle keys. This is exactly why wide-event stores exist: they absorb the cardinality that metrics systems can't, so engineers stop pre-aggregating away the dimensions they'll need during an incident.
- **In dynamic schemas, cardinality is attribute-key explosion.** Our JSON attribute columns cap dynamic paths and type the known ones — otherwise every team's ad-hoc attribute spelling becomes a new subcolumn forever.

The platform-team job is to route each signal to the regime that can afford it: bounded dimensions on metrics, everything else on events.

### Q4. "Design an ingestion pipeline for ~1M events/second. What breaks first?"

**Answer.** Shape: agents/collectors at the edge doing enrichment and normalization (semantic conventions, tenant tagging), a buffered transport, then batched inserts into the columnar store — large batches, because MergeTree-family engines die by a thousand small parts long before they die of volume. I'd put an explicit queue (Kafka/PubSub-class) in front if consumers are heterogeneous or replay matters; at Gadget, Vector's disk buffers plus ClickHouse Buffer tables were sufficient and simpler.

What breaks first, from experience, in order:

1. **Backpressure topology.** A slow sink can starve healthy ones through a shared source. We had a Loki WAL corruption fill Vector's tiny in-memory buffer and stall *ClickHouse and GCS archival* — sinks that were fine — for an hour. Fix: per-sink disk buffers with an explicit drop policy (`drop_newest`) so a dying sink sheds its own load instead of propagating. Design rule: decide *per sink* what you'll drop when it's unhealthy, before it's unhealthy.
2. **Edge read throughput.** Our sandbox nodes lagged ingestion by 36–111 minutes — not the database, the *agent*, reading 4 KB per file per poll cycle across 220 files. One config value (32 KB reads) took lag to ~1 second. Lesson: instrument pipeline lag end-to-end; the bottleneck is usually not where the dashboards point.
3. **Insert concurrency vs. part count.** Too many small concurrent inserts exhaust connections and generate part-merge storms. Batch timeouts and buffer sizing are the levers; we tuned batch timeout 10× at one point to fix connection exhaustion.
4. **Query load interfering with ingest.** Our biggest self-inflicted wound was our own log tailer: 2M queries/hour, 78% of cluster CPU. Workload isolation via per-user settings profiles (memory, threads, concurrency control) is what let ingest and interactive queries share a cluster safely.

### Q5. "How did you validate the Loki → ClickHouse migration was safe to cut over?"

**Answer.** Dual-write first: Vector wrote every log to both stores for weeks, so ClickHouse had full production data with zero user exposure. Then a shadow client: every real production log query ran against Loki (still authoritative) *and* ClickHouse, translated through the exact code path that would serve production later — not a parallel implementation. It logged structured comparisons and escalated to warnings when ClickHouse errored or result counts diverged more than 20%, aggregated into materialized views feeding a Grafana dashboard. That surfaced real translation bugs — substring-vs-token matching semantics, a filter silently reading an empty attribute path — that no synthetic test would have caught. Cutover was a per-environment feature flag ramped over weeks, and decommission was staged: query path removed, then Vector sinks, then Helm releases, then Terraform — each step independently reversible. The principle I'd carry to Wealthsimple: never migrate a query-serving system on faith; make the old system the oracle until the new one has proven itself on real traffic. In a regulated fintech environment that's not just good practice, it's the only defensible way to move.

---

## Part 2 — Standards, sampling, context propagation (includes your gap areas — study these)

### Q6. "How do you set instrumentation and naming standards so they actually get adopted?"

**Answer.** Standards enforced in code beat standards written in wikis, and the pipeline is your enforcement point. Three layers from my experience:

- **Normalize at the collector.** Our OTel collector runs transform processors that migrate deprecated attribute names to current semantic conventions and parse raw values (`user_agent.original`, full URLs) into structured attributes. Teams instrument imperfectly; the pipeline makes the *stored* data consistent anyway. I kept dev and prod collector configs in sync so engineers see the same shapes locally.
- **Make the canonical name a typed artifact.** In our Go router I built a typed telemetry key framework: define a key once and it renders consistently as the slog field, the HTTP header, the Kubernetes label, and the OTLP attribute. Vocabulary drift becomes a compile error instead of a dashboard mystery.
- **Migrate vocabularies with dual-emission.** When we renamed a core domain concept, every surface — headers, Prometheus labels, OTLP — emitted both old and new vocabularies with golden-file tests asserting both, so every consumer migrated on its own schedule, then legacy was removed on a deadline. Standards changes are migrations; treat them with migration discipline.

For Wealthsimple I'd add the layer I didn't have: OTel semantic conventions as the contract in SDK wrappers, so the golden path emits the standard by default and nobody reads the standard document at all.

### Q7. "Explain your approach to sampling. Head vs. tail — when would you use each?" ⚠️ *thin production experience — study this section*

**Honest framing first, then fluency.**

**Answer.** I'll be direct that at Gadget's volume we mostly chose *filtering and aggregation over sampling*, and I think knowing when you don't need sampling is part of the skill. Concretely: our collector drops known-noise spans (span events, health checks, low-value clients) and strips large attributes like `db.statement` — deterministic filtering, not statistical sampling. For billing we needed exact counts, so we aggregated all spans over 5-minute windows with zero sampling — you can't invoice on a sample. And for logs we kept everything queryable but tiered: high-volume user logs go to ClickHouse with a TTL, while the raw stream archives to GCS, so "sampled out" never means "gone."

The conceptual map I'd apply when volume demands real sampling:

- **Head sampling** decides at trace start — `TraceIdRatioBased` keeps a deterministic fraction by trace ID, wrapped in `ParentBased` so a trace is kept or dropped *atomically* (the child honors the parent's decision, propagated via the sampled flag in `traceparent`). Cheap, scales infinitely, but it's blind: it drops the slow, rare, and broken traces at the same rate as the boring ones, and errors are exactly what you want oversampled.
- **Tail sampling** buffers complete traces (collector `tailsampling` processor, or Honeycomb Refinery) and decides with full knowledge: keep all errors, all p99 latency, samples of the rest. Much better data, but now you need trace-complete routing — all spans of a trace to the same buffering node — plus memory for the buffer and a decision timeout. It's an infrastructure commitment.
- **Dynamic/per-key sampling** is the mature end state: sample rate varies by key (route, status), so `POST /login 500` keeps everything while `GET /health 200` keeps 1-in-10,000, and every kept event carries its sample rate so query-time aggregations re-weight correctly. That last part is the detail people miss: sampling without recorded weights silently corrupts every count on every dashboard.

My starting position for a fintech platform: sample nothing that's an error or crosses a money-movement flow, head-sample the long tail of healthy traffic for cost, and treat tail sampling as an investment you make when head sampling's blindness starts costing you incident time.

### Q8. "How does context propagation actually work? Tell me about a time it was broken." 

**Answer.** Mechanics: W3C `traceparent` carries trace ID, parent span ID, and flags across every boundary — HTTP headers, gRPC metadata, message attributes — with `baggage` for key-values you explicitly choose to propagate. In-process it rides context objects (Go `context.Context`, Node async-local storage). Logs join the trace by stamping trace/span IDs on every record — in our schema those are columns *and* part of the full-text index, so trace-ID-to-logs is one search.

The interesting broken case: long-lived websockets. Our config-streaming service created `WS open`/`WS close` spans as children of the connection's boot trace. A websocket can live for hours, so the "boot" trace's duration became the socket's lifetime — every duration chart and trace view for that service was garbage. Parent-child was the wrong relationship: causally related, but not part of that unit of work. I patched our OTel instrumentation library so socket lifecycle spans become *root spans with span links* back to the origin. That's the propagation-design skill in miniature: parent-child expresses "within this work," links express "caused by but independent" — async jobs, batch consumers that process many traces' messages, websockets. Most teams don't know links exist, and their async traces are wrong because of it.

Also worth having: our Vector pipeline emits real `TraceFlags` from the log source rather than hardcoded zeros — small, but it's the difference between logs that know their trace was sampled and logs that lie.

### Q9. "How would you design SLIs and SLOs for a critical user flow?" ⚠️ *conceptual gap — study this section*

Resume says "alerting and SLO dashboards"; be ready to go deeper than dashboards. The JD explicitly says "connecting teams to business impact through SLOs, user flows, and customer experience signals."

**Answer.** Start from the user, not the service. For a Wealthsimple flow like "place a trade" or "e-transfer funds," the SLI is the user-visible outcome: *what fraction of attempts completed successfully within X seconds*, measured as close to the edge as possible — ideally from the request that represents user intent, not from a backend that can succeed while the user still fails. Wide events make this natural: if every request is an event with flow, outcome, latency, and tenant, an SLI is just a query — good events over valid events — rather than new instrumentation.

Then: **SLO** is the target over a window (99.9% over 28 days), and the real product is the **error budget** — the quantified permission to fail that turns "is this reliable enough?" from an argument into arithmetic. Alerting on budget **burn rate**, not raw error rate: multi-window multi-burn-rate (the Google SRE pattern — e.g., page when burning ~14× budget over 1 hour, ticket at ~2× over 6 hours) so a fast catastrophic burn pages immediately while a slow leak files a ticket instead of waking someone. Raw-threshold alerts either flap or miss slow burns; burn-rate alerts are the single biggest alert-quality upgrade most orgs can make.

Platform-team angle: my job isn't writing every team's SLOs — it's making the SLI *derivable from standard telemetry* so defining an SLO is configuration, plus providing burn-rate alerting as a golden path. Where I've done the adjacent thing: our autoscaling runs on custom latency metrics served through the prometheus-adapter — defining a latency indicator good enough to *act on automatically* is the same discipline as defining one good enough to page on.

### Q10. "The JD says 'switching costs are a configuration change rather than a rewrite.' What does that mean to you in practice?"

**Answer.** Instrument against OTel APIs, ship data through an OTel collector you own, and store in open formats — then the vendor is an exporter config. I've lived both sides of this. At Gadget we ran Axiom, Loki, and ClickHouse *simultaneously* during transitions; because everything flowed through Vector and the OTel collector, adding or removing a backend was a sink config and a flag, and when we removed Axiom entirely it was config deletion, not application changes. The counterexample that proves the rule: our web IDE's log search spoke LogQL directly, so the Loki migration required building a LogQL-to-Lucene translation layer to preserve API compatibility for old CLI clients. Every place an application speaks a vendor's query language or SDK directly is future migration debt — which is exactly why the platform team should own thin SDK wrappers and standard query surfaces, so the abstraction boundary exists *before* you need it.

---

## Part 3 — Golden paths, AI agents, staff-level behavior

### Q11. "Tell me about developer-facing tooling you've built. How do you drive adoption?"

**Answer.** Three examples at different altitudes:

- **ggt**, Gadget's open-source CLI, which I created and maintain, plus the platform-side sync protocol behind it. It's the golden path for local development — adoption came from making the right way the easy way: one command, sensible defaults, and the platform team maintaining compatibility (when we migrated log backends, I preserved the old CLI's wire contract with a translation layer so no user ever noticed).
- **Instrumentation libraries**: our shared OTel instrumentation packages — the websocket span-link work shipped as a library option other services just turn on, not as advice.
- **Docs as product**: runbooks, an observability guide, a full docs site for the orchestrator — and, increasingly, docs restructured for LLM consumption, because agents are now half the audience.

On adoption generally: I don't believe in mandates, I believe in defaults plus migration mechanics. The dual-emission vocabulary migration is my model — make both paths work, instrument who's on which, set a deadline, delete the old one. And measure: our search-usage reporting exists precisely so we know which query features engineers actually use.

### Q12. "How would you enable AI agents to investigate production?" (Their "Enable AI Agents" bullet — you have unusually strong material here.)

**Answer.** I've been building exactly this for the past year, and my core finding is that agents need three things: **fast bounded queries, curated context, and guardrailed access.**

- **Access**: I stood up MCP servers for our admin surface and ClickHouse docs, and provisioned a dedicated ClickHouse MCP user — read-only, scoped grants on system tables for introspection, its own settings profile so an agent's runaway query gets killed by the same per-workload limits as any other client. Agents are a workload; give them a workload identity, quotas, and an audit trail (our query-log archive records every query by user, so agent activity is fully attributable).
- **Context**: raw schema access isn't enough — I wrote a ClickHouse operations skill encoding our schemas, query patterns, and cost rules (time-filter-first, correct attribute paths, safe query windows per table), structured as operations like "review this query" and "investigate this environment," with an eval suite that caught real errors in the skill itself. An eight-phase incident-investigation playbook turns "the agent can query" into "the agent investigates the way our best on-call does — cheap queries first, escalating."
- **Foundations**: this only works because the store is fast, high-cardinality SQL over wide events. An agent can't iterate hypotheses against a 90-second dashboard query; sub-second queries change what investigation loops are possible — for humans and agents equally, which is the JD's point.

The bar I'd set at Wealthsimple: an agent on day one can answer "why is this flow degraded?" through MCP against the event foundation, with the same guardrails and audit trail as a junior engineer.

### Q13. "How do you use AI tools in your own workflow?" 

**Answer.** Claude Code is my primary interface to production work — not autocomplete, but agentic workflows I've engineered: custom skills for our operational domains (the ClickHouse skill is the deepest), path-scoped rule files so the agent gets the right context per repo area, and docs restructured so LLMs retrieve them well. The migration work we've discussed was heavily AI-accelerated — with hard quality gates: EXPLAIN-based CI tests, shadow comparison against the old backend, golden-file tests. The discipline is that AI raises throughput while *verification* stays mechanical and non-negotiable — that's how you go fast without lowering the bar, which is what "AI-fluent with high standards" has to mean at staff level. I also maintain the meta-layer: I write and iterate the skills, rules, and hooks that make agents effective for the whole team, which is precisely the "elevate the team's AI-assisted workflows" bullet in this role.

### Q14. "Tell me about diving into an unfamiliar codebase to make a significant change safely." (System Navigator)

**Answer.** The log-search query layer: rather than writing a Lucene-to-SQL translator from scratch, I evaluated HyperDX's open-source common-utils package, vendored it into our monorepo, and adapted it — fixed real bugs in it, upgraded its toolchain, then refactored our client to drive its query builder directly. Someone else's parser and serializer became the foundation of our production search path. Safety came from characterization: EXPLAIN-shape snapshot tests and a seeded perf test that fails CI above a latency budget, so the foreign code's behavior was pinned before I depended on it. Same pattern with the billing forwarder and the OTel collector configs — systems I didn't originate, changed confidently because I built the verification harness first. My general method: read the data model before the code, find the invariants, write the test that would catch me being wrong, then change things.

### Q15. "Tell me about influencing without authority / leading a cross-team initiative." (Staff bar)

**Answer.** The Loki decommission touched the web IDE team's frontend, the CLI's public API, billing's span aggregation, and infrastructure Terraform — none of which I owned. What made it work: I made the migration *observable* (the shadow-comparison dashboard was a shared artifact anyone could check, so "is ClickHouse ready?" was never my opinion), I absorbed the compatibility burden myself (the LogQL translation layer meant other teams' timelines didn't block the cutover), and I sequenced it as independently-shippable reversible stages so no team had to take a leap of faith. My influence model: turn disagreements into measurements, make the migration cost land on the platform team instead of the customers, and write things down — plans, runbooks, decision docs — so alignment survives me being on vacation.

If asked about mentoring: skills/rules/docs *are* leverage-through-artifacts — encoding senior judgment (query cost rules, investigation playbooks) so it transfers without me in the room; plus PR review depth on schema and query changes, where the EXPLAIN test harness turned reviews from "trust me" into "the test asserts the plan."

### Q16. "Why Wealthsimple? Why this team?" 

**Answer angle.** The mission statement reads like the conclusions I reached independently: optimizing for unanticipated questions over pre-built dashboards is exactly why I moved us off Loki's label model onto wide events in ClickHouse; designing for AI agents as investigators is what I've been building with MCP and skills; platform-as-product with engineers as customers is how I already work (CLI, docs sites, golden paths). The delta that excites me: Gadget is ~thousands of tenants on one platform team's stack — Wealthsimple is the chance to do event-foundation work as *the product itself* for a whole engineering org, in a domain (fintech) where the safe-migration discipline I practice is a requirement rather than a nicety. Bonus honesty: hands-on ClickHouse, k8s, and agent-observability are their listed bonuses — that's my exact stack.

---

## Part 4 — Rapid-fire technical (short answers)

**"Buffer tables vs async inserts in ClickHouse?"** Both solve small-insert part explosion, and async inserts are ClickHouse's preferred answer — server-side batching with settings-controlled flush. The difference that decided it for us: async-inserted rows aren't queryable until the flush lands, while a SELECT against a Buffer table reads the in-memory layer *plus* the base table — immediate read-your-writes. We were replacing Loki, which returned just-written logs sub-second in a tail, so async inserts would have regressed the product; Buffer tables preserved sub-second visibility. The costs we learned: with `num_layers=1`, disk I/O saturation blocks inserts (documented in our runbook), the in-memory layer can duplicate rows against the base table (dedup with `LIMIT 1 BY id`), and reads through `clusterAllReplicas` amplify on shared storage.

**"How do skip indexes work?"** Sparse secondary structures per granule-block (e.g., token bloom filters, text indexes): at query time ClickHouse tests the predicate against the index to *skip reading* granules that can't match. They prune I/O, not rows — and only for functions they support: our `hasToken` predicates prune via the text index where a plain `LIKE '%x%'` can't, and we have a CI test asserting exactly that difference in granule counts.

**"MergeTree in one minute?"** Inserts create immutable sorted parts; background merges compact them; the primary key is a sparse index over the sort order (one mark per 8,192-row granule) used for range pruning, not uniqueness. Everything about write and query performance falls out of parts, granules, and merges — batch your inserts, align your sort key with your filters, and never let part counts run away.

**"PREWHERE?"** ClickHouse two-phase filtering: evaluate a cheap predicate first reading only its columns, then read remaining columns only for surviving rows. Usually automatic; I've applied it explicitly so multi-KB JSON columns decompress only for the ~500 rows passing a tuple filter rather than every candidate granule.

**"OTLP vs Prometheus formats?"** OTLP is push-based protobuf for traces/metrics/logs with resource+attribute semantics; Prometheus is pull-based text exposition for metrics with label semantics. They interconvert at the collector, with impedance mismatches (delta vs cumulative temporality, label naming). We run both: OTel for traces/logs, Prometheus for infra metrics and autoscaling signals — right tool per regime.

**"What's `traceparent`?"** W3C header: `version-traceid-parentspanid-flags`. The flags byte carries the sampled bit — how a head-sampling decision propagates so a whole trace is kept or dropped atomically. `tracestate` carries vendor data; `baggage` carries app key-values (use sparingly — it's on every hop).

**"Exemplars?"** Sampled trace IDs attached to metric datapoints (histogram buckets) so you can jump from a latency spike on a dashboard to concrete traces inside it — the metrics-to-traces bridge.

**"Logs vs events?"** A log line is text somebody printed; an event is a structured record of a unit of work with its full context. Our pipeline's job was literally converting the former into the latter at ingest — normalize severity, parse bodies, attach trace/resource context — so the store only ever sees wide events.

---

## Part 4.5 — ClickHouse engine inventory (every table type you actually run)

Sourced from the monorepo's ClickHouse migrations (`packages/api/src/migrations/clickhouse`). Six table types, forming one pipeline: raw wide events land through a write buffer, incremental views fan aggregates into specialized rollup engines, and plain views put stable names on top. Be able to name a *reason* and a *failure mode you've debugged* for each.

### MergeTree — raw event tables (13)

`otel_app_logs_v2`, `otel_traces_v2`, `otel_frontend_traces_v1`, five `otel_app_metrics_*` tables, `app_email_events_v1`, `gadget_deploys`, plus retired v1s.

The foundational engine; everything else is a variant. Every insert writes an immutable sorted part; background merges compact parts; `ORDER BY` doubles as a sparse primary index (one mark per 8,192-row granule) used for range *skipping*, not uniqueness. The log table: `ORDER BY (EnvironmentId, toStartOfSecond(SystemIngestedAt), Timestamp)`, daily partitions, TTL with `ttl_only_drop_parts = 1` so expiry drops whole parts instead of rewriting them.

**Interview nuance:** the DDL says `MergeTree` but production is ClickHouse Cloud, which transparently substitutes **SharedMergeTree** — same semantics, but all replicas read one copy on object storage. That substitution is precisely why `clusterAllReplicas` fan-out caused 3× read amplification (#20463): fan-out helps when replicas own different data, hurts when they share it.

### Buffer — write shock-absorbers (2)

`otel_app_logs_v2_buffer` in front of `otel_app_logs_v2`.

MergeTree hates many small inserts (each makes a part; too many parts = merge storms), but ~80 Vector pods flush every second. Buffer is an in-memory staging table with identical columns; inserts land in RAM, flushing to the base table on thresholds — v2 uses `Buffer(otel, otel_app_logs_v2, 16, 10, 60, 10000, 500000, 10000000, 128000000)`: 16 layers, flush at 10–60s age or row/byte limits, so a row sits at most ~60s in the buffer.

**Why Buffer and not async inserts:** async inserts are ClickHouse's preferred fix for small-insert part explosion, but async-inserted rows aren't visible to SELECTs until the server-side flush completes. Loki returned just-written logs sub-second in a tail; a Buffer table's in-memory layer *is* queryable (SELECTs read buffer + base), so it was the only option that preserved sub-second read-your-writes for log tailing. The v1 Vector sink explicitly disabled async inserts in favor of the buffer (gi#1360).

**Costs personally paid:** reads must consider buffer *and* base (the query-shape classifier and the 70s buffer boundary); the same row can appear in both (hence `LIMIT 1 BY SystemId`); a failed flush rejects whole batches (the `toJSONString` bug, #20509); disk saturation blocks buffer inserts (runbook, gi#1419).

### AggregatingMergeTree — rollups with rich math (24)

`app_errors_10s_*`, `app_triggers_10s_*`, `app_http_responses_10s_v1`, `database_rate_limit_usage_10s/minutely`, `app_background_action_attempts_10s_v4`, the Shopify tables, `billing_spans_agg_per_billing_attribute_per_minute_v1`, the (dropped) `shadow_*_comparison_5min`.

Columns hold **partial aggregation states**, not values — `RateLimitUsageP95 AggregateFunction(quantiles(0.95), UInt64)`. Background merges combine rows with the same sort key by *merging the math*: counts add, avg states combine, quantile sketches merge; queries finalize with `-Merge` functions. This is how canned charts answer "error rate over 30 days" without scanning raw rows — and unlike storing finished numbers, states merge correctly across buckets and late data. **Quotable line:** you can't average averages, but you can merge avg-states.

### SummingMergeTree — rollups when the math is just addition (4)

`field_usage_hourly_v1`, `model_search_usage_hourly_v1/v2`, `nginx_rate_limit_requests_v1`.

On merge, rows with the same sort key have their numeric columns **added** (`SummingMergeTree(UsageCount)`). Rule of thumb (encoded in your own skill, #19735): *pure counters → Summing; anything needing avg/percentile/max/uniq → Aggregating*.

### ReplacingMergeTree — dedup / last-write-wins (2 live)

`billing_events`, `system_query_log_archive_v1`.

On merge, rows with identical sort keys collapse to one, keeping the highest version (`ReplacingMergeTree(event_time_microseconds)`). Makes duplicate-prone writers idempotent: the archive job re-reads recent `system.query_log` windows on retry, and Replacing turns re-inserts into no-ops instead of double counts. **The catch to always state:** dedup happens *eventually, at merge time* — reads see duplicates until then, so correct reads use `FINAL` (as the Grafana panels do) or tolerate dupes.

### Views — two very different things sharing a keyword

- **Incremental materialized views** (~16 `mv_*`): not storage — **insert triggers**. Each fires on every insert into the source table, runs its SELECT over *just that insert block*, and writes into a Summing/Aggregating target. This is the plumbing from raw events to rollups. Failure modes handled: the MV runs inside the insert, so a broken MV rejects source inserts; drop the MV before its target to stop writes cleanly (#19948's ordering).
- **Refreshable materialized view** (`mv_system_query_log_archive_v1`): the opposite model — a scheduled query (`REFRESH EVERY 5 MINUTE`, randomized ±30s, retries) that recomputes and appends. Used because `system.query_log` isn't an insert stream you can trigger on; you must poll it. **Incremental = push, per-insert; refreshable = pull, on a clock.**
- **Plain views** (the `billing_events_*` family): no storage, no schedule — named queries giving billing consumers (Fivetran → Orb) a stable contract decoupled from physical tables, which is what let the v1→v2 migration repoint them without breaking billing.

### The one-breath summary

"Raw wide events land in MergeTree tables — logs, traces, metrics — behind Buffer tables that batch high-frequency writes. Incremental materialized views fan those inserts out into SummingMergeTree counters and AggregatingMergeTree tables holding partial aggregate states for dashboards. ReplacingMergeTree gives us idempotent ingestion where writers can duplicate — billing events and our query-log archive, which a refreshable MV populates on a 5-minute schedule. Plain views sit on top as stable contracts for downstream consumers like billing export."

---

## Part 5 — Gaps to study before the interview (be honest, be prepared)

1. **Sampling in production** (Q7): your story is filtering/aggregation/tiering. Rehearse the head/tail/dynamic taxonomy until fluent; know `ParentBased(TraceIdRatioBased)`, the collector's tail-sampling processor, Refinery, and the sample-rate-reweighting pitfall. Frame as "chose not to at our volume, know exactly how when needed."
2. **SLO formalism** (Q9): learn multi-window multi-burn-rate cold (Google SRE Workbook ch. 5). Your bridge story is autoscaling-on-latency-signals.
3. **Kotlin/Ruby**: JD wants multi-language; you bring Go/TypeScript/Rust. Answer: languages are the cheap part — the JVM/OTel-agent auto-instrumentation model is well-trodden, and platform SDKs are about API design, not language trivia. Skim how OTel works in Kotlin (javaagent, manual spans) and Ruby (SDK, no agent; Rails auto-instrumentation) so you can speak concretely.
4. **Kafka-class streaming**: you used Vector + Pub/Sub + Buffer tables, not Kafka. Be ready for "where would Kafka fit?" — answer: replay, fan-out to heterogeneous consumers, and decoupling ingest spikes from the store; at Gadget's shape the simpler pipeline won, and you have real Pub/Sub experience from the WAL-listener CDC pipeline.
5. **Progressive delivery** (bonus bullet): you have LaunchDarkly flag-ramp experience, not Argo Rollouts. One sentence: same philosophy — measured, reversible exposure — different enforcement layer.

## Questions to ask them

- "What's the current stack, and what does the migration path to the event foundation look like — greenfield store, or evolving what exists?" (Likely Datadog; your vendor-cost + shadow-migration stories land here.)
- "How do teams experience observability cost today — is there a chargeback/showback model, and who feels cardinality pain first?"
- "For the AI-agents bullet: what's live today — MCP against what data, with what guardrails — and what's aspiration?"
- "How does the team measure its own success — adoption metrics, query latency SLOs on the platform itself, incident MTTR?"
- "What's the first thing you'd want this role to ship in 90 days?"

---

## PR reference index

Every claim in this document traces to a PR. Grouped by theme; each entry notes which question(s) it backs. Unmerged PRs are marked — most were drafts superseded by a landed sibling.

### Wide-event schemas & log store (Q1, Q3, rapid-fire)

- [gadget#19624](https://github.com/gadget-inc/gadget/pull/19624) — `otel_app_logs_v2` migration: ingest-time partitioning, `(EnvironmentId, toStartOfSecond(SystemIngestedAt), Timestamp)` sort key, `FullBody` materialized column + full-text search.
- [gadget#18900](https://github.com/gadget-inc/gadget/pull/18900) — `otel_traces_v2` table migration: typed-path JSON columns, ALIAS tenant columns, billing MV re-pointed.
- [global-infrastructure#1378](https://github.com/gadget-inc/global-infrastructure/pull/1378) — `otel_traces_v2` collector pipeline with semantic-convention transforms; JSON `max_dynamic_paths = 0` with explicitly typed semconv paths.
- [global-infrastructure#1360](https://github.com/gadget-inc/global-infrastructure/pull/1360) — platform logs to `otel_logs_v1`: the VRL remap building OTel-shaped rows; bloom/tokenbf indexes, 13-month TTL.
- [gadget#18624](https://github.com/gadget-inc/gadget/pull/18624) — metrics tables rebuilt on JSON type instead of Map with a better primary key for insert performance.
- [gadget#18610](https://github.com/gadget-inc/gadget/pull/18610), [global-infrastructure#1359](https://github.com/gadget-inc/global-infrastructure/pull/1359) — table naming/rename groundwork across client, Vector sink, and Terraform.
- [gadget#20448](https://github.com/gadget-inc/gadget/pull/20448) — `hostname:`/`pid:` search aliases; `host.name` added to the `FullBody` search column.
- [gadget#20509](https://github.com/gadget-inc/gadget/pull/20509) — `toJSONString` vs `toString` on JSON columns: Decimal-scale and Dynamic-subtype insert failures.

### Query engine & performance (Q2, rapid-fire)

- [gadget#20463](https://github.com/gadget-inc/gadget/pull/20463) — query-shape classifier (`table-only`/`union`/`buffer-only`); diagnosed 3× read amplification from `clusterAllReplicas` over shared storage.
- [gadget#20464](https://github.com/gadget-inc/gadget/pull/20464) — the two-stage CTE (late materialization): 4.65s OOM → 0.77s; K-way merge memory math (89 parts × 8,192-row granules × ~3 KB JSON).
- [gadget#20476](https://github.com/gadget-inc/gadget/pull/20476) — two-stage CTE extended to Buffer-engine shapes with in-CTE dedup.
- [gadget#20488](https://github.com/gadget-inc/gadget/pull/20488) — timeout-tail work: PREWHERE on the tuple filter, chunk-on-timeout, buffer boundary 120s→70s, tail survives transient errors.
- [gadget#20359](https://github.com/gadget-inc/gadget/pull/20359) — tail polling backoff: 2M queries/hour at 78% cluster CPU cut with 500ms-base exponential backoff.
- [gadget#20370](https://github.com/gadget-inc/gadget/pull/20370) — transient-error retries (pRetry, error-code taxonomy) and memory/timeout errors mapped to actionable user errors.
- [gadget#20179](https://github.com/gadget-inc/gadget/pull/20179) — unit tests for the EXPLAIN JSON parser and plan-assertion helpers (`expectGranulePruning`, `expectSkipIndexUsed`).
- [gadget#18530](https://github.com/gadget-inc/gadget/pull/18530) — tracing spans on the ClickHouse client itself.

### Lucene query layer (Q10, Q14)

- [gadget#19717](https://github.com/gadget-inc/gadget/pull/19717) — vendored HyperDX `@hyperdx/common-utils` as the `clickhouse-utils` workspace package (the unfamiliar-codebase story).
- [gadget#20134](https://github.com/gadget-inc/gadget/pull/20134) — `ClickhouseLogsClient` driving `SearchQueryBuilder` directly; unconditional text skip index; EXPLAIN CI suite incl. the `hasToken`-prunes-where-`LIKE`-can't assertion.
- [gadget#20135](https://github.com/gadget-inc/gadget/pull/20135) — all LogQL translation unified behind a single Lucene query string.
- [gadget#20178](https://github.com/gadget-inc/gadget/pull/20178) — numeric column aliases: `level:>=warn` compiles to `SeverityNumber >= 40`.
- [gadget#20119](https://github.com/gadget-inc/gadget/pull/20119), [gadget#20284](https://github.com/gadget-inc/gadget/pull/20284), [gadget#20348](https://github.com/gadget-inc/gadget/pull/20348) — level filtering moved into the visible query string; duplicate-query frontend fixes.
- [gadget#20379](https://github.com/gadget-inc/gadget/pull/20379) — user-facing docs migrated from LogQL to Lucene syntax.
- [gadget#20117](https://github.com/gadget-inc/gadget/pull/20117), [gadget#20118](https://github.com/gadget-inc/gadget/pull/20118), [gadget#19927](https://github.com/gadget-inc/gadget/pull/19927) — *unmerged* drafts superseded by #20134/#20135.

### Shadow migration & Loki decommission (Q5, Q15)

- [gadget#19841](https://github.com/gadget-inc/gadget/pull/19841) — `LogsClient` interface + `ShadowLogsClient` comparison logging with the >20% divergence escalation.
- [gadget#19911](https://github.com/gadget-inc/gadget/pull/19911) — shadow correctness: dead attribute-path filter, substring-vs-token semantics, OOM caps.
- [gadget#19735](https://github.com/gadget-inc/gadget/pull/19735) — shadow-comparison materialized views (5-min buckets) + the Grafana dashboard monitoring the migration.
- [gadget#20473](https://github.com/gadget-inc/gadget/pull/20473) — log-export Temporal activity honoring the flag; nanosecond cursor + boundary dedup.
- [gadget#20483](https://github.com/gadget-inc/gadget/pull/20483) — ERROR level mapping fix in the Loki query filter.
- Decommission sequence: [gadget#20505](https://github.com/gadget-inc/gadget/pull/20505) (drop shadow MVs) → [gadget#20512](https://github.com/gadget-inc/gadget/pull/20512) (remove Loki backend; LogQL translation layer preserves the old CLI wire contract) → [gadget#20520](https://github.com/gadget-inc/gadget/pull/20520) (remove Loki frontend) → [global-infrastructure#1788](https://github.com/gadget-inc/global-infrastructure/pull/1788) (stop Vector→Loki writes) → [global-infrastructure#1813](https://github.com/gadget-inc/global-infrastructure/pull/1813) (delete Helm charts/namespaces) → [global-infrastructure#1819](https://github.com/gadget-inc/global-infrastructure/pull/1819) + [#1818](https://github.com/gadget-inc/global-infrastructure/pull/1818) (Terraform teardown: buckets, SA/IAM, Memcache, PSC).
- v1 retirement: [global-infrastructure#1544](https://github.com/gadget-inc/global-infrastructure/pull/1544) (stop v1 ingestion) → [gadget#19926](https://github.com/gadget-inc/gadget/pull/19926) (drop v1: ~97B rows / 13 TB) → [gadget#19947](https://github.com/gadget-inc/gadget/pull/19947) + [gadget#19948](https://github.com/gadget-inc/gadget/pull/19948) (repoint consumers, drop ~49 GiB of dead tables/MVs).
- [global-infrastructure#1650](https://github.com/gadget-inc/global-infrastructure/pull/1650) — Axiom removed entirely (sinks, exporters, secrets): the "vendor exit is config deletion" story.

### Ingestion pipeline & backpressure (Q4)

- [global-infrastructure#1654](https://github.com/gadget-inc/global-infrastructure/pull/1654) — single-pass `otel-app-logs.vrl` pipeline; `vector test` snapshot suite over a sanitized ~7k-line production corpus; raw-stream GCS archival. ([#1651](https://github.com/gadget-inc/global-infrastructure/pull/1651)/[#1652](https://github.com/gadget-inc/global-infrastructure/pull/1652) were *unmerged* precursors.)
- [global-infrastructure#1690](https://github.com/gadget-inc/global-infrastructure/pull/1690) — the backpressure-starvation incident: 10 GB disk buffer with `drop_newest` so a dying sink sheds its own load.
- [global-infrastructure#1695](https://github.com/gadget-inc/global-infrastructure/pull/1695) — 36–111 min ingestion lag → ~1s via source `max_read_bytes` 4 KB→32 KB.
- [global-infrastructure#1391](https://github.com/gadget-inc/global-infrastructure/pull/1391) — insert connection exhaustion fixed by batch-timeout tuning.
- [global-infrastructure#1419](https://github.com/gadget-inc/global-infrastructure/pull/1419) — Buffer-table runbook: `num_layers=1` cascade failure under disk I/O saturation, flush monitoring.
- [global-infrastructure#1423](https://github.com/gadget-inc/global-infrastructure/pull/1423) — ClickHouse sinks promoted from staging to production Vector.
- [vector-conf#13](https://github.com/gadget-inc/vector-conf/pull/13) — sandbox log parsing for the log viewer (earliest Vector work).

### Cluster operations & capacity (Q2 "measure first", key numbers)

- [gadget#19561](https://github.com/gadget-inc/gadget/pull/19561) — `system_query_log_archive_v1`: ReplacingMergeTree + refreshable MV archiving ClickHouse's own query log; the standing measurement instrument. ([gadget#19486](https://github.com/gadget-inc/gadget/pull/19486) was the *unmerged* first draft.)
- [global-infrastructure#1514](https://github.com/gadget-inc/global-infrastructure/pull/1514) — per-workload settings profiles imported into Terraform (7 profiles, 9 user associations).
- Sizing saga: [#1548](https://github.com/gadget-inc/global-infrastructure/pull/1548) (pre-tightening) → [#1549](https://github.com/gadget-inc/global-infrastructure/pull/1549) (64 GB downsize) → [#1552](https://github.com/gadget-inc/global-infrastructure/pull/1552) (revert after outage) → [#1655](https://github.com/gadget-inc/global-infrastructure/pull/1655) (p99-driven limit right-sizing across 7 profiles) → [#1658](https://github.com/gadget-inc/global-infrastructure/pull/1658) (96 GB, stuck) → [#1680](https://github.com/gadget-inc/global-infrastructure/pull/1680) (round-2 tightening: threads 4→2, OOM-driven memory bumps) → [#1730](https://github.com/gadget-inc/global-infrastructure/pull/1730) (pinned 64 GB, autoscaling off).
- [global-infrastructure#1753](https://github.com/gadget-inc/global-infrastructure/pull/1753) — `app_logs_workload` threads 2→4 + `use_concurrency_control`; paired with #20488's chunk-on-timeout.
- [global-infrastructure#1761](https://github.com/gadget-inc/global-infrastructure/pull/1761) — measurement-backed `max_execution_time` 15s→30s (0.009% timeout rate; SQL levers proven ineffective).

### Standards, context propagation & OTel (Q6, Q8)

- [opentelemetry-instrumentations#30](https://github.com/gadget-inc/opentelemetry-instrumentations/pull/30) + [gadget#16987](https://github.com/gadget-inc/gadget/pull/16987) — websocket lifecycle spans as root spans with span links (the propagation-design story).
- [gadget#19941](https://github.com/gadget-inc/gadget/pull/19941) — semantic-convention + attribute-remap OTTL processors synced to the dev collector; processor-ordering and array-normalization correctness fixes.
- [global-infrastructure#1547](https://github.com/gadget-inc/global-infrastructure/pull/1547) — the array-normalization fixes in the production collector (`URL()`/`UserAgent()` silently no-op on list values).
- [global-infrastructure#1691](https://github.com/gadget-inc/global-infrastructure/pull/1691) — OTTL statements rewritten with explicit context prefixes for the collector upgrade.
- [global-infrastructure#1098](https://github.com/gadget-inc/global-infrastructure/pull/1098) — zone-local trace routing (`trafficDistribution: PreferClose`) on the collector LB.
- [skipper#128](https://github.com/gadget-inc/skipper/pull/128) — typed telemetry-key framework: one canonical name rendered as slog field, header, K8s label, and OTLP attribute.
- [skipper#139](https://github.com/gadget-inc/skipper/pull/139) — dual-emit vocabulary migration across headers, Prometheus labels, and OTLP; the explicit series-cardinality reasoning. (*Unmerged at research time.*)
- [skipper#115](https://github.com/gadget-inc/skipper/pull/115) — dual-output structured logging with per-destination level/format.
- [dateilager#113](https://github.com/gadget-inc/dateilager/pull/113) — dropped logs-as-span-events (128-event cap, memory waste, collector drops them anyway).
- [dateilager#130](https://github.com/gadget-inc/dateilager/pull/130) — enriched span attributes on the filesystem service.
- [gadget#17839](https://github.com/gadget-inc/gadget/pull/17839) — *unmerged*: ioredis instrumentation re-added to debug connection spam.

### Filtering & aggregation in lieu of sampling (Q7)

- [billing#17](https://github.com/gadget-inc/billing/pull/17) — billing forwarder aggregates all spans over 5-minute windows with zero sampling.
- [global-infrastructure#1098](https://github.com/gadget-inc/global-infrastructure/pull/1098) — (same PR as above) diff shows the collector's drop filters: span events, health/noise spans, large attributes like `db.statement`.
- [global-infrastructure#1654](https://github.com/gadget-inc/global-infrastructure/pull/1654) — keep-everything tiering: user-visible logs to ClickHouse, raw stream to GCS archival.

### AI agents, MCP & skills (Q12, Q13)

- [global-infrastructure#1512](https://github.com/gadget-inc/global-infrastructure/pull/1512) — MCP server configs (admin + ClickHouse docs) with permission rules.
- [global-infrastructure#1511](https://github.com/gadget-inc/global-infrastructure/pull/1511) — scoped ClickHouse MCP user: read-only introspection grants on system tables.
- [gadget#19546](https://github.com/gadget-inc/gadget/pull/19546) — ClickHouse best-practices skill: 4 operations, 28 rules (PK cardinality ordering, `LowCardinality` types, MV patterns), 23-case eval suite.
- [gadget#19708](https://github.com/gadget-inc/gadget/pull/19708) — 8-phase environment-investigation playbook (cheap→expensive queries, safe windows per table).
- [gadget#19735](https://github.com/gadget-inc/gadget/pull/19735) — Create-Materialized-View operation added to the skill (and applied to the shadow-comparison MVs).
- [gadget#20367](https://github.com/gadget-inc/gadget/pull/20367) — skill restructured to spec (operations/references split).
- [gadget#19599](https://github.com/gadget-inc/gadget/pull/19599) — skill + Grafana dashboards updated for the gRPC migration (SpanKind disambiguation).

### Dashboards-as-code (resume claim support)

- [gadget#19110](https://github.com/gadget-inc/gadget/pull/19110) — Grafana query tool and dashboard management.
- [gadget#19149](https://github.com/gadget-inc/gadget/pull/19149) — consolidated Grafana query docs + dashboards.
- [gadget#19171](https://github.com/gadget-inc/gadget/pull/19171), [gadget#19179](https://github.com/gadget-inc/gadget/pull/19179), [gadget#17428](https://github.com/gadget-inc/gadget/pull/17428) — Tree Owners, Sandbox Assignments, and field-usage ops dashboards.
