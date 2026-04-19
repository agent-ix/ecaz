# Review Request: C1 Task16 Persisted Pq FastScan Rerank Source Column

Current head at execution: `a3e1ea0`

## Context

Packet `430` established two important facts on current head:

- a packed raw-f32 heap rerank source column can materially improve the
  recall-preserving `heap_f32` lane
- the only current-head way to use that source was the ad hoc env override
  `TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN`

That left the useful path as measurement-only plumbing rather than a supported
index contract. This slice productizes that result for `pq_fastscan` indexes by
adding a persisted reloption-backed rerank source column.

## What Landed

### Reloption surface

- added `rerank_source_column` to `TqHnswOptions` / `TqHnswReloptions`
- registered `rerank_source_column` as an index reloption
- rejected `rerank_source_column` unless
  `storage_format = 'pq_fastscan'`

### Build-time validation

- non-empty build now validates the persisted rerank source column against the
  heap relation before build proceeds
- empty build performs the same validation up front so broken reloptions fail
  at index creation time
- the validation uses the shared source-column resolver with a
  `real[] or bytea` type policy
- source-resolution errors now mention the caller label, so failures on this
  path explicitly name `rerank_source_column`

### Scan/runtime resolution

- default grouped rerank mode now resolves to `heap_f32` when a persisted
  `rerank_source_column` exists, even without an env override
- effective rerank source precedence is now:
  - `TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN`
  - persisted `rerank_source_column`
  - persisted `build_source_column`
- runtime settings now surface the new default-resolution reason
  `default_heap_f32_with_rerank_source_column`
- the heap-rerank error path now names all supported source selectors instead
  of only `build_source_column`

### Coverage

- pg fixtures can now create a `pq_fastscan` index with a persisted
  `source_raw` rerank source column
- new pg coverage verifies:
  - runtime settings report the persisted rerank source
  - persisted `bytea` rerank emits exact heap scores without env overrides
  - non-`pq_fastscan` indexes reject `rerank_source_column`
  - missing rerank source columns fail cleanly
  - wrong rerank source types fail cleanly
- new scan-unit coverage verifies that a persisted rerank source column flips
  the default rerank mode to `heap_f32`

## Validation

Green on this head:

- `cargo test`
- `bash scripts/run_pgrx_pg17_test.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Readout

### 1. Packet `430`'s useful measurement now has a durable AM contract

The source-raw heap rerank path no longer depends on a session env override.
`pq_fastscan` indexes can persist the intended raw rerank source directly in
index metadata.

### 2. This is deliberately narrower than changing `build_source_column`

`build_source_column` still means "derive grouped build/search inputs from this
heap column". `rerank_source_column` is the new narrower control surface for
"use this raw heap column for heap-f32 rerank".

That separation lets the AM keep grouped-code derivation on one column while
using a different raw `real[]` or packed `bytea` heap column for rerank.

### 3. The env override remains the top-priority escape hatch

The runtime precedence intentionally keeps the existing env override above the
persisted reloption so measurement/debug sessions can still force a different
source column without rebuilding the index.

### 4. The packet does not solve the stale-TID maintenance constraint from `430`

Packet `430` showed that backfilling a new heap column on an already-populated
table left the existing index pointing at stale heap TIDs until reindex.

This slice makes the rerank source durable and validated, but explicit
maintenance / rebuild semantics for existing indexes after heap rewrites remain
follow-on work.
