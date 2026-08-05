# Scott Côté

**Senior Software Engineer — Backend & Distributed Systems**
Ottawa, Ontario, Canada

- Email: scott.cote@hey.com
- LinkedIn: [www.linkedin.com/in/scott-rc](https://www.linkedin.com/in/scott-rc)

## Summary

Senior backend engineer who designs and operates the distributed systems other teams trust to be correct. For 4+ years at Gadget I've owned core platform services end to end, from technical design through production on-call: the Go Kubernetes control plane that routes and scales every tenant's compute, a real-time change-data-capture pipeline built on Postgres logical replication, and the gRPC filesystem service behind every app's code and assets. The problems I keep coming back to are correctness under concurrency, low latency, and staying available through peak load and partial failure. I work primarily in Go and TypeScript and ramp quickly on new stacks, and I build the scaffolding (custom skills, rules, and repo conventions for coding agents) that makes AI-assisted engineering rigorous for the whole team.

## Skills

- **Languages**: Go, TypeScript/Node.js, Rust, SQL, Bash
- **Distributed systems**: gRPC, Kubernetes, consistent-hashring sharding, zone/topology-aware routing, zero-downtime operation
- **Event streaming & data**: Postgres logical replication (CDC), Google Pub/Sub, Temporal, Elasticsearch, ClickHouse, Redis, MySQL
- **Reliability & observability**: OpenTelemetry, Grafana, Prometheus metrics, incident response with automated remediation, SLO dashboards
- **AI tooling**: agentic coding workflows (Claude Code — custom skills, rules, hooks), LLM-oriented documentation
- **Cloud & delivery**: GCP, AWS, Terraform, Buildkite, GitHub Actions

## Experience

### Gadget — Senior Software Engineer

_March 2022 – Present (4 years 5 months) · Ottawa, ON_

- Designed and built Skipper, a Go Kubernetes orchestrator (request router plus autoscaling controller) that serves every customer app process on the platform: gRPC communication, consistent-hashring sharding of workloads across controller replicas, zone-aware service discovery, and profile-guided optimization keeping the hot paths cheap. Took it from empty repo to fully replacing the incumbent FaaS system in production in under three months.
- Keep the platform dependable through peak load and partial failure: tiered node-pool architecture with automatic backup pools that keep workloads scheduling through cloud machine-type stockouts, topology-aware service routing, and an emergency feature-flag fallback that kept flags serving through a LaunchDarkly outage.
- Built the real-time event pipeline behind platform search: a Go WAL listener on Postgres logical replication publishes ordered changes to Pub/Sub, and batched index workers use optimistic versioning to keep Elasticsearch consistent with the source of truth; Temporal-driven backfills have reindexed every model of every app on the platform.
- Carry the pager and turn incidents into permanent automated fixes across Postgres, Elasticsearch, ClickHouse, Temporal, and Kubernetes: automated vacuum healing, WAL replication-slot recovery with reindexing, and scheduled queue auto-repair sweeps.
- Core maintainer of DateiLager, the Go and gRPC filesystem service behind every app's code and assets: shipped transport tuning (keepalive, flow-control windows, message sizing) and chunked streaming for large responses, enriched its OpenTelemetry instrumentation, and built reproducible filesystem benchmarks to guide caching strategy.
- Led the Loki-to-ClickHouse log platform migration and validated the cutover with shadow query comparison against the old backend before decommissioning it; rewrote log search as memory-bounded two-stage queries and right-sized the cluster against production query logs.
- Build the scaffolding that makes AI-assisted engineering work for the team: authored and iterated a ClickHouse operations skill for coding agents in the monorepo, set up Claude Code configuration with path-scoped rule files, and restructured internal operational docs for LLM consumption.
- Created and maintain ggt, Gadget's open-source developer CLI, and the platform-side sync protocol it talks to (bidirectional file sync with hash-based conflict detection); document what I build in runbooks, an observability guide, and a full docs site for the orchestrator.

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
