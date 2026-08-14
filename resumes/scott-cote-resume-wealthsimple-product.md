# Scott Côté

**Senior Software Engineer — Full Stack & AI-Native Development**
Ottawa, Ontario, Canada

- Email: scott.cote@hey.com
- LinkedIn: [www.linkedin.com/in/scott-rc](https://www.linkedin.com/in/scott-rc)

## Summary

Senior engineer at Gadget who owns meaningful slices of the product end to end, from backend systems to the surfaces people touch. I created and maintain the platform's open-source developer CLI, designed and built the Kubernetes control plane that runs every customer app, and ship the internal tooling operators rely on daily — and I carry the pager for what I ship. The through-line of my work is turning complex platform problems into experiences that feel simple to the person using them. I work AI-natively: genuinely agentic Claude Code workflows drive my day-to-day building, testing, and reviewing, and I write the custom skills, rules, and repo conventions that make those workflows rigorous for the whole team. My background is TypeScript and Go rather than Ruby, and I ramp quickly on new stacks.

## Skills

- **Languages**: TypeScript/Node.js, Go, Rust, SQL, Bash
- **AI tooling**: agentic coding workflows (Claude Code — custom skills, rules, hooks), LLM-oriented documentation
- **Backend & data**: PostgreSQL (logical replication/CDC), Temporal, Google Pub/Sub, Elasticsearch, ClickHouse, Kubernetes, gRPC, Redis, MySQL
- **Frontend**: React, Tailwind CSS, server-driven declarative UI (Datastar/SSE)
- **Cloud & delivery**: GCP, AWS (Lambda, SQS, SES, SNS), Terraform, Buildkite, GitHub Actions, OpenTelemetry observability

## Experience

### Gadget — Senior Software Engineer

_March 2022 – Present (4 years 5 months) · Ottawa, ON_

- Created and maintain ggt, Gadget's open-source developer CLI and the local development experience for everyone building on the platform: bidirectional file sync with hash-based conflict detection, deploys, source control, shell completions, and friendly error output, with fully automated release pipelines.
- Designed and built Skipper, a Go Kubernetes orchestrator (request router plus autoscaling controller) that serves every customer app process on the platform, and took it from empty repo to fully replacing the incumbent FaaS system in production in under three months.
- Built internal product surfaces end to end: background-actions admin pages with one-click operational workflows, queue inspection views bounded to stay fast on large tenants, and a Temporal-driven orphaned-file cleanup workflow with admin buttons to drive it.
- Built the real-time event pipeline behind platform search: a Go WAL listener on Postgres logical replication publishes ordered changes to Pub/Sub, and batched index workers use optimistic versioning to keep Elasticsearch consistent with the source of truth; Temporal-driven backfills have reindexed every model of every app on the platform.
- Establish the patterns that make agentic coding work for the whole team: authored and iterated a ClickHouse operations skill for coding agents in the monorepo, set up Claude Code configuration with path-scoped rule files, and restructured internal operational docs for LLM consumption.
- Carry the pager and turn incidents into permanent automated fixes across Postgres, Elasticsearch, ClickHouse, Temporal, and Kubernetes: automated vacuum healing, WAL replication-slot recovery with reindexing, and scheduled queue auto-repair sweeps.
- Led the Loki-to-ClickHouse log platform migration and validated the cutover with shadow query comparison against the old backend before decommissioning it; document what I build in runbooks, an observability guide, and a full docs site for the orchestrator.

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
