# Task 223: ec_distann Direct Owner Tuple Materialization

Status: **proposed, gated on Task 222** (2026-08-21). Priority: P1 latency.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidate
MAT-41. This is a new physical tuple-access candidate, not a reopening of Task
220's rejected SQL concatenation.

## Why

The current owner endpoint resolves row-tier TIDs and then executes generated
`unnest`/LATERAL SPI SQL, constructs `boolean[]` and `bytea[]` values, decodes
those arrays in Rust, and flattens them into the wire response. Task 220 changed
the SQL expression shape and regressed badly; it did not test bypassing SPI and
array construction altogether.

Task 222 may remove most payload bytes and send-function work. This task starts
only from Task 222's refreshed profile so it targets the residual rather than
optimizing an obsolete whole-row denominator.

## Goal

Determine whether direct row-tier tuple fetch plus cached per-attribute binary
send calls can remove material owner endpoint latency while preserving
PostgreSQL snapshot, type, null, TOAST, ordering, and failure semantics.

## Entry gate

1. Task 222 is review-closed with its production disposition applied to the
   control.
2. Feature-only counters separate relation/slot setup, heap tuple fetch,
   detoast/send-function work, SPI/executor work, array construction/decode,
   and response assembly.
3. Direct access proceeds only if the measured addressable residual is at least
   1 ms/scan or 5% of warm end-to-end mean at 100k.

## Scope

### P1 — Refreshed substage attribution

Use the production lazy-10, Task-222 control to measure the owner SQL bucket by
substage and payload shape. Instrumentation is benchmark-only and must reconcile
to owner endpoint wall/work counters.

### P2 — One direct tuple candidate

Use already-resolved immutable row-tier TIDs, a relation-backed tuple slot, and
generation/schema-cached type I/O metadata. Encode only Task 222's proven
attribute mask directly into the existing ordered payload contract. Preserve
missing-row, tombstone, snapshot, and schema-fingerprint failures exactly.

### P3 — Decision

Run one same-generation 100k A/B. Advance only a useful result to the standard
10k/50k/100k recall + latency + storage matrix through `ecaz bench suite`.

## Non-goals

- Repeating chained `bytea ||` or cumulative `octet_length` SQL.
- Changing the payload wire format and tuple-access mechanism in the same arm.
- Heap-block sorting, prefetch, overlap, or caching; Tasks 224 and 225 own those
  conditional families.
- Changing row-tier storage or traversal.

## Acceptance

1. The refreshed substage decomposition selects or rejects direct tuple access
   from a measurable ceiling.
2. Semantic coverage includes null, toasted, dropped-column/schema mismatch,
   mixed-owner, qual deepening, restart, and outage cases.
3. Predictions and SQL results are byte-identical to control where ordering is
   part of the contract.
4. The task records STOP or a complete 10k/50k/100k decision matrix.

## Required review packets

1. `reviews/task-223/001-plan/`
2. `reviews/task-223/002-owner-substage-attribution/`
3. `reviews/task-223/003-direct-tuple-screen/`
4. `reviews/task-223/004-full-scale-decision/` (only after a useful screen)

## References

- Task 222
- Task 220's rejected MAT-16 SQL form
- `src/am/ec_distann/generation_read.rs` physical payload endpoint
- `src/am/ec_distann/remote_endpoint.rs` payload SQL builders
