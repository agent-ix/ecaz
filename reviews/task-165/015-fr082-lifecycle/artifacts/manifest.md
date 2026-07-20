# Manifest — Task 165 packet 015 (FR-082 epoch lifecycle)

- head SHA: e4141a00a (+ lifecycle e-series commits)
- branch: task-165-ec-distann-m3
- date: 2026-07-08
- surface: pg_test (in-process PG18), one index per table. No fixture/bench run
  (the lifecycle state machine + tombstone semantics are unit-level; the
  concurrency/frozen-tier/swap-under-load pieces are the disk-blocked tail).

## Code
- src/am/ec_distann/page.rs — format v4 (epoch_state, active_epoch, in_flight_count)
- src/am/ec_distann/epoch_manifest.rs — publish/retire/force-retire/status/debug-set-in-flight
- src/am/ec_distann/scan.rs — run_scan_attempt_with_restart (+4 unit tests)
- src/am/ec_distann/routine.rs — restart wrapper wired into collect_distann_hits
- src/am/ec_distann/remote_endpoint.rs — ec_distann_owning_node surface
- src/tests/ec_distann_basic.rs — lifecycle, tombstone AC-4/AC-5, owning_node tests

## Commands
- cargo pgrx test pg18 --no-default-features --features pg18 distann
- cargo clippy --lib --no-default-features --features pg18

## Key result
- test result: ok. 110 passed; 0 failed (was 103 at session start)
- restart_* (4), epoch_lifecycle_publish_retire_override, tombstone_excludes_and_preserves_live_vectors, owning_node_surface — all ok
