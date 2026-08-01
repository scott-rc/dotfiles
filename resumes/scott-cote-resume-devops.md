# Scott Côté

**Senior Software Engineer — Platform & Infrastructure**
Ottawa, Ontario, Canada

- Email: scott.cote@hey.com
- LinkedIn: [www.linkedin.com/in/scott-rc](https://www.linkedin.com/in/scott-rc)

## Summary

Senior engineer who builds and operates the infrastructure that platforms run on. For the past 4+ years I've owned production infrastructure at Gadget, a multi-tenant cloud platform: Kubernetes orchestration and autoscaling, Terraform, CI/CD pipelines, and an observability stack built on OpenTelemetry, ClickHouse, and Grafana. I carry the pager, turn incidents into permanent automated fixes, and write down what I build in runbooks and guides. Before Gadget I introduced Terraform and serverless AWS patterns at Spoonity. Comfortable in TypeScript, Go, Rust, and Bash.

## Skills

- **Cloud**: GCP (GKE, AlloyDB, GCS), AWS (Lambda, SQS, SES, SNS)
- **Infrastructure as code**: Terraform, Kubernetes manifests (krane/ERB templating), Helm, Nix
- **CI/CD**: GitHub Actions, Buildkite, automated release pipelines (changesets, GoReleaser)
- **Observability**: OpenTelemetry, Grafana, ClickHouse, Sentry, alerting and SLO dashboards
- **Databases**: PostgreSQL/AlloyDB, ClickHouse, Redis, Elasticsearch, MySQL
- **Languages**: TypeScript/Node.js, Go, Rust, Bash, SQL

## Experience

### Gadget — Senior Software Engineer

_March 2022 – Present (4 years 5 months) · Ottawa, ON_

- Manage production infrastructure as code with Terraform and Kubernetes manifests, with 700+ commits to the infrastructure repo. Designed the tiered node-pool architecture and its automatic backup pools that keep workloads scheduling through cloud machine-type stockouts, brought an AlloyDB shard under Terraform management with HA regional failover and a read replica, and right-sized the production ClickHouse cluster from 120 GB to 64 GB per replica by tuning per-query limits against production query logs.
- Designed and built Skipper, a Go Kubernetes orchestrator (router plus autoscaling controller) that serves every customer app process on the platform. It uses gRPC for communication, hashring-based pod assignment, and zone-aware routing, and ships an embedded web UI for live cluster observability.
- Build and maintain CI/CD across the organization: Buildkite for the monorepo and GitHub Actions for the open-source projects. That covers parallelism tuning, flake retries, Kubernetes-backed test infrastructure, fully automated release pipelines, Dependabot auto-merge, and security-alert remediation.
- Migrated platform logging and log search from Loki to a ClickHouse, OpenTelemetry, and Grafana stack. Designed the trace and platform-log schemas and the collector pipelines, rewrote search as memory-bounded queries, validated the cutover with shadow query comparison, built the monitoring dashboards and tuned alerting, then decommissioned Loki down to its Helm releases and Terraform resources.
- Respond to production incidents on the on-call rotation and turn root causes into permanent fixes: automated Postgres vacuum healing, WAL replication-pipeline fixes with recovery reindexing, queue auto-repair sweeps, and a cutover to a Redis-backed flag store that kept feature flags serving through a LaunchDarkly outage.
- Drive autoscaling and capacity work. Built HPAs on custom latency metrics to scale queue workers on time-to-start, improved workload placement and topology-aware scheduling, and right-sized memory to stop recurring OOM kills across API shards and background workers.
- Created and maintain ggt, Gadget's open-source CLI for file sync, deploys, and shell completions, and contribute to the Nix and direnv development environment used by the whole engineering team.
- Document infrastructure for the long term. Wrote the Skipper operational runbook, expanded the ClickHouse runbook, rewrote the Grafana README as an observability guide, and built a full docs site for the orchestrator.

### Spoonity Inc. — Full Stack Developer

_November 2019 – March 2022 (2 years 5 months) · Ottawa, ON_

- Introduced Terraform and AWS Lambda to the company's stack, establishing infrastructure-as-code and serverless patterns adopted for all subsequent projects.
- Provisioned and managed all supporting cloud infrastructure with Terraform, making deployments reproducible and version-controlled.
- Designed and built the backend for multi-channel marketing campaigns (email, SMS, push) on AWS Lambda, SQS, SES, and SNS, reaching customers at scale across all channels.
- Built an ETL pipeline syncing the production MySQL database to a dedicated analytics database using Lambda and SQS, decoupling reporting workloads from production.

### Employment and Social Development Canada — Programmer Analyst

_January 2017 – November 2019 (2 years 11 months) · Gatineau, QC_

- Full stack development using WET-BOEW, .NET MVC, WCF, and Oracle 12c.
- Led a backend refactor to simplify code and improve performance.
- Led multiple emergency deployments to production.

## Education

**Algonquin College of Applied Arts and Technology**
Ontario College Diploma, Computer Programming · 2015 – 2017
