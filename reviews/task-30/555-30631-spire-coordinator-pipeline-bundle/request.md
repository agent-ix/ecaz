# SPIRE Coordinator Pipeline Bundle

## Scope

Task 30 SPIRE Phase 7 now has an internal coordinator pipeline bundle for the
highest-level remote search gate.

Code checkpoint: `6ab8c8b3` (`Add SPIRE coordinator pipeline bundle`)

## Changes

- Added reusable internal helpers that derive execution rows, libpq request
  rows, connection rows, dispatch rows, dispatch summaries, and receive rows
  from already-computed pipeline state.
- Added `SpireCoordinatorPipeline::execute_once(...)` for
  `remote_search_coordinator_gate_summary_row(...)`.
- Updated the coordinator gate to compute target readiness once, then derive
  execution, dispatch, receive, merge, finalization, and executor-readiness
  summaries from that shared pipeline bundle.
- Updated the Phase 7 task note to record the new bundle.

## Validation

- `cargo fmt`
- `cargo test --no-default-features --features "pg18 pg_test" test_ec_spire_remote_search_coordinator_gate_summary`
- `git diff --check`

The first validation run failed because the empty-dispatch summary fallback used
a synthetic zero vector for query metadata. The fix carries the original query
through the pipeline; the rerun passed.

## Review Focus

- Whether the shared bundle preserves existing coordinator-gate SQL output.
- Whether the helper boundaries are the right starting point for a future
  libpq executor to consume a single pre-I/O pipeline result instead of
  recomputing fanout/readiness summaries.
