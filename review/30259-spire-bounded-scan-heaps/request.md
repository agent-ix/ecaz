# Review Request: SPIRE Bounded Scan Heaps

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `ed9c4ee5 Add SPIRE bounded scan heaps`

## Scope

This packet covers the A5 pre-persistence architecture feedback slice for
bounded top-k selection in SPIRE scan helpers.

Changed files:

- `src/am/ec_spire/scan.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Replaced full sort/truncate in `route_root_object_to_leaf_pids` with a
  bounded heap keyed so the heap head is the worst retained route.
- Replaced final candidate sort/truncate in `rank_routed_leaf_rows_by_ip` with
  a bounded heap after existing `vec_id` dedupe.
- Preserved deterministic route ordering:
  higher centroid inner product, then lower centroid index, then lower child
  PID.
- Tightened deterministic candidate ordering:
  lower ORDER BY score, heap TID, PID, row index, then `vec_id` bytes.
- Added regression coverage for bounded route selection and bounded candidate
  limiting.
- Updated the Task 30 plan and architecture feedback response note to mark the
  bounded-heap gate item complete.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `172 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

This checkpoint intentionally leaves A7 open: Phase 1 still dedupes through the
`vec_id` `HashMap`. The heap is applied after that dedupe step, matching the
current correctness contract. A later slice will make dedupe mode explicit so
the primary-only local path can skip the map when boundary replicas are off.
