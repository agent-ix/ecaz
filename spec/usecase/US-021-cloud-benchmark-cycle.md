---
id: US-021
title: Run a Complete Cloud Benchmark Cycle
type: user-story
artifact_type: US
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/StR-007"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-044"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-045"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-046"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-047"
    type: "derives_into"
    cardinality: "1:N"
---
# US-021: Run a Complete Cloud Benchmark Cycle

## Story

**As** an Ecaz operator working from a developer workstation with AWS
credentials in `AWS_PROFILE`,
**I want** to provision a cloud-hosted Ecaz database at a chosen scale, load a
named corpus, run the benchmark suite, capture artifacts, and tear the
environment down through `ecaz cloud ...` commands,
**So that** product-scale benchmark cycles are repeatable, auditable, and
bounded by explicit cost controls.

## Preconditions

- AWS credentials are configured and have permission to create VPC,
  EC2, EBS, S3, IAM, and SSM resources in the target region.
- `terraform` is installed and on `PATH`.
- The branch's ecaz extension builds for `aarch64-unknown-linux-gnu`.

## Main Flow

1. Operator runs `ecaz cloud up --profile 1m`.
2. Harness applies the Terraform module for the `1m` profile,
   provisioning VPC, S3 bucket, DB EC2 (Graviton), and loader EC2.
3. Cloud-init on the DB host installs Postgres 18 and builds ecaz
   from the current branch SHA via `cargo pgrx install --release`.
4. Harness waits for SSM agent readiness, runs `CREATE EXTENSION
   ecaz` via `tokio-postgres`, and reports a queryable DSN.
5. Operator runs `ecaz cloud corpus stage --dataset dbpedia-1m`.
6. Harness downloads parquet shards from Hugging Face and uploads
   them to the profile's S3 bucket.
7. Operator runs `ecaz cloud corpus load --dataset dbpedia-1m`.
8. Harness fans out parallel workers on the loader EC2; each runs
   the existing `ecaz corpus prepare` + `ecaz corpus load` against
   the DB's private IP.
9. Operator runs `ecaz cloud bench --suite smoke`.
10. Harness invokes the existing `ecaz bench` entry points against
    the remote DSN and uploads `--log-file` artifacts to S3.
11. Operator runs `ecaz cloud down`, which destroys the stack.

## Alternate Flow A — Pause and Resume

- After step 10, operator runs `ecaz cloud pause`. EC2 instances
  stop; EBS volumes are retained.
- Later, operator runs `ecaz cloud resume`, then jumps back to
  step 9 without re-loading the corpus.

## Alternate Flow B — Snapshot Reuse

- Before step 11, operator runs `ecaz cloud snapshot`. Harness
  takes an EBS snapshot of the DB volume.
- A future `ecaz cloud up --profile 1m --from-snapshot <id>` skips
  steps 5–8 entirely.

## Postconditions

- Bench artifacts are durable in the profile's S3 bucket with
  lifecycle rules attached.
- After `down`, `ecaz cloud status` reports zero paid resources.

## Notes

- Auth is environment-driven (`AWS_PROFILE`); operators expect the harness to
  fail loudly with remediation text rather than prompt interactively.
- Every verb is idempotent and resumable.

## Acceptance Criteria

### US-021-AC-1

`ecaz cloud up --profile 1m` provisions the selected profile and reaches a
queryable PostgreSQL 18 database with `ecaz` installed.

### US-021-AC-2

`ecaz cloud corpus stage` and `ecaz cloud corpus load` stage the named corpus
inside the profile's S3/VPC boundary and load it through parallel workers
without manual SSH or SQL.

### US-021-AC-3

`ecaz cloud bench --suite smoke` runs the benchmark suite against the remote
DSN and uploads packetable `--log-file` artifacts.

### US-021-AC-4

`ecaz cloud pause`, `resume`, and snapshot restore preserve loaded data and
avoid re-loading the corpus when the profile is resumed.

### US-021-AC-5

`ecaz cloud down` followed by `ecaz cloud status` reports zero paid resources
for the profile.
