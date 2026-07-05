# Task 51 Packet 002 Artifact Manifest

- task bucket: `reviews/task-51/002-ivf-heap-prefetch-dedup`
- IVF code commit under review: `6c066017d` (`Deduplicate IVF heap rerank prefetch blocks`)
- validation head SHA: `863f8b0c8f9c6e7543e57b4b9929354a86f20f04`
- timestamp: `2026-05-22T21:46:30-07:00`
- surface: local PG18 only; no AWS
- lane: `ec_ivf` RaBitQ, heap_f32 rerank
- storage format: `rabitq`
- rerank mode: `heap_f32`
- isolated one-index-per-table smoke surface: yes

## Artifacts

- `cargo-check-pg18.log`
  - command: `cargo check --lib --no-default-features --features pg18`
  - result: passed
  - key line: `Finished dev profile`
  - note: existing unrelated warnings remain in `src/am/mod.rs`, `src/am/ec_ivf/build.rs`, and `src/quant/rabitq.rs`.
- `rustfmt-scoped.log`
  - command: `rustfmt --check src/am/ec_ivf/scan.rs`
  - result: passed
  - note: rustfmt emitted existing stable-channel warnings for unstable config keys.
- `git-diff-check.log`
  - command: `git diff --check -- src/am/ec_ivf/scan.rs`
  - result: passed
- `cargo-test-candidate-heap-blocks.log`
  - command: `cargo test --lib candidate_heap_blocks_collapses_adjacent_sorted_blocks --no-default-features --features pg18`
  - result: blocked before test execution by existing local lib-test harness symbol failure
  - key lines: `undefined symbol: BufferBlocks`, `error: test failed`
- `run-pg18-ivf-rabitq-prefetch-smoke.sh`
  - starts an isolated temporary PG18 cluster with `shared_preload_libraries=ecaz`
  - creates an `ec_ivf` RaBitQ index with `rerank = 'heap_f32'` and `rerank_width = 3`
  - runs `EXPLAIN (ecaz, ANALYZE, COSTS OFF, VERBOSE)` for a local KNN query
- `pg18-ivf-rabitq-prefetch-smoke.log`
  - command: `bash reviews/task-51/002-ivf-heap-prefetch-dedup/artifacts/run-pg18-ivf-rabitq-prefetch-smoke.sh`
  - result: passed
  - key lines:
    - `shared_preload_libraries | ecaz`
    - `Rerank Rows: 3`
    - `Heap Blocks Fetched: 1`
    - `Execution Time: 0.664 ms`
