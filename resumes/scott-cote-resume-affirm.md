# Scott Côté

**Senior Software Engineer — Full Stack & AI-Native Development**
Ottawa, Ontario, Canada

- Email: scott.cote@hey.com
- LinkedIn: [www.linkedin.com/in/scott-rc](https://www.linkedin.com/in/scott-rc)

## Summary

Senior engineer at Gadget who takes ambiguous, high-stakes systems from empty repo to production. I designed and built the Kubernetes control plane that serves every app on a multi-tenant cloud platform, created its open-source developer CLI, and own the systems I ship end to end — architecture, code, deploys, dashboards, and the pager. I work AI-natively: agentic coding workflows drive my day-to-day output, and I build the scaffolding (custom skills, rules, and repo conventions for coding agents) that makes those workflows productive for the whole team. Comfortable across the stack, from gRPC backends and streaming data pipelines to the web UIs and CLIs people actually touch.

## Skills

- **Languages**: TypeScript/Node.js, Go, Rust, SQL, Bash
- **AI tooling**: agentic coding workflows (Claude Code — custom skills, rules, hooks), LLM-oriented documentation
- **Backend**: Kubernetes, gRPC, PostgreSQL, ClickHouse, Elasticsearch, Redis, Temporal, Google Pub/Sub, MySQL
- **Frontend**: React, Tailwind CSS, server-driven declarative UI (Datastar/SSE)
- **Cloud & delivery**: GCP, AWS (Lambda, SQS, SES, SNS), Terraform, Buildkite, GitHub Actions, OpenTelemetry observability

## Experience

### Gadget — Senior Software Engineer

_March 2022 – Present (4 years 5 months) · Ottawa, ON_

- Took the platform's compute layer from empty repo to fully replacing the incumbent FaaS system in production in under three months: designed and built Skipper, a Go Kubernetes orchestrator (request router plus autoscaling controller) that serves every customer app process on the platform. Primary author with 500+ of its ~535 commits.
- Build the scaffolding that makes agentic coding work for the team: authored and iterated a ClickHouse operations skill for coding agents in the monorepo (7 PRs), set up Claude Code configuration with path-scoped rule files in the CLI repo, and restructured internal operational docs for LLM consumption.
- Created and maintain ggt, Gadget's open-source developer CLI, and the platform-side sync protocol it talks to: bidirectional file sync with hash-based conflict detection, deploys, a declarative command-definition API, shell completions, and friendly error output, with fully automated release pipelines.
- Built internal product surfaces end to end: the background-actions admin pages with one-click operational workflows, queue inspection views bounded to stay fast on large tenants, and a Temporal-driven orphaned-file cleanup workflow with admin buttons to drive it.
- Built the real-time CDC pipeline behind platform search: a Go WAL listener on Postgres logical replication publishes ordered changes to Pub/Sub, and batched index workers with optimistic versioning keep Elasticsearch consistent; Temporal-driven backfills have reindexed every model of every app on the platform.
- Carry the pager and turn incidents into permanent automated fixes — Postgres vacuum healing, WAL replication-slot recovery, queue auto-repair sweeps — and led the Loki-to-ClickHouse log platform migration, validating the cutover with shadow query comparison before decommissioning the old stack.

### Spoonity Inc. — Full Stack Developer

_November 2019 – March 2022 (2 years 5 months) · Ottawa, ON_

- Designed and built the backend for multi-channel marketing campaigns (email, SMS, push) on AWS Lambda, SQS, SES, and SNS, reaching customers at scale across all channels.
- Built an ETL pipeline syncing the production MySQL database to a dedicated analytics database using Lambda and SQS, decoupling reporting workloads from production.
- Introduced Terraform and AWS Lambda to the company's stack, establishing infrastructure-as-code and serverless patterns adopted for all subsequent projects.

### Employment and Social Development Canada — Programmer Analyst

_January 2017 – November 2019 (2 years 11 months) · Gatineau, QC_

- Full stack development using WET-BOEW, .NET MVC, WCF, and Oracle 12c.
- Led a backend refactor to simplify code and improve performance.
- Led multiple emergency deployments to production.

## Education

**Algonquin College of Applied Arts and Technology**
Ontario College Diploma, Computer Programming · 2015 – 2017
