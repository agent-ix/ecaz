# Review Request: A5 Concurrency Retry Hardening

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/06-graph-insert.md`
- `review/199-aminsert-graph-aware-insertion-roadmap/request.md`
- `spec/adr/ADR-026-live-insert-backlink-lock-ordering.md`
- `spec/functional/FR-016-hnsw-insert.md`

This is the final A5 checkpoint after drift accounting. It hardens the
full-slice backlink path against concurrent drift without changing the existing
page-order lock protocol.

Checkpoint scope:

1. stop silently dropping stale full-slice backlink rewrites
2. retry those targets through bounded read-only replanning
3. keep ADR-026 intact: no replan while holding a data-page `EXCLUSIVE` lock
4. add deterministic regression coverage for the stale-snapshot race
5. update task/status/ADR/spec docs to mark A5 closed

## Scope

- `src/am/insert.rs`
- `plan/tasks/06-graph-insert.md`
- `plan/status.md`
- `spec/adr/ADR-026-live-insert-backlink-lock-ordering.md`
- `spec/functional/FR-016-hnsw-insert.md`
- `review/README.md`

## What Landed

### 1. Backlink mutation now retries stale full-slice plans

Previously, a full-slice rewrite plan carried:

- `expected_slice`
- `replacement_slice`

If the live target layer no longer matched `expected_slice` under the page write
lock, the mutation was skipped outright.

Now the write path returns an explicit retry outcome for that case:

- `NoChange`
- `Changed`
- `RetryReplan`

That keeps the old safety rule of “never overwrite a slice that drifted under
us,” but it no longer silently drops the target forever.

### 2. Retry happens outside data-page write locks

The insert path now runs backlink mutation in bounded passes:

1. plan backlink mutations read-only
2. apply them page-by-page in ascending physical order
3. collect stale full-slice targets that need replanning
4. re-enter read-only planning for only those targets
5. retry through another ordered write pass

The retry count is intentionally bounded (`MAX_BACKLINK_REPLAN_PASSES = 3`).

Most importantly, stale-snapshot replanning does **not** happen while holding a
data-page `EXCLUSIVE` lock, so the ADR-026 deadlock surface stays unchanged.

### 3. The retry target is the logical neighbor, not just the page tuple

`BacklinkMutation` now carries the target element TID in addition to the
neighbor-tuple TID.

That lets the retry pass reload:

- the current target element
- the current neighbor tuple
- the current layer slice

from fresh read-side state before computing a new full-slice replacement.

### 4. Deterministic regression coverage now locks in the race behavior

Two new unit tests in `src/am/insert.rs` cover the exact stale-snapshot arc:

- `rewrite_full_slice_requests_retry_when_snapshot_drifted`
- `rewrite_full_slice_applies_after_replanning_against_current_slice`

Those tests prove:

- a stale full-slice plan no longer mutates the live layer blindly
- the stale plan surfaces an explicit retry request
- a replanned full-slice mutation can then admit the new node against the
  current live layer

This is concurrency-focused coverage of the hardening path itself, without
introducing extra cross-session harness machinery into the checkpoint.

### 5. Tracking/docs now mark A5 complete

The task/status/spec surfaces now record that:

- A5 is `100%` complete on `main`
- stale full-slice plans retry through bounded read-only replanning
- metadata-last lock ordering from ADR-026 is still the governing rule

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## New / Updated Coverage

- `am::insert::tests::rewrite_full_slice_requests_retry_when_snapshot_drifted`
- `am::insert::tests::rewrite_full_slice_applies_after_replanning_against_current_slice`

## Review Focus

- Is bounded stale-snapshot replanning the right final A5 contract, or does any
  remaining correctness gap still force a stronger retry model before A6?
- Does carrying retry planning strictly outside data-page `EXCLUSIVE` locks keep
  ADR-026 coherent, or is there a lock-order edge case still exposed here?
- Are the task/status/spec updates now aligned with the real merged behavior on
  `main`?
