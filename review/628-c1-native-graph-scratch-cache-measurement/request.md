# Review Request: Native Graph Scratch Cache Measurement

Current head: `0be01d1`

Scope:
- Code checkpoint: `0be01d1` (`Reuse native graph build scratch caches`)
- `review/628-c1-native-graph-scratch-cache-measurement/artifacts/manifest.md`
- `review/628-c1-native-graph-scratch-cache-measurement/artifacts/pg18_native_graph_scratch_cache_50k_timing.sql`
- `review/628-c1-native-graph-scratch-cache-measurement/artifacts/pg18_native_graph_scratch_cache_50k_timing.log`

Question:
- Does replacing hash-table work in native graph assembly scratch with
  reusable indexed/linear caches materially reduce the now-dominant graph
  phase identified in packet 627?

Result:
- Fixture: 50,000 rows, 64 dimensions, default `turboquant`, no
  `build_source_column`, `m = 6`, `ef_construction = 40`.
- Serial create-index time: 30,333.870 ms.
- Parallel create-index time: 30,845.279 ms.
- Parallel was slightly slower than serial in this single run because the
  remaining graph phase dominates and is still serial leader work.
- Both measured index sizes were identical at 11,624,448 bytes.

Comparison to packet 627:
- Serial create-index time improved from 48,926.700 ms to 30,333.870 ms
  (`~38.0%` faster).
- Parallel create-index time improved from 46,103.259 ms to 30,845.279 ms
  (`~33.1%` faster).
- Serial graph construction fell from 47,023.806 ms to 28,556.153 ms.
- Parallel graph construction fell from 45,105.472 ms to 29,816.052 ms.
- Parallel sort plus `BuildState::push` stayed small: 181.685 ms.

Interpretation:
- The graph-scratch cache change is a real win. The native graph builder was
  paying substantial hash-table overhead in hot search/backlink loops; removing
  that work cut the 50k graph phase by roughly one third to two fifths.
- Tuple ingest is no longer the relevant bottleneck on this fixture. The
  remaining path to a faster parallel build needs to either parallelize graph
  assembly or reduce the serial HNSW insertion search work.
- The dedicated parallel heap scan still works and launches 4 workers, but at
  this point it cannot overcome the serial graph phase by itself.

Validation:
- Code checkpoint gates run before commit:
  - `cargo test`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `git diff --check`
- Raw log is stored packet-locally at
  `artifacts/pg18_native_graph_scratch_cache_50k_timing.log`.
- SQL fixture is stored packet-locally at
  `artifacts/pg18_native_graph_scratch_cache_50k_timing.sql`.
- Artifact metadata and key lines are recorded in `artifacts/manifest.md`.
- The run used `ecaz dev sql --pg 18 --log-output`, not shell redirection or
  `script`.

Review focus:
- Whether this closes the graph-scratch hash overhead slice.
- Whether the next implementation should stay in serial graph assembly
  optimization or start a larger design packet for true parallel graph
  assembly.
