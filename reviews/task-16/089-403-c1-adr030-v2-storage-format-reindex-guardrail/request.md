# Review Request: C1 ADR-030 V2 Storage-Format REINDEX Guardrail

## Context

Packet `378` introduced per-index `storage_format` selection and packet `387`
documented the operator contract:

- `WITH (storage_format = 'turboquant' | 'pq_fastscan')` chooses the on-disk
  format at build time
- switching formats later requires `REINDEX`

Reviewer feedback on both packets called out the same footgun:

- `ALTER INDEX ... SET (storage_format=...)` updates the reloption
- but it does **not** rewrite the on-disk metadata
- so runtime could silently read one thing from reloptions and a different
  thing from the actual index pages

That is unacceptable for a mainline landing surface. The reloption and the
persisted metadata need to agree, or runtime has to fail loudly.

## Problem

Before this slice, runtime lifecycle paths trusted on-disk metadata alone:

- ordered scan
- live insert
- vacuum
- debug scan helpers
- tuple counting helpers used by vacuum stats

That meant a reloption-only format switch could leave the index in a mixed
state:

- reloption says `pq_fastscan`
- metadata says `turboquant`

or vice versa.

The index would not be rewritten, but the operator would also not get a direct,
actionable error explaining the mismatch.

## Planned Slice

Add one shared guardrail seam and thread it through all runtime open paths:

1. decode the actual storage descriptor from metadata
2. decode the expected `storage_format` from reloptions
3. reject mismatches with a direct `REINDEX after switching formats` error
4. add pg coverage that proves `ALTER INDEX ... SET (storage_format=...)`
   without `REINDEX` fails during ordered scan

This slice intentionally does not:

- reject `ALTER INDEX ... SET (storage_format=...)` itself
- rewrite indexes automatically
- change build-time format selection
- change on-disk wire tags

## Implementation

Updated:

- `src/am/options.rs`
- `src/am/graph.rs`
- `src/am/insert.rs`
- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/am/shared.rs`
- `src/am/vacuum.rs`
- `src/lib.rs`

Concrete changes:

1. added `StorageFormat::as_str()` in `src/am/options.rs`
2. added `GraphStorageDescriptor::from_index_relation(...)` in `src/am/graph.rs`
   - derives the actual descriptor from metadata
   - reads the expected reloption from the live index relation
   - errors on mismatch with:
     - `tqhnsw index reloption storage_format=... does not match on-disk metadata format=...; REINDEX after switching formats`
3. routed runtime/open paths through that helper:
   - ordered scan setup in `src/am/scan.rs`
   - insert adapter resolution in `src/am/insert.rs`
   - vacuum adapter resolution in `src/am/vacuum.rs`
   - tuple counting in `src/am/shared.rs`
   - grouped debug scan validation in `src/am/scan_debug.rs`
4. kept metadata-only unit tests metadata-only by resolving directly from
   `GraphStorageDescriptor::from_metadata(...)` where no live relation exists
5. added pg coverage in `src/lib.rs`:
   - build a `turboquant` index
   - change only the reloption to `pq_fastscan`
   - force ordered index scan
   - assert runtime panics with the explicit `REINDEX` guidance

## Validation

Passed:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands were run and still hit the same workstation linker
boundary as the rest of this branch:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This slice turns the reloption/metadata mismatch footgun into an explicit
runtime guardrail:

1. users still choose formats per index through `storage_format`
2. users can still flip the reloption later if they want
3. but runtime now refuses to pretend that a reloption-only flip rewrote the
   index
4. the failure mode tells the operator exactly what to do next: `REINDEX`

That closes the most obvious operator trap left by packet `378` and makes the
README guidance from packet `387` enforceable in code instead of aspirational.

## Next Slice

Use the hardened canonical real-corpus harness to run the task-15 `50k`
explicit-format recall lanes on the actual canonical `pq_fastscan` /
`turboquant` index families, then capture that execution proof in a review
packet.
