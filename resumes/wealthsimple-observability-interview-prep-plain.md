# Wealthsimple Observability Prep — Plain-English Edition

Companion to [wealthsimple-observability-interview-prep.md](wealthsimple-observability-interview-prep.md). That file has the polished, technical answers. This one explains what everything actually *means* — the concepts from scratch, what you did in ordinary language, and why anyone cares. Read this first; drill the other one after.

---

## The big picture: what is this team even for?

When something breaks in production at 2am, an engineer needs to answer "what's happening and why?" **Observability** is everything that makes that possible: the data your systems emit about themselves (logs, metrics, traces), the place that data is stored, and the tools for asking questions of it.

Wealthsimple's Observability Platform team builds that tooling for *every other engineer at the company*. They're not debugging the trading system themselves — they're building the thing engineers use to debug the trading system. That's what "platform as a product, engineers as customers" means: your users are your coworkers.

**Your one-sentence fit:** you did this exact job at Gadget for four years — you built and ran the system every Gadget engineer uses to investigate production, and you migrated it from an older stack (Loki) to a modern one (ClickHouse) without anyone noticing a bump.

---

## The three kinds of telemetry (and why "wide events" is a fourth)

- **Logs** — text lines a program prints. "Payment failed for user 123." Easy to produce, messy to search.
- **Metrics** — numbers over time. "Requests per second: 4,512." Cheap to store, great for dashboards, but pre-summarized: you can see *that* errors spiked, not *which requests* failed.
- **Traces** — the story of one request as it hops between services. Service A called B, B called C, C took 4 seconds — there's your problem. Each step is called a **span**.

### So what's a "wide event"?

The core idea of the school of thought Wealthsimple subscribes to (popularized by Honeycomb): instead of a metric with 3 labels or a bare log line, emit **one rich record per unit of work** — a request, a job, a page load — carrying *everything* you know about it: who, what, where, how long, what happened, plus any business context (which app, which customer, which feature flag). Maybe 50–200 fields per event. That's "wide."

**Why it wins:** with metrics you must decide *in advance* which questions matter ("track errors by region"). With wide events you keep the raw material and slice it *later*, any way you want: "show me failed trades, over $10k, from iOS, in Ontario, on the new pricing code path." Nobody planned for that question — and that's the point. The job description's line "we optimize for the questions engineers did not anticipate" is this philosophy verbatim.

**What you did:** your ClickHouse log and trace rows *are* wide events. Every log row carries the message plus a bag of attributes (app ID, environment, pod, trace ID, severity, arbitrary JSON). Every trace span carries dozens of standardized fields (HTTP method, URL path, gRPC status…). And you wrote the pipeline code (in a tool called Vector) that takes raw messy Kubernetes log lines and *turns them into* these structured wide events on the way to storage.

---

## Columnar storage, or: why ClickHouse?

Regular databases (Postgres, MySQL) store data **row by row**: all of row 1's fields together, then all of row 2's. Great when you want everything about one record ("load user 123's profile").

**Columnar** databases (ClickHouse) store data **column by column**: all the timestamps together, all the error codes together, all the messages together. Two consequences:

1. **Queries only read the columns they touch.** "Count errors by hour over 30 days" reads two columns and ignores the other 60. Over billions of rows, that's the difference between milliseconds and minutes.
2. **Compression gets absurdly good.** A column of a million near-identical values ("error", "error", "info"…) squeezes down to almost nothing.

That's why columnar stores are the natural home for wide events: billions of rows, wide records, and analytical questions ("count / group / top-N") rather than "fetch record 123."

**Loki (what you migrated away from)** takes the opposite bet: it only indexes a few pre-chosen labels and stores log text in compressed blobs. Cheap, but you can basically only ask questions along the labels someone chose in advance — the exact thing Wealthsimple's mission statement rejects.

**What you did:** moved Gadget's logs — the old system held about **97 billion rows, 13 terabytes** — from Loki into ClickHouse, and designed how the tables are laid out. Which brings us to…

### The three design decisions that matter in a ClickHouse table

You'll be asked about these; here's the intuition:

- **Sort key ("what order is data physically stored in?").** ClickHouse keeps data sorted, and can skip huge chunks of it when your query filters match the sort order — like finding a name in a phone book vs. a shuffled pile. You sorted by **customer environment first, then time**, because every query is "this customer's logs in this time range." One design choice makes every query fast.
- **How to store the grab-bag attributes.** Wide events have unpredictable fields. You used ClickHouse's JSON columns but *pinned the important fields to real typed columns* and capped the dynamic ones — flexibility without letting every team's typo'd attribute name permanently bloat the schema.
- **Full-text search.** Engineers want to type "timeout" and find logs. You built a hidden combined-text column with a token index over it — think of a book's index in the back: it doesn't tell you the answer, it tells you which pages you can *skip*. Same trick makes "paste a trace ID into log search" work.

---

## The six kinds of tables you ran (and why each exists)

ClickHouse tables come in flavors called "engines," and Gadget's schema uses six. They're easiest to understand as one assembly line: **events arrive → get buffered → get stored raw → get summarized → get named.**

**1. MergeTree — the warehouse floor.** The basic engine; everything raw lives here (logs, traces, metrics). How it works: every batch of inserts is written as an immutable sorted file; background workers constantly merge small files into bigger ones (hence "merge tree"). Data stays sorted, so queries can skip whole sections like a phone book. It has one dislike: *lots of tiny inserts*, because every insert makes a new file and the mergers can't keep up. Which is why you also had…

**2. Buffer — the loading dock.** A thin staging table sitting in front of the log table, holding rows in memory and flushing them downstairs in big batches (at most every ~60 seconds). Your ~80 log shippers each write every second — the Buffer turns that flood of tiny inserts into the large batches MergeTree wants. ClickHouse's *recommended* tool for this problem is actually a feature called async inserts (the server batches for you) — but async-inserted rows are **invisible to queries until the batch lands**, sometimes seconds later. You were replacing Loki, where a log you just wrote shows up in a live tail within a second, and the Buffer's memory layer *is* searchable the instant a row arrives (queries read the dock *and* the warehouse). So the Buffer was the only option that kept live log tailing feeling instant. The trade-offs you lived: queries have to check both places (your "buffer boundary" logic), a row can briefly exist in both (you dedupe on an ID), and if a flush fails it rejects the whole batch (a real bug you fixed).

**3. AggregatingMergeTree — the running scoreboard, with real math.** 24 of these power the dashboards. Instead of raw events, each row is a pre-computed summary for a time bucket — "errors for app X in this 10-second window" — so charts over 30 days never touch billions of raw rows. The clever part: rows don't store finished numbers, they store *mergeable math-in-progress* (a partially-computed average, a percentile sketch). When two rows for the same bucket meet, the math merges correctly. Why that matters: **you can't average two averages** (10 avg of 2 items + 20 avg of 1000 items ≠ 15), but these mergeable states combine correctly no matter how data arrives.

**4. SummingMergeTree — the tally counter.** The simple sibling, used when the only math is addition: "how many times was this field used this hour." Rows with the same key just have their counts added together. Rule of thumb from your own internal guide: *plain counters → Summing; averages/percentiles/maxes → Aggregating.*

**5. ReplacingMergeTree — the "keep the latest copy" table.** Used for billing events and your query-performance archive. If the same row is written twice — say a retry re-sends data — duplicates with the same key eventually collapse to one, keeping the newest. That makes retries safe: writing twice ends up meaning writing once, which billing very much requires. The fine print you always mention: "eventually" — until a background merge runs, a query can still see both copies, so careful readers ask for deduplication explicitly (the `FINAL` keyword).

**6. Views — three kinds of glue.**
- **Incremental materialized views** are *insert triggers*: every time raw logs arrive, the view runs a little query over just that batch and writes summaries into the scoreboards (#3 and #4). This is the conveyor belt between raw and summarized. Push-based: fires per insert.
- **A refreshable materialized view** is a *scheduled job in table form*: every 5 minutes it re-runs a query and appends the results. You used it to archive ClickHouse's own performance log, which you can't trigger on — you have to poll it. Pull-based: fires on a clock.
- **Plain views** are just *saved queries with a name* — no storage at all. Your billing exports read from views like `billing_events_database_queries`, so when you rebuilt the physical tables underneath, billing never noticed. A stable name over changeable plumbing.

**Why interviewers care:** most candidates know "ClickHouse is a fast columnar database." Being able to say *which engine you chose for which job and what went wrong with each* — the buffer flush that rejected batches, the average-of-averages trap, the eventual-dedup fine print — is what separates "used it" from "ran it."

---

## Your best war story, in plain English (learn to tell this one)

After launch, some searches over long time ranges crashed with out-of-memory errors. The debugging went in two layers:

**Layer 1 — the cluster was doing triple work.** The query was written to fan out to all 3 replica servers. On old-style clusters each replica has its *own* copy of the data, so fanning out splits the work. But this cluster uses *shared* storage — all 3 replicas read the *same* data. So fan-out meant every replica scanned everything: 3× the work for zero benefit. Fix: only fan out when actually needed (very recent data lives in per-replica memory buffers; old data doesn't).

**Layer 2 — the query carried its luggage too early.** Even after that, a query needing the newest 200 log lines was loading the *full fat rows* — including ~3 KB of JSON attributes each — for huge numbers of candidate rows, just to sort them and throw most away. Loading everything before deciding what you need is like packing your whole house to move and then unpacking 99% of it. The fix, a classic columnar pattern called **late materialization**: first pass looks at *only* the tiny sorting columns (IDs and timestamps) to decide which 200 rows win; second pass fetches the heavy JSON *only for those 200*. Result: a query that died at 4.65 seconds using 2 GB of memory now finishes in **0.77 seconds**.

**Why interviewers love this story:** you didn't guess. You had built a table that archives ClickHouse's own query performance log, so you could replay the exact failing production queries and do the memory arithmetic. And you locked the fix in with automated tests that check the query *plan* (does it use the index? does it skip data?) so nobody can silently break it later.

---

## Cardinality (the word that will definitely come up)

**Cardinality = how many distinct values a field has.** "HTTP method" has ~8. "Customer ID" has millions. "Request ID" is unique every time — effectively infinite.

Why it's *the* observability word:

- **In metrics systems (Prometheus, Datadog metrics), high cardinality is a bomb.** Every unique combination of label values becomes its own stored time series. Add a `customer_id` label to a metric and you've turned 1 series into 10 million. This is the classic way companies get shock Datadog bills or take down their metrics stack.
- **In columnar event stores, high cardinality is fine — that's the selling point.** ClickHouse shrugs at a billion distinct customer IDs in a column. This is *why* the wide-events crowd exists: put bounded fields on metrics, put everything else in events, and stop throwing away the detailed fields you'll desperately want during an incident.
- **In table design, cardinality tells you how to arrange things.** Sort low-cardinality fields before high-cardinality ones in the sort key (phone book: sort by last name, then first — not the reverse). Use special compact types for fields with few values.

**What you did:** all three. You wrote a design note during a metrics-label migration explicitly reasoning that adding a renamed label key with the same values creates *zero* new series (that's the precise vocabulary in action). You encoded sort-key and type rules into an internal best-practices guide. And your JSON attribute design caps "how many distinct field *names* can exist" — cardinality control one level up.

---

## Sampling (your weak spot — understand it, don't fake it)

**The problem:** at high volume you can't afford to keep every trace. **Sampling = keeping a fraction.** Three strategies:

- **Head sampling:** decide *when the request starts* — e.g., keep 1 in 100, chosen by trace ID so every service keeps the *same* traces (the decision travels in the trace headers). Cheap and simple, but blind: it throws away errors and slow requests at the same rate as boring healthy ones — and errors are the ones you wanted.
- **Tail sampling:** wait until the request *finishes*, look at the whole trace, then decide. "Keep every error, keep everything slower than 2s, keep 1% of the rest." Much smarter data, but expensive: something must buffer every in-flight trace in memory and route all pieces of a trace to the same place.
- **Dynamic sampling:** different rates per category — keep 100% of `payment-failed`, 0.01% of `health-check-ok`. The subtle trap: every kept event must *record its sample rate* so that when you count things later, a kept 1-in-1000 event counts as 1000. Skip that and every dashboard silently lies.

**Your honest story:** Gadget deliberately *didn't* sample — you filtered and aggregated instead. The collector drops known-noise spans (health checks, chatty internals) and oversized fields — deterministic trimming, not statistics. Billing counted **every** span, aggregated over 5-minute windows, because you can't send customers an invoice based on a sample. And "dropped" logs still went to cheap cloud-storage archive, so nothing was truly gone. In the interview: "at our volume, filtering and aggregation beat sampling — here's the taxonomy, and here's when I'd reach for tail sampling" is a *stronger* answer than pretending.

---

## Context propagation (how the pieces of a story find each other)

A request touches six services. How do six sets of logs and spans get stitched into one story? Every service **passes a shared ID along** — a standard HTTP header called `traceparent` carrying the trace ID (plus the parent span ID and a "was this sampled?" flag). Each service tags everything it emits with that ID. Stitching becomes a lookup.

Rule of thumb: **traces answer "where did the time go," logs answer "what exactly happened at that spot"** — and propagation is what lets you jump between them. That's why your log rows store trace IDs as searchable columns.

**Your good story here — websockets.** Gadget has connections that stay open for *hours*. The instrumentation made the connection's open/close spans *children* of the request that started the connection — so that request's trace appeared to take hours, wrecking every latency chart. The insight: parent/child means "part of the same piece of work," and a long-lived connection *isn't* part of the request that opened it — it's merely *caused by* it. OpenTelemetry has a lesser-known feature for exactly this, **span links** ("related, but separate story"). You patched the shared instrumentation library so connection spans become independent traces *linked* back to their origin. Most engineers don't know links exist; using them correctly is a quiet signal of depth.

---

## OpenTelemetry & "open standards" (the vendor-freedom argument)

**OpenTelemetry (OTel)** is the industry-standard, vendor-neutral way to produce telemetry: standard SDKs, a standard wire format, standard *names* for common fields ("semantic conventions" — everyone calls it `http.request.method`, not their own spelling), and a **collector** — a pipeline service that receives everything, cleans/transforms it, and ships it wherever you point it.

Why the job description cares ("switching costs are a configuration change rather than a rewrite"): if apps code against OTel and everything flows through a collector *you* control, then changing storage vendors = changing a config file, not re-instrumenting 200 services. That's negotiating leverage and freedom.

**What you did:** lived it. Gadget ran three backends simultaneously during transitions (Axiom, Loki, ClickHouse) precisely because everything flowed through the pipeline layer — adding or dropping one was config. You also ran the collector's cleanup layer: transforms that rename outdated field names to current standards and parse raw values (user-agent strings, URLs) into structured fields, so imperfect instrumentation still lands as consistent data.

---

## SLOs (second gap — learn the vocabulary cold)

- **SLI** (indicator): a *measurement* of user-experienced quality. "% of trades that completed successfully within 2 seconds."
- **SLO** (objective): the *target*. "99.9% over 28 days."
- **Error budget:** the allowed failure that falls out of the target — 0.1% of a month ≈ 43 minutes. Its power is political: it converts "is this reliable enough?" from an endless argument into arithmetic. Budget left → ship features; budget spent → fix reliability.
- **Burn rate alerting:** don't page on raw error rate; page on *how fast the budget is burning*. Burning ~14× faster than sustainable for an hour → wake someone. Burning 2× for six hours → file a ticket. This "multi-window, multi-burn-rate" pattern (from Google's SRE books) kills both false pages and slow silent leaks.

**Your bridge:** you built latency metrics reliable enough that Kubernetes *autoscales on them automatically* — defining a signal trustworthy enough for a machine to act on is the same discipline as one trustworthy enough to page a human. And the platform angle: with wide events, an SLI is *just a query* — the team's job is making SLO definition a config file, not a project.

---

## The migration story (your safety credentials — fintech will care most about this)

How do you swap the log system every engineer relies on, without a bad day? Your playbook:

1. **Dual-write.** Send every log to *both* systems for weeks. New system fills with real data; nobody's using it yet. Zero risk.
2. **Shadow querying.** Every real user search secretly runs against *both* systems. Old system answers the user; a comparison is logged, with alarms if results differ by more than 20%. Real traffic finds the bugs synthetic tests never would (it did — subtle text-matching differences).
3. **Gradual flag rollout.** Flip customers to the new system a slice at a time via feature flag; any problem, flip back instantly.
4. **Staged teardown.** Only after 100% for weeks: remove old code, then old pipelines, then old servers, then old infrastructure definitions — each step separately reversible.

**The transferable principle:** never migrate a system on faith — make the old system the referee until the new one proves itself on real traffic. In a regulated fintech, this mindset is table stakes, and you've *executed* it, not just read about it.

---

## AI agents in production (their most unusual ask — your strongest differentiator)

The team wants AI agents (like Claude) to investigate incidents alongside humans. Your last year at Gadget was literally this. Three ingredients, plain terms:

1. **A door for the agent (MCP).** Model Context Protocol is a standard plug that lets an AI tool safely call real systems. You set up MCP servers and — the detail worth saying out loud — gave the agent **its own database account**: read-only, scoped permissions, its own resource limits so a runaway agent query gets killed like any other, and full attribution in the query logs. *Treat the agent as a user with an identity, quotas, and an audit trail.*
2. **Knowledge (skills).** Database access without context produces expensive garbage. You wrote a curated operations guide the agent loads — the schemas, the safe query patterns, the cost rules, an eight-step "how our best on-call engineer investigates" playbook — *with an automated test suite for the guide itself.* You gave the agent your judgment, not just your credentials.
3. **Speed (the foundation).** Agents investigate by rapid trial and error — ask, look, refine, repeat. That loop is useless against a 90-second query and transformative against a sub-second one. This closes the loop with everything above: the fast wide-event store isn't just for humans; it's what makes agent investigation *possible*. That's the job description's thesis, and you arrived at it independently.

---

## Quick glossary (skim before the interview)

| Term | Plain meaning |
|---|---|
| Wide event | One rich record per request/job with tons of fields, sliced at query time |
| Columnar store | Database storing data by column, not row — fast scans, huge compression |
| ClickHouse | The leading open-source columnar database; what you ran |
| Loki | Label-indexed log store you migrated away from |
| Vector | Pipeline agent that collects/transforms/ships logs; you ran ~80 of them |
| OTel collector | Standard pipeline service that receives, cleans, and routes telemetry |
| Semantic conventions | OTel's standard field names so all data speaks one language |
| Cardinality | Number of distinct values in a field; metrics choke on it, event stores don't |
| Sort key | Physical storage order of a table; the #1 lever for query speed |
| Skip index | Back-of-the-book index telling the database which chunks it can *not* read |
| Late materialization | Decide which rows win using tiny columns first; fetch heavy columns only for winners |
| Buffer table | ClickHouse staging layer absorbing many small writes into big efficient batches |
| MergeTree | ClickHouse's base engine: immutable sorted files, merged in the background |
| Summing/AggregatingMergeTree | Rollup engines that merge counts / math-in-progress on the same key |
| ReplacingMergeTree | Engine that eventually collapses duplicate rows — makes retried writes safe |
| Materialized view | Insert trigger that summarizes new data into rollup tables (or, "refreshable": a scheduled query) |
| Head/tail sampling | Keep-a-fraction decided at request start (cheap, blind) vs. after finish (smart, costly) |
| traceparent | The HTTP header carrying the trace ID so services can stitch one story |
| Span link | "Related but separate story" trace relationship — your websocket fix |
| SLI / SLO / error budget | Quality measurement / target / allowed failure that makes reliability arithmetic |
| Burn rate | How fast you're spending error budget; the right thing to page on |
| MCP | Standard plug letting AI agents safely call real systems |

## The 60-second self-summary (memorize the shape, not the words)

"For the last four years I've owned observability for a multi-tenant cloud platform — the same platform-as-product job this team does. I led the migration of our logging from Loki to a ClickHouse and OpenTelemetry stack: designed the schemas, built the ingestion pipelines, and did it safely — dual-writes, shadow comparison against the old system, gradual flag rollout — then tore the old stack out down to the Terraform. I've done the deep performance work on the store itself, like a query rewrite that took an out-of-memory failure to sub-second by deferring heavy column reads. And for the past year I've been building what this role calls 'AI agents in production': MCP access with scoped credentials, and curated operational knowledge that lets agents investigate incidents the way our best on-call does. I write Go and TypeScript daily, and Claude Code agentic workflows are how I ship."
