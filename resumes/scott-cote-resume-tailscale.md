# Scott Côté

**Senior Software Engineer — Infrastructure & Platform**
Ottawa, Ontario, Canada

- Email: scott.cote@hey.com
- LinkedIn: [www.linkedin.com/in/scott-rc](https://www.linkedin.com/in/scott-rc)

## Summary

Senior engineer who builds and operates the shared infrastructure other engineers rely on. For the past 4+ years I've owned production infrastructure at Gadget, a multi-tenant cloud platform: Terraform-managed GCP resources, Kubernetes orchestration, CI/CD pipelines, and an observability stack built on OpenTelemetry, ClickHouse, and Grafana. I carry the pager, turn incidents into permanent automated fixes, and write down what I build in runbooks and docs sites. Much of my recent work is in Go, including a Kubernetes control plane I designed and operate in production. Comfortable working asynchronously and owning projects end to end.

## Skills

- **Languages**: Go, TypeScript/Node.js, Rust, Bash, SQL
- **Infrastructure as code**: Terraform, Kubernetes manifests (krane/ERB templating), Helm, Nix
- **CI/CD**: Buildkite, GitHub Actions, automated release pipelines (changesets, GoReleaser), keyless cloud auth via OIDC workload identity federation
- **Observability**: OpenTelemetry, Grafana, ClickHouse, Prometheus metrics, Sentry, alerting and SLO dashboards
- **Cloud**: GCP (GKE, AlloyDB, GCS, Pub/Sub), AWS (Lambda, SQS, SES, SNS)
- **Networking**: gRPC transport tuning, zone/topology-aware routing, ingress-nginx, NetworkPolicy isolation

## Experience

### Gadget — Senior Software Engineer

_March 2022 – Present (4 years 5 months) · Ottawa, ON_

- Manage production infrastructure as code with Terraform and Kubernetes manifests, with 700+ commits to the infrastructure repo. Imported a hand-created AlloyDB Postgres cluster under Terraform management, designed the tiered node-pool architecture with automatic backup pools that keep workloads scheduling through cloud machine-type stockouts, and set up keyless GitHub Actions authentication to GCP through an OIDC workload identity federation pool.
- Led the migration of platform logging and log search from Loki to a ClickHouse, OpenTelemetry, and Grafana stack. Designed the trace schema and collector pipelines and co-designed the log schemas, rewrote log search as memory-bounded queries, validated the cutover with shadow query comparison against the old backend, then decommissioned Loki down to its Helm releases and Terraform resources.
- Own observability tooling beyond the migration: Grafana dashboards deployed from source-controlled YAML with alert rules version-controlled alongside them, custom Prometheus metrics served through the prometheus-adapter to drive autoscaling, and right-sizing the production ClickHouse cluster from 120 GB to 64 GB per replica by tuning per-query limits against production query logs.
- Own CI/CD for the open-source projects (GitHub Actions with fully automated changesets-based npm release pipelines and multi-arch Docker image builds) and contribute heavily to the monorepo's Buildkite pipelines: parallelism and test-splitting tuning, flake retries, Kubernetes-backed test infrastructure, and a fix to the CI agent autoscaler that was double-counting instances and running CI at half capacity.
- Designed and built Skipper, a Go Kubernetes orchestrator (router plus autoscaling controller) that serves every customer app process on the platform. It uses gRPC for communication, consistent-hashring sharding of workloads across controller replicas, and zone-aware service discovery between routers and controllers, with profile-guided optimization keeping the hot paths cheap and an embedded web UI for live cluster observability.
- Respond to production incidents on the on-call rotation and turn root causes into permanent fixes: automated Postgres vacuum healing, WAL replication-slot recovery with reindexing, queue auto-repair sweeps that run on a schedule, and an emergency feature-flag fallback that kept flags serving through a LaunchDarkly outage.
- Drive upgrade work across platform services: shepherded the Fission FaaS platform through nine version upgrades from 1.17 to 1.20, migrated the observability stack off a deprecated Helm chart to ClickStack, moved workloads onto new machine-family node pools, and led Go, pgx, and toolchain upgrades across the Go services I maintain.
- Work close to the network layer: topology-aware service routing (`trafficDistribution: PreferClose`) across pgbouncer, rate limiting, the request router, and telemetry collectors; a zone-aware gRPC resolver for router-to-controller traffic; NetworkPolicy isolation for untrusted sandbox workloads; and gRPC keepalive, flow-control window, and message-size tuning on a high-throughput file service.
- Build the tooling other engineers use every day: ggt, Gadget's open-source developer CLI, and internal admin tools for queue inspection and orphaned-storage cleanup; maintain and extend the Nix and direnv development environment shared by the whole team. Document it all in runbooks, an observability guide, and a full docs site for the orchestrator.

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
