# Scott Côté

**Senior Software Engineer**
Ottawa, Ontario, Canada

- Email: scott.cote@hey.com
- LinkedIn: [www.linkedin.com/in/scott-rc](https://www.linkedin.com/in/scott-rc)

## Summary

Senior software engineer at Gadget, where I design and own core platform infrastructure. I build the systems the platform runs on: the Kubernetes control plane that scales and routes every tenant's compute, and the official CLI that is the local development experience for everyone building on it. The problems I keep coming back to are the hard ones in multi-tenant systems, like fairness, isolation, zero-downtime operation, and real-time sync, and I own them end to end from architecture to production. Outside of work I stay close to the languages and tooling I enjoy most: TypeScript, Go, and Rust.

## Experience

### Gadget — Senior Software Engineer

_March 2022 – Present (4 years 5 months) · Ottawa, ON_

- Respond to production incidents on the on-call rotation and turn root causes into permanent fixes across Postgres, Elasticsearch, ClickHouse, Temporal, and Kubernetes, including automated vacuum healing, WAL-slot recovery, and queue auto-repair sweeps.
- Designed and built Skipper, a Go Kubernetes orchestrator (router plus autoscaling controller) that replaced Fission for serving customer app processes. It uses gRPC for communication, hashring-based pod assignment, and zone-aware routing, with profile-guided optimization on the hot paths and an embedded web UI for live observability.
- Created and maintain ggt, Gadget's open-source developer CLI, along with the platform-side sync protocol it talks to. It handles bidirectional file sync with hash-based conflict detection, deploys, source control, and shell completions.
- Core maintainer of DateiLager, a Go and gRPC filesystem service. Shipped transport tuning and chunked streaming for large responses, enriched its OpenTelemetry instrumentation, built reproducible filesystem benchmarks (hardlinks, reflinks, LVM) to guide caching strategy, and drive its release and upgrade cadence across the platform.
- Built the real-time search indexing pipeline: a Go WAL listener on Postgres logical replication feeds batched Elasticsearch index workers, and Temporal-driven backfills have reindexed every model of every app on the platform.
- Drove the Loki to ClickHouse log platform migration. Designed the trace and platform-log schemas, standardized Lucene query syntax across log search, rewrote search as memory-bounded two-stage queries, validated the cutover with shadow query comparison, and right-sized the cluster before decommissioning Loki.
- Contributed to cloud cost and reliability work through node-pool architecture (tiered gold, silver, and bronze pools with backups for machine-type stockouts), topology-aware service routing, and workload placement and right-sizing improvements.

### Spoonity Inc. — Full Stack Developer

_November 2019 – March 2022 (2 years 5 months) · Ottawa, ON_

- Designed and built the backend for multi-channel marketing campaigns (email, SMS, and push notifications) using AWS Lambda, SQS, SES, and SNS, enabling clients to reach customers at scale across all channels.
- Introduced Terraform and AWS Lambda to the company's stack, establishing infrastructure-as-code practices and serverless patterns adopted for subsequent projects.
- Built an ETL pipeline syncing data from the production MySQL database to a dedicated analytics database using Lambda and SQS, decoupling reporting workloads from the production environment.
- Provisioned and managed all supporting cloud infrastructure with Terraform, making deployments reproducible and version-controlled.

### Employment and Social Development Canada — Programmer Analyst

_January 2017 – November 2019 (2 years 11 months) · Gatineau, QC_

- Full stack development using WET-BOEW, .NET MVC, WCF, and Oracle 12c.
- Led backend refactor to simplify code and improve performance.
- Participated in client meetings to resolve or establish top priorities.
- Created well documented tasks and issues for myself and others.
- Led multiple emergency deployments to production.
- Resident JavaScript expert.

## Education

**Algonquin College of Applied Arts and Technology**
Ontario College Diploma, Computer Programming · 2015 – 2017
