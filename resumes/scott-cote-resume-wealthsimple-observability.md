# Scott Côté

**Senior Software Engineer — Observability & Platform Engineering**
Ottawa, Ontario, Canada

- Email: scott.cote@hey.com
- LinkedIn: [www.linkedin.com/in/scott-rc](https://www.linkedin.com/in/scott-rc)

## Summary

Senior engineer who builds platforms as products, with other engineers as the customers. For 4+ years at Gadget I've owned the observability stack of a multi-tenant cloud platform end to end: I led the migration to a ClickHouse, OpenTelemetry, and Grafana stack, designed the trace and log schemas, and operate the cluster, dashboards-as-code, and alerting that every engineer uses to investigate production. Alongside it I build the developer-facing tooling — an open-source CLI, gRPC services, a Kubernetes control plane — and write the runbooks, guides, and docs sites that make those systems legible. I write production code daily in Go and TypeScript (plus Rust for my own tooling), and I work AI-natively: Claude Code agentic workflows drive my day-to-day output, and I build the custom skills and rules that make coding agents effective against production systems.

## Skills

- **Languages**: Go, TypeScript/Node.js, Rust, SQL, Bash
- **Observability**: OpenTelemetry instrumentation and collector pipelines, ClickHouse, Grafana dashboards-as-code, custom Prometheus metrics, Sentry, alerting and SLO dashboards
- **Data & streaming**: Postgres logical replication (CDC), Google Pub/Sub, Elasticsearch, Temporal, Redis, MySQL
- **Platform**: Kubernetes, gRPC, Terraform, GCP, AWS
- **AI tooling**: agentic coding workflows (Claude Code — custom skills, rules, hooks), LLM-oriented documentation
- **Delivery**: Buildkite, GitHub Actions, fully automated release pipelines

## Experience

### Gadget — Senior Software Engineer

_March 2022 – Present (4 years 5 months) · Ottawa, ON_

- Led the migration of platform logging and log search from Loki to a ClickHouse, OpenTelemetry, and Grafana stack: designed the trace schema and collector pipelines, co-designed the log schemas, standardized Lucene query syntax across log search, validated the cutover with shadow query comparison against the old backend, then decommissioned Loki down to its Helm releases and Terraform resources.
- Own the observability platform beyond the migration: Grafana dashboards deployed from source-controlled YAML with alert rules version-controlled alongside them, custom Prometheus metrics served through the prometheus-adapter to drive autoscaling, and rewrote log search as memory-bounded two-stage queries while right-sizing the production ClickHouse cluster from 120 GB to 64 GB per replica by tuning per-query limits against production query logs.
- Built a real-time, high-throughput ingestion pipeline: a Go WAL listener on Postgres logical replication publishes ordered changes to Pub/Sub, and batched index workers with optimistic versioning keep Elasticsearch consistent with the source of truth; Temporal-driven backfills have reindexed every model of every app on the platform.
- Make production systems legible to coding agents: authored and iterated a ClickHouse operations skill that equips agents to query and operate the observability store, set up Claude Code configuration with path-scoped rule files, and restructured internal operational docs for LLM consumption.
- Designed and built Skipper, a Go Kubernetes orchestrator (request router plus autoscaling controller) that serves every customer app process on the platform: gRPC communication, consistent-hashring sharding across controller replicas, zone-aware service discovery, and profile-guided optimization keeping the hot paths cheap.
- Core maintainer of DateiLager, the Go and gRPC filesystem service behind every app's code and assets: shipped transport tuning (keepalive, flow-control windows, message sizing) and chunked streaming for large responses, enriched its OpenTelemetry instrumentation, and built reproducible filesystem benchmarks to guide caching strategy.
- Carry the pager and turn incidents into permanent automated fixes across Postgres, Elasticsearch, ClickHouse, Temporal, and Kubernetes: automated vacuum healing, WAL replication-slot recovery with reindexing, and scheduled queue auto-repair sweeps.
- Created and maintain ggt, Gadget's open-source developer CLI, and the platform-side sync protocol it talks to — the golden path for local development on the platform; document what I build in runbooks, an observability guide, and a full docs site for the orchestrator.

### Spoonity Inc. — Full Stack Developer

_November 2019 – March 2022 (2 years 5 months) · Ottawa, ON_

- Designed and built the backend for multi-channel marketing campaigns (email, SMS, push) on AWS Lambda, SQS, SES, and SNS, reaching customers at scale across all channels.
- Built an ETL pipeline syncing the production MySQL database to a dedicated analytics database using Lambda and SQS, decoupling reporting workloads from production.
- Introduced Terraform and AWS Lambda to the company's stack, establishing infrastructure-as-code and serverless patterns adopted for all subsequent projects.

### Employment and Social Development Canada — Programmer Analyst

_January 2017 – November 2019 (2 years 11 months) · Gatineau, QC_

- Full stack development using WET-BOEW, .NET MVC, WCF, and Oracle 12c.
- Participated in client meetings with non-technical stakeholders to resolve issues and establish priorities.
- Led multiple emergency deployments to production.

## Education

**Algonquin College of Applied Arts and Technology**
Ontario College Diploma, Computer Programming · 2015 – 2017
