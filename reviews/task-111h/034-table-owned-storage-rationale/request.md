# Task 111h / 034 Table-Owned Storage Rationale

Code/docs commit under review:

- `c3ce05a542e640b5c5a2613ed5351ef7ea4622a0` -
  `task111h: close table-owned storage decision`

## Summary

This packet closes the Task 111h table-owned compact payload checklist item by
rejecting `rerank_placement = 'table'` as a 111h product path and recording the
replacement:

- `source` remains the exact table/source-vector baseline. It uses the existing
  f32 source vector, adds no compact payload storage, and is the warm-cache
  matched-recall reference.
- `index` remains the persisted compact payload path for f16, RaBitQ-4,
  RaBitQ-8, and TurboQuant. That is the architecture under active 111h review.
- `table` stays reserved and errors until a separate PostgreSQL-owned storage
  design exists. It must not mean query-time f32-to-compact conversion.

The evidence is in `artifacts/table-owned-storage-audit.md`.

## Evidence Readout

- Current code has only two product read paths: source heap fetch and index
  packed group fetch. `rerank_placement = 'table'` is rejected during reloption
  resolution.
- Build and insert encode compact payloads only for
  `rerank_placement = 'index'`; non-index placements return no sidecar payload.
- The historical companion-table harness is explicitly a benchmark harness, not
  an index feature. It creates unlogged fixed-width `bytea` tables and does not
  define INSERT/UPDATE/DELETE/MVCC maintenance for a product AM path.
- Existing packet-local companion-table evidence rejects naive random-id lookup
  and shows TID-sorted table reads are a separate storage design with load-bearing
  static-corpus assumptions.

## Review Focus

- Confirm the rejection is scoped correctly: not "PostgreSQL can never store
  compact payloads", but "Task 111h should not promote a `table` placement
  without a new DDL/MVCC/storage design."
- Confirm the replacement is acceptable for 111h: exact existing table storage
  is `source/f32`; compact persisted storage is `index`.

## Validation

No runtime tests or new benchmarks were run for this packet. This is a
documentation/evidence checkpoint over existing code and packet-local benchmark
artifacts.
