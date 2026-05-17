# Review Request: Task 41 HNSW Debug Heap Scan Resource Guards

## Summary

Task 41 slice for HNSW pg_test heap-backed scan helpers.

The code commit `a9589ed432e7e1488d3b42996680173a5c3b9e13` wraps the manual PostgreSQL resource lifecycle in `src/am/ec_hnsw/scan_debug.rs`:

- `DebugActiveSnapshotGuard` owns `RegisterSnapshot` / `PushActiveSnapshot` and the matching pop/unregister
- `DebugIndexScanGuard` owns `index_beginscan` and the matching `index_endscan`
- `DebugTupleSlotGuard` owns heap-fetch tuple slot allocation and drop
- `DebugHeapBackedScan` now stores guards instead of raw relation/snapshot/scan resources
- `debug_profile_ordered_scan_with_heap_fetch` now unwinds through guards instead of hand-written error cleanup blocks

## Baseline Delta

- unsafe baseline entries: `4284 -> 4265`
- `src/am/ec_hnsw/scan_debug.rs`: `461 -> 442`

See `artifacts/manifest.md` and `artifacts/validation.md`.

## Validation

- `cargo fmt`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `make fmt-check`

`cargo check` passed with the existing PG18 C-header warnings and the existing unused re-export warning in `src/am/mod.rs`.

## Review Focus

- Confirm `DebugHeapBackedScan` field order preserves cleanup order: scan end, snapshot pop/unregister, index relation close, heap relation close.
- Confirm the heap-fetch profile keeps `scan_guard` and `slot_guard` alive for the full raw-pointer use scope.
- Confirm the local guards are acceptable here, or identify which of these should be promoted into a shared PostgreSQL resource guard module for reuse in IVF debug scan code.
