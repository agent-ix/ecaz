---
id: NFR-001
title: Query Latency
type: NFR
status: APPROVED
traces:
  - StR-001
---
# NFR-001: Query Latency

## Statement

Ecaz query and scoring paths SHALL meet the latency and throughput targets below on representative hardware.

### HNSW Index Scan

- p50 latency: < 5ms for top-10 query on a 50K-vector, 1536-dim, 4-bit table (m=8, ef_search=40)
- p99 latency: < 15ms under steady-state load

### Sequential Scan (small agents)

- The extension SHALL publish measured compressed-domain scoring throughput in scores/sec and rows/sec for representative sequential scans
- Upstream routing thresholds for choosing sequential scan vs HNSW SHALL be calibrated from those measurements; no fixed row-count threshold is normative in this specification

### Distance Function

- Single `tqvector_inner_product` call: benchmarked and reported at 1536-dim, 4-bit
- Prepared-query scoring throughput (`score_ip_encoded`) SHALL be benchmarked separately from symmetric SQL-function scoring because they have different cost profiles

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| HNSW top-10 p50 latency (50K×1536, 4-bit, m=8, ef_search=40) | < 5 ms | <= 5 ms | Latency Benchmark |
| HNSW top-10 p99 latency (steady-state load) | < 15 ms | <= 15 ms | Latency Benchmark |
| Prepared-query scoring throughput (`score_ip_encoded`, 1536-dim, 4-bit) | Reported per release | Reported | Isolated Scoring Benchmark |


Benchmarks SHALL be run on representative hardware and reported in `BENCHMARKS.md`.

The real-corpus latency lane reuses the canonical loader path documented in
`docs/RECALL_REAL_CORPUS.md` (see "Reusing the Loaded Tables for NFR-001
Latency`). Durable HNSW artifacts should use `ecaz bench latency` against a
canonically loaded corpus. Artifacts MUST record the exact `ecaz` invocation,
the selected profile/prefix/sweep surface, and a representative `EXPLAIN` plan
showing that the expected `ec_hnsw` index was chosen for the measured run.

### Required Methodology

- Use a fixed dataset, fixed query set, and fixed random seeds for all compared runs.
- Report hardware, CPU model, RAM, storage class, PostgreSQL version, build profile, and relevant PostgreSQL settings.
- Measure HNSW latency with the same `m`, `ef_construction`, and `ef_search` values used in recall benchmarks.
- Report warm-cache and cold-cache results separately when feasible.
- Measure query latency as wall-clock time from statement start to last row returned, excluding network transport.
- Measure single-call scoring latency in an isolated benchmark harness, not by extrapolating from full SQL query timings.

### Required Comparisons

- Compare prepared-query scoring throughput against symmetric code-to-code scoring throughput at the same dimension and bit-width.
- Compare HNSW query latency against sequential scan throughput on the same dataset.
- Compare insert latency before and after enabling the index.

## Verification

A repeatable `ecaz bench latency` run against a canonically loaded corpus measures p50/p99 HNSW latency and prepared-query scoring throughput, captures the `EXPLAIN` plan confirming the expected `ec_hnsw` index, and asserts the measured values stay within threshold.

