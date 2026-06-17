---
id: NFR-011
title: Cloud Corpus Load Throughput
type: NFR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/StR-007"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-047"
    type: "constrains"
    cardinality: "1:1"
---
# NFR-011: Cloud Corpus Load Throughput

## Statement

In-VPC parquet → COPY load throughput SHALL meet the targets below
so that a 100M-row corpus is loadable in a single working session,
not a multi-day operation.

## Targets

Initial targets (subject to revision after the first `1m` and `10m`
runs land their measurement artifacts):

| Profile | Rows | Target wall time | Implied rows/sec |
|---|---|---|---|
| `dev` (50k) | 5e4 | < 60 s | ≥ 800 |
| `1m` | 1e6 | < 10 min | ≥ 1700 |
| `10m` | 1e7 | < 90 min | ≥ 1850 |
| `100m` | 1e8 | < 12 hours | ≥ 2300 |

Targets exclude index build time, which is measured and reported
separately (FR-047 §4).

## Policy

1. Throughput SHALL be measured from the start of the first worker
   to completion of the last worker.
2. The wall-time target is for the load phase only; index builds
   are out of scope for this NFR but are also recorded.
3. If a profile misses its target by more than 25%, the next
   benchmark suite run SHALL include a `load-throughput-regression`
   review packet.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| `dev` (50k) load wall time | < 60 s | <= 60 s | Load Benchmark |
| `1m` load wall time | < 10 min | <= 10 min | Load Benchmark |
| `10m` load wall time | < 90 min | <= 90 min | Load Benchmark |
| `100m` load wall time | < 12 hours | <= 12 hours | Load Benchmark |

## Verification

Throughput is measured from the start of the first worker to completion of the last worker (load phase only), recorded in `throughput.json`, uploaded to the profile S3 bucket, and asserted against the per-profile wall-time targets; a miss of more than 25% triggers a `load-throughput-regression` review packet.

## Acceptance Criteria

### NFR-011-AC-1

The `dev`-profile load completes within the target wall time on
the first end-to-end smoke run.

### NFR-011-AC-2

`corpus load` artifacts include a `throughput.json` recording
`rows`, `bytes`, `wall_seconds`, `rows_per_sec`, `bytes_per_sec`,
and `worker_count`.

### NFR-011-AC-3

Throughput artifacts are uploaded to the profile's S3 bucket under
`bench-artifacts/<run-id>/load/`.
