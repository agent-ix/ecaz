# Review Request: C1 ADR-030 V2 PqFastScan Insert And Vacuum Scan Assertions

## Context

The branch already had structural `PqFastScan` insert and vacuum checkpoints:

- built-index live insert appends grouped hot + rerank + neighbor tuples
- duplicate insert coalesces grouped hot tuples
- vacuum stats / compaction / repair / finalize all have `PqFastScan` coverage

But those tests were still mostly structural:

- tuple counts
- tuple tags
- hot/rerank payload decode
- deleted-edge unlink / replacement bookkeeping

Task 15 also wants confidence that these lifecycle operations still preserve the
ordered-scan behavior users actually care about.

## Problem

Without this slice, the branch could prove:

1. `PqFastScan` live insert wrote the right tuple shapes
2. `PqFastScan` vacuum repaired the right tuple shapes

but it did not yet prove, at the test surface, that:

1. a newly inserted row becomes visible as the best ordered-scan result for its
   own embedding
2. a deleted row disappears from ordered-scan results after vacuum repair

That was a missing behavior-level check, not a missing structural check.

## Planned Slice

One test-only checkpoint:

1. extend the existing built-index `PqFastScan` insert test with an ordered-scan
   assertion
2. extend the existing `PqFastScan` vacuum unlink test with before/after
   ordered-scan assertions for the deleted row's embedding

No AM behavior change.

## Implementation

Updated:

- `src/lib.rs`

### 1. Insert test now verifies ordered-scan visibility of the new row

Extended:

- `test_tqhnsw_insert_appends_to_built_pq_fastscan_index`

After the existing tuple-shape checks, the test now:

- runs `debug_gettuple_scan_heap_tids_with_scores(...)` with the inserted row's
  embedding as the query
- maps heap TIDs back to table row ids
- asserts that row `17` ranks first

This checks that the inserted `PqFastScan` row is not just persisted, but is
also surfaced correctly by the ordered scan path.

### 2. Vacuum test now verifies deleted-row disappearance in ordered scan

Extended:

- `test_tqhnsw_vacuum_pass2_unlinks_pq_fastscan_refs`

The test now:

- reconstructs the deleted row's original embedding deterministically from the
  runtime fixture formula
- runs an ordered scan before delete/vacuum and asserts the deleted row ranks
  first for its own embedding
- runs the existing delete + debug vacuum repair path
- reruns ordered scan and asserts the deleted row no longer appears in emitted
  results

That gives the vacuum path a user-visible ordered-scan behavior check rather
than only structural tuple assertions.

## Measurements

No benchmark or recall rerun in this slice.

## Validation

Passed:

- `cargo check --tests`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family, including:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This slice raises the proof level for task 15 without changing runtime code:

1. built-index `PqFastScan` live insert now has an ordered-scan visibility
   assertion
2. `PqFastScan` vacuum repair now has a deleted-row disappearance assertion
3. insert/vacuum coverage now better reflects user-visible scan behavior, not
   just on-disk structure

What this slice intentionally does **not** do:

- change any production AM logic
- solve empty-index `PqFastScan` insert
- replace the need for a real corpus-scale end-to-end validation pass

## Next Slice

The remaining important work is still functional:

1. close the outstanding task-15 parity gaps that still block a true `main`
   landing
2. decide whether empty-index `PqFastScan` insert needs a dedicated design +
   implementation checkpoint
