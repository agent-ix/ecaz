---
id: NFR-001
title: Query Latency
type: non-functional-requirement
status: APPROVED
traces:
  - StR-001
---
# NFR-001: Query Latency

## Requirement

### HNSW Index Scan

- p50 latency: < 5ms for top-10 query on a 50K-vector, 1536-dim, 4-bit table (m=8, ef_search=40)
- p99 latency: < 15ms under steady-state load

### Sequential Scan (small agents)

- The extension SHALL publish measured compressed-domain scoring throughput in scores/sec and rows/sec for representative sequential scans
- Upstream routing thresholds for choosing sequential scan vs HNSW SHALL be calibrated from those measurements; no fixed row-count threshold is normative in this specification

### Distance Function

- Single `tqvector_inner_product` call: benchmarked and reported at 1536-dim, 4-bit
- Prepared-query scoring throughput (`score_ip_encoded`) SHALL be benchmarked separately from symmetric SQL-function scoring because they have different cost profiles

## Measurement

Benchmarks SHALL be run on representative hardware and reported in `BENCHMARKS.md`.

The real-corpus latency lane reuses the canonical loader path documented in
`docs/RECALL_REAL_CORPUS.md` (see "Reusing the Loaded Tables for NFR-001
Latency"). Durable HNSW artifacts should use
`scripts/bench_sql_latency_verified.sh --prefix <canonical-prefix> --m <m>`,
which aborts unless a representative `EXPLAIN` plan selects the expected
tqhnsw index for that run. The delegated reporting surface remains
`scripts/bench_sql_latency.sh`, which emits per-cell `p50` / `p95` / `p99`
summaries, `server_qps` derived from the summed per-query timing surface for
the selected mode (`EXPLAIN (ANALYZE)` by default, or plain server-side
statement timing when requested), and a stdout environment banner for artifact
capture.

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
