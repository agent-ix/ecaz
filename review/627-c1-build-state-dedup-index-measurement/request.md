# Review Request: BuildState Dedup Index 50k Measurement

Current head: `cc23d28`

Scope:
- Code checkpoint: `cc23d28` (`Index build duplicate payload lookup`)
- `review/627-c1-build-state-dedup-index-measurement/artifacts/manifest.md`
- `review/627-c1-build-state-dedup-index-measurement/artifacts/pg18_build_state_dedup_index_50k_timing.sql`
- `review/627-c1-build-state-dedup-index-measurement/artifacts/pg18_build_state_dedup_index_50k_timing.log`

Question:
- Does replacing the linear duplicate scan in `BuildState::push` with a
  payload-keyed lookup materially reduce the 50k parallel build sort/push
  bottleneck identified in packet 626?

Result:
- Fixture: 50,000 rows, 64 dimensions, default `turboquant`, no
  `build_source_column`, `m = 6`, `ef_construction = 40`.
- Serial create-index time: 48,926.700 ms.
- Parallel create-index time: 46,103.259 ms.
- Parallel was about 5.8% faster than serial in this single 50k run.
- Both measured index sizes were identical at 11,624,448 bytes.

Comparison to packet 626:
- Serial create-index time improved from 53,683.587 ms to 48,926.700 ms
  (`~8.9%` faster).
- Parallel create-index time improved from 52,391.576 ms to 46,103.259 ms
  (`~12.0%` faster).
- Serial heap ingest fell from 8,618.142 ms to 1,471.241 ms.
- Parallel total heap ingest fell from 7,121.196 ms to 604.500 ms.
- Parallel sort plus `BuildState::push` fell from 6,706.119 ms to 157.275 ms.

Interpretation:
- This is a real path forward, not threshold tuning. The O(N^2) duplicate
  lookup was removed from build ingest, and the packet-local phase counter
  shows the leader sort/push bottleneck dropping by roughly 97.7%.
- After this change, graph construction is again the dominant cost:
  47,023.806 ms in serial and 45,105.472 ms in parallel on this fixture.
- The remaining parallel build opportunity is graph assembly itself, not
  tuple transport or `BuildState::push`.

Validation:
- Code checkpoint gates run before commit:
  - `cargo test`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `git diff --check`
- Raw log is stored packet-locally at
  `artifacts/pg18_build_state_dedup_index_50k_timing.log`.
- SQL fixture is stored packet-locally at
  `artifacts/pg18_build_state_dedup_index_50k_timing.sql`.
- Artifact metadata and key lines are recorded in `artifacts/manifest.md`.
- The run used `ecaz dev sql --pg 18 --log-output`, not shell redirection or
  `script`.

Review focus:
- Whether this closes the packet-626 `BuildState::push` bottleneck.
- Whether the next implementation slice should target graph assembly cost,
  now that tuple ingest and leader sort/push are no longer the dominant
  parallel-build overhead.
