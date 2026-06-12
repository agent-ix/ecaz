---
id: NFR-011
title: Cloud Corpus Load Throughput
type: non-functional-requirement
artifact_type: NFR
quality_attribute: performance_efficiency
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

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| `dev` (50k) profile load wall time (5e4 rows) | < 60 s (>= 800 rows/sec) | 60 s | `corpus load` `throughput.json` artifact |
| `1m` profile load wall time (1e6 rows) | < 10 min (>= 1700 rows/sec) | 10 min | `corpus load` `throughput.json` artifact |
| `10m` profile load wall time (1e7 rows) | < 90 min (>= 1850 rows/sec) | 90 min | `corpus load` `throughput.json` artifact |
| `100m` profile load wall time (1e8 rows) | < 12 hours (>= 2300 rows/sec) | 12 hours | `corpus load` `throughput.json` artifact |

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

## Verification

Compliance is checked from `corpus load` artifacts: throughput is measured
from the start of the first worker to completion of the last worker (load
phase only; index builds excluded), recorded in a `throughput.json` artifact
with `rows`, `bytes`, `wall_seconds`, `rows_per_sec`, `bytes_per_sec`, and
`worker_count`, and uploaded to the profile's S3 bucket under
`bench-artifacts/<run-id>/load/`. The `dev`-profile target is verified on the
first end-to-end smoke run; a profile that misses its target by more than 25%
triggers a `load-throughput-regression` review packet in the next benchmark
suite run.

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
