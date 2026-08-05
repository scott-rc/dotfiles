# Scott Côté

**Senior Software Engineer — Full Stack, Internal Tooling & Workflow Automation**
Ottawa, Ontario, Canada

- Email: scott.cote@hey.com
- LinkedIn: [www.linkedin.com/in/scott-rc](https://www.linkedin.com/in/scott-rc)

## Summary

Senior engineer at Gadget who finds the workflows where manual effort compounds into operational risk and ships the automation that makes them disappear. For 4+ years I've owned systems end to end on a multi-tenant cloud platform — architecture, code, deploys, dashboards, and the pager — without a PM or ticket backlog scoping the work: I diagnose the friction, decide what's worth building, and close the loop myself. Recent work spans internal admin tooling, streaming data pipelines, and the on-call automation that turned our most common incident classes into non-events. I work AI-natively: agentic coding workflows drive my day-to-day output, and I build the scaffolding (custom skills, rules, and repo conventions for coding agents) that makes them productive for the whole team.

## Skills

- **Languages**: TypeScript/Node.js, Go, Rust, SQL, Bash
- **AI tooling**: agentic coding workflows (Claude Code — custom skills, rules, hooks), LLM-oriented documentation
- **Data & backend**: PostgreSQL (logical replication/CDC), ClickHouse, Elasticsearch, Temporal, Google Pub/Sub, Kubernetes, gRPC, Redis, MySQL
- **Frontend**: React, Tailwind CSS, server-driven declarative UI (Datastar/SSE)
- **Cloud & delivery**: GCP, AWS (Lambda, SQS, SES, SNS), Terraform, Buildkite, GitHub Actions, OpenTelemetry observability

## Experience

### Gadget — Senior Software Engineer

_March 2022 – Present (4 years 5 months) · Ottawa, ON_

- Make recurring manual workflows disappear: turned the on-call rotation's most common incident classes into scheduled, self-healing automation — Postgres vacuum healing, WAL replication-slot recovery with reindexing, queue auto-repair sweeps — plus an emergency feature-flag fallback that kept flags serving through a LaunchDarkly outage.
- Built internal operational tooling end to end: background-actions admin pages with one-click operational workflows, queue inspection views bounded to stay fast on large tenants, and a Temporal-driven orphaned-file cleanup workflow with admin buttons to drive it.
- Built the real-time CDC pipeline behind platform search: a Go WAL listener on Postgres logical replication publishes ordered changes to Pub/Sub, and batched index workers with optimistic versioning keep Elasticsearch consistent with the source of truth; Temporal-driven backfills have reindexed every model of every app on the platform.
- Build the scaffolding that makes agentic coding work for the team: authored and iterated a ClickHouse operations skill for coding agents in the monorepo, set up Claude Code configuration with path-scoped rule files, and restructured internal operational docs for LLM consumption.
- Took the platform's compute layer from empty repo to fully replacing the incumbent FaaS system in production in under three months: designed and built Skipper, a Go Kubernetes orchestrator (request router plus autoscaling controller) that serves every customer app process on the platform.
- Led the Loki-to-ClickHouse log platform migration, validating the cutover with shadow query comparison against the old backend before decommissioning it, and rewrote log search as memory-bounded queries along the way.
- Created and maintain ggt, Gadget's open-source developer CLI, and the platform-side sync protocol it talks to; document what I build in runbooks, an observability guide, and a full docs site for the orchestrator.

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
