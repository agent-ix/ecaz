# Review Request: Native Build Layer-Search Scratch Reuse

Current head: `2515e4b`

Scope:
- `src/am/ec_hnsw/build.rs`
- `review/625-c1-native-build-layer-search-scratch-measurement/artifacts/manifest.md`
- `review/625-c1-native-build-layer-search-scratch-measurement/artifacts/pg18_layer_search_scratch_timing.sql`
- `review/625-c1-native-build-layer-search-scratch-measurement/artifacts/pg18_layer_search_scratch_timing.log`

Question:
- Can native HNSW graph construction shed more time by reusing layer-search
  scratch state inside the build loop instead of allocating a visited set,
  heaps, and successor vectors for every upper/layer-0 search?

Code Change:
- Added a build-local `NativeBuildLayerSearchScratch` that owns reusable:
  - visited set
  - candidate heap
  - result heap
  - successor buffer
- Replaced the native-build calls to the generic
  `graph::search_layer0_result_candidates_with_successors` with a local search
  helper that preserves the same ordering and pruning semantics while reusing
  those buffers.
- Kept the generic graph search surface unchanged for scan/vacuum callers.

Result:
- Fixture: 10,000 rows, 64 dimensions, default `turboquant`, no
  `build_source_column`, `m = 6`, `ef_construction = 40`.
- Serial create-index times:
  - round 1: 7929.252 ms
  - round 2: 7921.307 ms
- Parallel create-index times:
  - round 1: 7610.618 ms
  - round 2: 7720.030 ms
- Average serial: 7925.280 ms, versus 8823.079 ms in packet 624 and
  9013.102 ms in packet 622.
- Average parallel: 7665.324 ms, versus 8698.641 ms in packet 624 and
  8694.528 ms in packet 622.
- Current parallel average is about 3.3% faster than current serial average.
- All measured index sizes were identical at 2,334,720 bytes.

Phase Breakdown:
- Average serial graph construction: 7304.026 ms, versus 8197.469 ms in
  packet 624 and 8384.270 ms in packet 622.
- Average parallel graph construction: 7184.037 ms, versus 8220.860 ms in
  packet 624 and 8214.275 ms in packet 622.
- Average serial heap ingest: 532.775 ms.
- Average parallel heap ingest total: 398.580 ms.
- Average parallel queue drain/finish: 135.155 ms.
- Average parallel sort plus `BuildState::push`: 260.542 ms.

Interpretation:
- This is a material graph-construction improvement, not threshold tuning.
- The parallel build path now shows useful wall-clock improvement on the 10k
  PG18 fixture, but graph construction is still the dominant phase.
- The next performance path should keep reducing graph-build search/allocation
  costs or move to a more parallel graph-construction strategy; heap ingest is
  no longer the main limiter.

Validation:
- Passed:
  - `cargo test hnsw_graph_build --lib`
  - `cargo pgrx test pg18 test_pg18_parallel_index_build_uses_workers`
  - `cargo test`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `cargo pgrx test pg18`
  - `git diff --check`
- Raw measurement log is stored packet-locally at
  `artifacts/pg18_layer_search_scratch_timing.log`.
- SQL fixture is stored packet-locally at
  `artifacts/pg18_layer_search_scratch_timing.sql`.
- Artifact metadata and key lines are recorded in `artifacts/manifest.md`.
- The run used `ecaz dev sql --pg 18 --log-output`, not shell redirection or
  `script`.

Review focus:
- Whether duplicating the generic layer-search loop locally for native build is
  acceptable to keep scratch reuse isolated from scan/vacuum behavior.
- Whether the ordering and tie-break semantics match the generic helper closely
  enough for maintainability.
- Whether this result is sufficient to proceed to larger-fixture validation.
