# Scott Côté

**Senior Infrastructure Engineer**
Ottawa, Ontario, Canada

- Email: scott.cote@hey.com
- LinkedIn: [www.linkedin.com/in/scott-rc](https://www.linkedin.com/in/scott-rc)

## Summary

Senior infrastructure engineer at Gadget. I build and operate the streaming and data infrastructure behind a multi-tenant app platform: a CDC pipeline that feeds Postgres changes into Elasticsearch, the ClickHouse log platform and its ingestion pipeline, and the Go Kubernetes control plane that runs every tenant's compute. I own my systems from design through production and carry the pager for them. Most comfortable in Go, TypeScript, and Rust.

## Experience

### Gadget — Senior Software Engineer

_March 2022 – Present (4 years 5 months) · Ottawa, ON_

- Built Gadget's CDC pipeline for real-time search. A Go WAL listener on Postgres logical replication publishes ordered change events to Google Pub/Sub, and index workers I wrote batch and dedupe them into Elasticsearch. Optimistic versioning keeps out-of-order events consistent, poison messages land in a dead-letter topic, and Temporal-driven backfills have reindexed every model of every app on the platform.
- Maintain Gadget's fork of wal-listener, an open-source Go CDC connector, as its top contributor with 12 merged PRs. Fixed replication correctness bugs, including unchanged TOAST values dropped from update events, crashes on null `restart_lsn`, and LSN acks firing before the downstream publish, and shepherded the fork through the pgx v5 upgrade.
- Debugged the pipeline under production load: paused Pub/Sub ordering keys, oversized-message publish failures, silent data corruption from partial updates nulling unchanged TOAST values (remediated with a platform-wide reindex), and a dropped replication slot that meant finding and reindexing every affected environment.
- Drove the Loki to ClickHouse log platform migration. Designed the trace and platform-log schemas and the Vector/OpenTelemetry ingestion pipeline, rewrote log search as memory-bounded two-stage queries behind a standardized Lucene syntax, validated the cutover with shadow query comparison, and right-sized the cluster before decommissioning Loki.
- Operate and harden silo, Gadget's sharded background-job store built on object storage. Fixed silent data loss in shard splits, built a gameday that verifies data integrity through a live split, cut time-to-leasable latency from seconds to under 100 ms in benchmarks by streaming scanner commits in chunks, and set up worker autoscaling on a leasable-to-start latency signal.
- Designed and built Skipper, a Go Kubernetes orchestrator (router plus autoscaling controller) that replaced Fission for serving customer app processes. It uses gRPC for communication, hashring-based pod assignment, and zone-aware routing, with profile-guided optimization and allocation tuning keeping controller GC off the hot path.
- Created and maintain ggt, Gadget's open-source developer CLI, along with the platform-side sync protocol it talks to. It handles bidirectional file sync with hash-based conflict detection, deploys, and source control, and I work directly with the developers who use it every day.
- Respond to production incidents on the on-call rotation and turn root causes into permanent fixes across Postgres, Elasticsearch, ClickHouse, Temporal, and Kubernetes, including automated vacuum healing, WAL-slot recovery, and queue auto-repair sweeps. Contributed tiered node pools with backups that survive machine-type stockouts, along with workload placement and right-sizing improvements.

### Spoonity Inc. — Full Stack Developer

_November 2019 – March 2022 (2 years 5 months) · Ottawa, ON_

- Designed and built the backend for multi-channel marketing campaigns (email, SMS, and push notifications) using AWS Lambda, SQS, SES, and SNS, enabling clients to reach customers at scale across all channels.
- Built an ETL pipeline syncing data from the production MySQL database to a dedicated analytics database using Lambda and SQS, decoupling reporting workloads from the production environment.
- Introduced Terraform and AWS Lambda to the company's stack, establishing infrastructure-as-code practices and serverless patterns adopted for subsequent projects.

### Employment and Social Development Canada — Programmer Analyst

_January 2017 – November 2019 (2 years 11 months) · Gatineau, QC_

- Full stack development using WET-BOEW, .NET MVC, WCF, and Oracle 12c.
- Led backend refactor to simplify code and improve performance.
- Led multiple emergency deployments to production.

## Education

**Algonquin College of Applied Arts and Technology**
Ontario College Diploma, Computer Programming · 2015 – 2017
