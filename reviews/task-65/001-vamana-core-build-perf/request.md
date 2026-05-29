# Review Request: Vamana Core Build Perf

## Summary

This checkpoint implements the Task 65 single-process DiskANN build core overhaul:

- Code checkpoint: `351987249` (`Optimize DiskANN Vamana build core`)
- removes ambuild's O(N^2) exact-vector dedup scan;
- treats duplicate source vectors as distinct graph nodes during build;
- removes build-time overflow heap-TID staging while leaving runtime insert overflow support intact;
- switches Vamana graph construction from two full pivot passes to one shuffled pass with growing-alpha robust prune;
- introduces reusable `SearchScratch` with `Vec<u64>` bitsets for greedy search and candidate-pool dedup;
- routes build-time greedy search through scratch-backed search;
- replaces closest-frontier linear scans with a `BinaryHeap<Reverse<Candidate>>` unexpanded frontier;
- parallelizes per-node DiskANN payload encoding with rayon.

No scan, insert, persistence layout, or SIMD path is intentionally changed.

## Notes For Review

- The `overflow_ms` build timing field remains in the log surface and will now be zero for ambuild, preserving the existing timing line shape.
- Runtime `insert::stage_overflow_heap_tids_in_chain` is untouched and remains covered by existing insert tests.
- There is no repository `CHANGELOG` file in this checkout; the duplicate-node behavior is documented in `ambuild.rs`.
- The local macOS pgrx test binary still cannot execute because of the known `_BufferBlocks` dyld symbol issue, so this packet includes compile/no-run evidence rather than executed unit-test evidence.

## Validation

Artifacts are under `reviews/task-65/001-vamana-core-build-perf/artifacts/`.

- `cargo-check-pg18-lib.log`: `cargo check -p ecaz --lib --no-default-features --features pg18` passed.
- `cargo-test-vamana-no-run.log`: `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana --no-run` passed.
- `cargo-test-vamana-run.log`: compile succeeded, then execution aborted before tests ran with the local `_BufferBlocks` dyld issue.

## Follow-Up

Next checkpoint should run 10k real/synth corpus build timing before attempting 100k, and 1M should remain blocked until the smaller build curve is acceptable.
