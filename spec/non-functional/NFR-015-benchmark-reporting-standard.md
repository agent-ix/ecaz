---
id: NFR-015
title: Benchmark Reporting Standard
type: NFR
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

## Reporting Rules

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
8. Product benchmark reports SHALL additionally identify hardware, CPU
   architecture, storage class, PostgreSQL settings, cache-control procedure,
   repeat count, and variance or repeatability summary.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Comparison rows using the common reporting schema | 100% | 100% | Inspection |
| Required identity/quality fields present per row | Complete | Complete | Inspection |

## Verification

Reviewers inspect benchmark comparison reports to confirm every candidate row carries the common-schema identity fields (access method, opclass, storage/quantizer format, options, reloptions, GUC overrides, rerank mode) and the required environment, dataset, command, packet, claim-class, and recall/quality fields.

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
