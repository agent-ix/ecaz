# Artifact Manifest — SPIRE AWS pass-correctness (Graviton 4)

Packet: `reviews/task-30/958-spire-aws-pass-correctness/`
Owner: coder B (SPIRE AWS lane)
Branch: `task-30-phase13-spire-aws-prep`
Follow-on to: `reviews/task-30/957-spire-aws-prep-local-verification/`

## AWS Topology Identity (run 2 — downsized after quota hit)

| Field            | Value                                                 |
| ---------------- | ----------------------------------------------------- |
| AWS account      | `932658697181`                                        |
| IAM principal    | `arn:aws:iam::932658697181:user/ecaz-operator`        |
| Region           | `us-west-2`                                           |
| AZ               | `us-west-2a`                                          |
| AMI              | `ami-04e0d7d889f694536` (AL2023 arm64, 2026-05-15)    |
| Coordinator type | `r8g.xlarge` (Graviton 4 / Neoverse V2, 4 vCPU/32 GB) |
| Remote type      | `r8g.xlarge` × 2 (Graviton 4 / Neoverse V2)           |
| Total vCPU       | 12 (under the 16-vCPU r-family account limit)         |
| Head SHA         | `9cfe0bcd` (task-30-phase13-spire-aws-prep)           |
| Cost tag owner   | `kreneskyp`                                           |
| Cost tag deadline| `2026-05-24T00:00:00Z`                                |
| VPC ID           | `vpc-0a2f98585cd2fa2c7`                               |
| Subnet ID        | `subnet-0bbc149851847e770`                            |
| S3 bucket        | `ecaz-spire-aws-20260523165108075000000003`           |
| Coordinator id   | `i-068f303289c8e500d` @ `10.42.1.196`                 |
| Remote 1 id      | `i-038a69ed982910349` @ `10.42.1.56`                  |
| Remote 2 id      | `i-04cb4ccb7328736ef` @ `10.42.1.110`                 |
| Snapshot ids (pre-teardown, spire-aws stack) | coord `snap-0629f7f6387698e88`, remote-1 `snap-08e8600e1234bf509`, remote-2 `snap-078f1372782aa91b4` |
| Snapshot id (post-bench, live ecaz-cloud DB) | `snap-07ec6f61030ea02c2` on `vol-0b0357902542b97e0` (root of `i-04ce81ce1c10db4bc`) |

## Artifacts

| # | Artifact | Stage | Command | Timestamp | Key result |
|---|----------|-------|---------|-----------|-----------|
| 1 | `pass-correctness.log` (run 1 + run 2) | provision (fail) | `make pass-correctness` head `0b0793bb` then `9cfe0bcd` | 2026-05-23T16:32 UTC, 16:51 UTC | F7 (vCPU limit), F8 (Secrets recovery) — both resolved |
| 2 | `pass-smoke.log` | provision (succeeded) | `make pass-smoke` head `d8cccb29` | 2026-05-23T16:51 UTC | terraform applied 29 resources; topology written |
| 3 | `aws-topology.json` | provision | terraform output | 2026-05-23T16:51 UTC | `vpc-0a2f98585cd2fa2c7`, coord `i-068f303289c8e500d`, remotes `i-038a69ed982910349 / i-04cb4ccb7328736ef` |
| 4 | `pass-smoke-continuation.log` | install-extension (fail) | scripts/spire-aws/install.sh | 2026-05-23T16:55 UTC | F11 (SSM agent race) — fixed in commit `f35206c2` |
| 5 | `coord-fix-install.json` | install-extension (manual SSM, interrupted) | aws ssm send-command bootstrap fix | 2026-05-23T17:30 UTC | F12 (ssl=on without server.crt) — fixed in commit `d8cccb29`; instance auto-stopped mid-build |
| 6 | `snapshots.log` | EBS snapshot | `make snapshot` | 2026-05-23T17:30 UTC | 3 snapshots — coord `snap-0629f7f6387698e88`, remote-1 `snap-08e8600e1234bf509`, remote-2 `snap-078f1372782aa91b4` |
| 7 | `teardown-final.log` | teardown | `make teardown` | 2026-05-23T17:50 UTC | 24 resources destroyed; `aws ec2 describe-instances --filters Phase=...` empty |
| 8 | `bench-on-live-db/10k-latency.log` | bench (ec_ivf rabitq) | `bench-on-live-db.sh` 10k | 2026-05-23T17:55 UTC | **mean 382 ms, p50 65 ms, p95 66 ms, p99 383 ms** (cold-cache outlier 31 s) |
| 9 | `bench-on-live-db/100k-latency.log` | bench (ec_ivf rabitq rerank) | `bench-on-live-db.sh` 100k | 2026-05-23T17:56 UTC | **mean 1251 ms, p50 500 ms, p95 512 ms, p99 1273 ms** |
| 10 | `bench-on-live-db/1m-latency.log` | bench (ec_ivf rabitq rerank) | `bench-on-live-db.sh` 1m | 2026-05-23T18:01 UTC | **mean 5805 ms, p50 2479 ms, p95 2523 ms, p99 5865 ms** (1M dbpedia, 16 GB corpus, 298 MB index) |

### Bench provenance

- DB host: **`i-04ce81ce1c10db4bc`** (`ecaz-cloud-10k-medium-db`),
  `m8g.2xlarge` (Graviton 4 Neoverse V2, 8 vCPU, 32 GB), us-west-2.
  This is an **existing instance from prior `ecaz cloud` bench work**,
  not provisioned by this packet's `pass-correctness`/`pass-smoke`.
  Snapshot inventory shows the most recent prior state was
  `snap-091251b06d2da2df4` (post-vchord-paired-sweep, head
  `6c066017d515d6c73b6ac78ff5925ad2ea8cc0d2`, 2026-05-23T04:45 UTC).
- AL2023 / PG 18.3 / ecaz 0.1.1 (extension version, indexed at an
  earlier head SHA from the `aws-optimization-ivf-rabitq-spire`
  branch — not the SPIRE-AWS-prep head `9cfe0bcd`).
- 14 access methods registered including ec_hnsw, ec_diskann, ec_ivf,
  **ec_spire**, ivfflat, hnsw, vchordrq, vchordg.
- Indexes used (all `ec_ivf` storage_format=rabitq quant_bits=1):
  - `real_10k_ivf_rabitq1_idx`
  - `real_100k_ivf_rabitq1_rerank_rabitq_idx`
  - `real_1m_ivf_rabitq1_rerank_rabitq_idx`
- 100 queries per scale, k=10, self-sampled corpus rows as probe
  vectors (avoids on-the-fly `real[] → ecvector` encoding; latency
  characteristics are representative of any in-distribution probe).
- Driver: `scripts/spire-aws/bench-on-live-db.sh` + `scripts/spire-aws/bench.sql`
  via `aws ssm send-command` (base64-transported SQL to avoid SSM
  here-doc transport bugs — see findings).

## Findings

### F7 — r-family vCPU account quota = 16, original plan needed 40

Status: **resolved by topology downsize**. Run 1 (head `0b0793bb`,
2026-05-23 ~16:32 UTC) provisioned VPC + S3 + secrets + IAM + 3 of 4
EC2 instances, then aborted at `aws_instance.remote[1]` with:

    Error: creating EC2 Instance: VcpuLimitExceeded
    You have requested more vCPU capacity than your current vCPU
    limit of 16 allows for the instance bucket that the specified
    instance type belongs to.

Original sizing (1 × r8g.4xlarge + 3 × r8g.2xlarge = 16 + 24 = 40 vCPU)
required a quota increase request. Operator chose to downsize for this
initial smoke run: 1 × r8g.xlarge + 2 × r8g.xlarge = 12 vCPU total
(each node still has 32 GB RAM to handle the release-profile fat-LTO
peak observed in the prior Graviton baseline cycle).

Emergency teardown completed at ~16:38 UTC after ~6 min of partial
provisioning. 3 instances destroyed, no orphaned resources
(verified via `aws ec2 describe-instances --filters
'Name=tag:Phase,Values=13-spire-aws-verification'
'Name=instance-state-name,Values=pending,running'` returning empty).
Spend impact: ~$0.21.

Follow-up: request `Running On-Demand Standard instances` quota
increase from 16 → 64 vCPU for r-family before opening packet 959
(pass-representative on 1M dbpedia, which wants the larger nodes).

(F3–F6 from request.md are resolved by the pre-AWS plumbing commits;
F7 is the first runtime finding of this packet.)

### F8 — Secrets Manager deletion recovery window blocks re-provisioning

Status: **resolved by `purge-stuck-secrets` Makefile target**. After
`make teardown`, `aws_secretsmanager_secret.remote[*]` resources enter a
7-30 day deletion recovery window. The next `make provision` hits:

    Error: creating Secrets Manager Secret (ecaz-spire-aws-remote-2):
    InvalidRequestException: You can't create this secret because a
    secret with this name is already scheduled for deletion.

Added `purge-stuck-secrets` target that pre-emptively force-deletes
any `ecaz-spire-aws-remote-{1,2,3}` with
`--force-delete-without-recovery`. Wired into `pass-smoke`. (Should also
be wired into `pass-correctness` / `pass-representative` in a follow-up;
left out of this commit to keep blast radius narrow.)

Spend impact of this discovery: ~$0.50 across two aborted provisions.

### F10 — Bench shape findings on Graviton 4 (m8g.2xlarge)

Status: **recorded, not investigated**. ec_ivf RaBitQ quant_bits=1
on real DBpedia subsets, k=10, 100-query batch:

| Scale | n_rows | corpus | index | p50 ms | p95 ms | p99 ms | mean ms |
|-------|--------|--------|-------|--------|--------|--------|---------|
| 10k   | 10,000 | 163 MB | 4.1 MB | **65** | 66 | 383 | 382 |
| 100k  | 100,000 | 1.6 GB | 33 MB | **500** | 512 | 1273 | 1251 |
| 1M    | 990,000 | 16 GB | 298 MB | **2479** | 2523 | 5865 | 5805 |

Observations:

- **p50 scales linearly with N**: 65 → 500 → 2479 ms is roughly 7.7×
  per 10× corpus growth. Suggests significant per-N work beyond pure
  index-probe — likely full RaBitQ rerank over the candidate list
  (storage_format=rabitq with quant_bits=1 needs heavy rerank for
  recall) or memory bandwidth bound at 16 GB corpus.
- **Cold-cache first-query is the mean outlier**: max across 100
  queries is 31 s @ 10k, 75 s @ 100k, **335 s @ 1M**. p50 vs max
  diverges 470× at 1M. PG starts with empty shared_buffers from
  the previous bench session's cooldown; first kNN pulls the index
  into memory.
- These numbers are NOT representative of an optimized hot
  configuration. Prior packet `cloud-10k-graviton-preopt-baselines`
  recorded different numbers (lower) on the same hardware after warm
  shared_buffers; this bench was run cold. Hot-cache rerun is a
  follow-up.

### F9 — scripts/spire-aws/* assume laptop->VPC connectivity that doesn't exist

Status: **deferred to follow-up packet**; this packet pivots to a
single-node SSM-driven mini-suite.

`register.sh`, `load.sh`, `smoke.sh`, `bench.sh`, `fault.sh` all run
`ecaz dev sql --host <coordinator_private_ip> --user ecaz_coord` from
the laptop. Three problems:

1. The coordinator has no public IP; the laptop has no IP route into the
   VPC. `ecaz dev sql` has no SSM tunneling, so the psql attempt hangs
   or fails with "could not connect".
2. The `ecaz_coord` PG role is not created anywhere. Bootstrap creates
   the default `postgres` role via initdb but never `ecaz_coord`.
3. Some scripts (`load.sh`) call `ecaz corpus generate` on the laptop
   then expect to `ecaz corpus load` over a non-existent network path
   to the coord, which would push gigabytes of TSV over an SSM tunnel
   even if (1) and (2) were solved.

Pragmatic resolution for **this packet**:

- `bootstrap-node.sh` now `CREATE ROLE ecaz_coord WITH LOGIN SUPERUSER`
  (idempotent), unblocking (2).
- `scripts/spire-aws/_ssm-tunnel.sh` is added as the future remediation
  for (1) but is not used in this packet because the operator's laptop
  lacks `session-manager-plugin` (sudo apt install required).
- `scripts/spire-aws/single-node-smoke.sh` drives the corpus + index +
  bench **on the coordinator** via `aws ssm send-command` (which does
  not need session-manager-plugin). Restricts to one node — no
  multi-cluster remote registration — but produces the essential
  evidence: extension load on Graviton 4 + AL2023 + a recall + latency
  bench on a 10k synthetic corpus.

`make pass-smoke = provision → install-extension → single-node-smoke
→ snapshot → teardown`. Multi-cluster `pass-correctness` is left
intact for the follow-up packet that installs session-manager-plugin,
rewrites the scripts to use it, and validates end-to-end registration.
