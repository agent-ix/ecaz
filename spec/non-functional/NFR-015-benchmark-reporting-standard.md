---
id: NFR-015
title: Benchmark Reporting Standard
type: NFR
quality_attribute: compliance
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-006"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/NFR-007"
    type: "extends"
    cardinality: "1:1"
---
# NFR-015: Benchmark Reporting Standard

## Statement

Ecaz benchmark reports SHALL use one common reporting schema across access
methods, quantizers, storage formats, and option sets.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Reporting-schema compliance | 100% of benchmark report rows carry the required candidate identity, environment, and metric fields for their metric family | no exceptions | repo audit of `docs/benchmarks.md` and `docs/benchmark-index.md` against `docs/benchmark-reporting-standard.md` |
| Provenance completeness | every reported run includes the environment, dataset, command, artifact packet, and claim class required by NFR-007 | no exceptions | packet manifest audit |

1. Every candidate comparison row SHALL identify the access method, opclass,
   storage or quantizer format, format-specific options, AM reloptions, session
   GUC overrides, and rerank mode.
2. Every reported run SHALL include the environment, dataset, command,
   artifact packet, and claim class required by `NFR-007`.
3. Recall and quality reports SHALL include corpus rows, query rows,
   dimensionality, distance metric, recall@10, recall@100, and nDCG when the
   harness emits it.
4. Latency reports SHALL include iteration count, cache state, p50, p95, p99,
   mean, and whether the planner path was forced or natural.
5. Storage reports SHALL include index size, table size when relevant,
   per-row or per-vector bytes when derivable, and any required model metadata
   or rerank sidecar.
6. Memory reports SHALL distinguish backend RSS, high-water mark, build memory,
   and query-time memory when the harness exposes those fields.
7. Build, ingest, update, delete, vacuum, and distributed transport reports
   SHALL use the same candidate identity fields as recall and latency reports.
8. Block-kernel counter reports SHALL include access-method surface,
   `quant_kind`, `isa`, total/kernel/scalar flushes, candidates and elapsed
   time, and width buckets `<8`, `8..15`, `16..31`, and `>=32`.
9. Latency and recall reports SHALL include backend build profile provenance
   from `NFR-007` when the run is suite-driven or otherwise intended as task
   closeout evidence.
10. Product benchmark reports SHALL additionally identify hardware, CPU
   architecture, storage class, PostgreSQL settings, cache-control procedure,
   repeat count, and variance or repeatability summary.

## Verification

Compliance is checked by auditing benchmark documentation against the
reporting standard: `docs/benchmark-reporting-standard.md` is checked to
define the required reporting fields for all current metric families, and
`docs/benchmarks.md` / `docs/benchmark-index.md` are checked to link to the
standard and to contain no benchmark rows whose scope cannot be traced to
packet-local evidence or an explicit gap. Cross-format and block-kernel
comparison rows are reviewed for the common candidate identity fields and the
counter fields required by the rules above.

## Acceptance Criteria

### NFR-015-AC-1

`docs/benchmark-reporting-standard.md` defines the required reporting fields
for all current metric families.

### NFR-015-AC-2

`docs/benchmarks.md` and `docs/benchmark-index.md` link to the reporting
standard and avoid benchmark rows whose scope cannot be traced to packet-local
evidence or an explicit gap.

### NFR-015-AC-3

Future comparisons between `turboquant`, `pq_fastscan`, `rabitq`, trained
quantizers, and future formats report the same candidate identity and metric
fields so the rows can be compared across `ec_hnsw`, `ec_ivf`, `ec_diskann`,
`ec_spire`, and future access methods.

### NFR-015-AC-4

Block-kernel comparisons preserve enough counter fields to distinguish
scoring-share wins from end-to-end latency changes and to identify
structurally absent or missing-kernel cells in the Task 99 matrix.
