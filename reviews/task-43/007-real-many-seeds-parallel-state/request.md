# Review Request: Task 43 Real Many-Seeds Parallel State

## Summary

This checkpoint closes the campaign tracker's most serious lane gap:
`miri-many-seeds` now has real concurrent code to schedule.

Changes:

- Adds `miri_parallel_worker_slots_are_unique_under_threaded_contention` in
  `src/am/common/parallel.rs`.
  - Uses the real AM-private parallel scan shared-state layout.
  - Spawns more Rust threads than worker slots.
  - Forces all workers to contend before any successful claimant releases.
  - Asserts unique slot ownership, runtime snapshot publication, release reset,
    and final coordinator claim count.
- Promotes stale-epoch publish rejection into the Miri prefix with
  `miri_publish_parallel_scan_worker_slot_runtime_snapshot_rejects_stale_epoch`.
- Fixes parallel shared-state initialization so Miri sees raw-pointer
  provenance for the whole AM-private descriptor rather than a header-only
  `&mut EcParallelScanState` followed by adjacent raw writes.
- Updates the campaign tracker under packet 001: G2 and common-parallel
  shared-state rows are now done, while mutation-probe work remains open.

## Review Focus

- Confirm the threaded test exercises real shared-state atomics rather than a
  synthetic counter.
- Confirm the initialization change is the right provenance fix for writing
  the header, coordinator, and worker-slot regions inside one AM-private
  descriptor.
- Confirm the tracker status is accurate: real many-seeds coverage is done,
  but common-parallel mutation probing is still open.

## Validation

Validation artifacts are in `artifacts/` and summarized by
`artifacts/manifest.md`.

- Default Miri threaded test passed.
- Tree Borrows threaded test passed.
- Full `0..128` many-seeds threaded test passed and recorded 128 seed attempts.
- Stale-epoch publish rejection passed under default Miri.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

Normal `cargo test --lib ...` was not used as evidence because the pgrx test
binary fails to start outside PostgreSQL with unresolved server symbols such as
`LockBuffer`; these pure checks are intended to run under Miri.
