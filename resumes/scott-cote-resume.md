# Scott Côté

**Senior Software Engineer**
Ottawa, Ontario, Canada

- Email: scott.cote@hey.com
- LinkedIn: [www.linkedin.com/in/scott-rc](https://www.linkedin.com/in/scott-rc)

## Summary

Senior software engineer at Gadget, where I design and own core platform infrastructure. I build the systems the platform runs on — the Kubernetes control plane that scales and routes every tenant's compute, and the official CLI that is the local development experience for everyone building on it. I'm drawn to the hard problems in multi-tenant systems: fairness, isolation, zero-downtime operation, and real-time sync, owned end to end from architecture to production. Outside of work I stay close to the languages and tooling I enjoy most — TypeScript, Go, and Rust.

## Experience

### Gadget — Senior Software Engineer

_March 2022 – Present (4 years 5 months) · Ottawa, ON_

- Respond to production incidents as part of the on-call rotation and turn root causes into permanent fixes (automated vacuum healing, WAL-slot recovery, queue auto-repair sweeps) across Postgres, Elasticsearch, ClickHouse, Temporal, and Kubernetes.
- Designed and built Skipper, a Go-based Kubernetes orchestrator (router + autoscaling controller) that replaced Fission for serving customer app processes, featuring gRPC/Protobuf communication, hashring-based pod assignment, zone-aware routing, profile-guided optimization of hot paths, and an embedded live-observability web UI.
- Created and maintain ggt, Gadget's open-source developer CLI, from the ground up (a bidirectional file-sync engine with hashing-based conflict detection, deploys, source control, and shell completions), along with the platform-side sync protocol it talks to.
- Act as a core maintainer of DateiLager (Go/gRPC filesystem service): shipped transport tuning and chunked large-response streaming, enriched its OpenTelemetry instrumentation, built reproducible filesystem benchmarks (hardlinks, reflinks, LVM) to guide caching strategy, and drove its release-and-upgrade cadence across the platform.
- Implemented real-time search indexing pipeline: a Go WAL listener on Postgres logical replication feeding batched Elasticsearch index workers, with Temporal-driven backfills that reindexed every model of every app on the platform.
- Migrated platform observability from Loki to ClickHouse: designed the log/trace schemas, added Lucene query syntax to the log viewer, rewrote search as memory-bounded two-stage queries, validated the cutover with shadow query comparison, and right-sized the cluster before decommissioning Loki entirely.
- Contributed to cloud cost and reliability initiatives through node-pool architecture (tiered gold/silver/bronze pools, backup pools for machine-type stockouts), topology-aware service routing, and workload bin-packing improvements.

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
