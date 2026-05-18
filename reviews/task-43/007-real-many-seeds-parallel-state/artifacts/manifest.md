# Artifact Manifest: Task 43 Real Many-Seeds Parallel State

- Head SHA: `7f2eb68b52e31e5456ca61ee7ddc79ba7c77bd85`
- Task bucket: `reviews/task-43/`
- Packet: `reviews/task-43/007-real-many-seeds-parallel-state/`
- Timestamp: `2026-05-18T12:30:18-07:00`
- Storage surface: N/A, pure common parallel shared-state unit tests
- Rerank mode: N/A
- Table isolation: N/A, no PostgreSQL tables or indexes were created

## Code Checkpoint

- `7f2eb68b` - adds real threaded Miri coverage for common parallel scan
  worker-slot claim/publish/release paths and promotes stale-epoch publish
  rejection to the `miri_` prefix. The checkpoint also changes shared-state
  initialization to keep raw-pointer provenance across the full AM-private
  parallel descriptor instead of creating a header-only mutable reference
  before writing adjacent coordinator and worker-slot regions.

## Validation Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `miri-parallel-threaded-default.log` | `cargo +nightly miri test --lib miri_parallel_worker_slots_are_unique_under_threaded_contention` | 1 passed; 0 failed; 1791 filtered out; exit 0 |
| `miri-parallel-threaded-tree.log` | `MIRIFLAGS=-Zmiri-tree-borrows cargo +nightly miri test --lib miri_parallel_worker_slots_are_unique_under_threaded_contention` | 1 passed; 0 failed; 1791 filtered out; exit 0 |
| `miri-parallel-threaded-many-seeds.log` | `MIRIFLAGS=-Zmiri-many-seeds=0..128 cargo +nightly miri test --lib miri_parallel_worker_slots_are_unique_under_threaded_contention` | 128 seed attempts; completed batches report 1 passed, 0 failed; exit 0 |
| `miri-parallel-stale-epoch-default.log` | `cargo +nightly miri test --lib miri_publish_parallel_scan_worker_slot_runtime_snapshot_rejects_stale_epoch` | 1 passed; 0 failed; 1791 filtered out; exit 0 |
| `cargo-fmt-check.log` | `cargo fmt --all -- --check` | exit 0 |
| `git-diff-check.log` | `git diff --check` | exit 0 |

## Notes

- The threaded test uses the production common parallel shared-state layout:
  `EcParallelScanState`, `EcParallelCoordinatorState`, and
  `EcParallelWorkerSlot`.
- The test spawns more Rust threads than available worker slots, forces all
  threads to attempt a claim before any successful claimant releases, and then
  asserts one live claim per slot, no duplicate slot ownership, idle runtime
  reset on release, and a final coordinator claim count of zero.
- This packet closes the campaign tracker's real many-seeds gate, but it does
  not close the separate common-parallel mutation-probe row. Mutation probes
  remain tracked for packet 012.
