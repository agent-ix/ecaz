## Feedback: Source-Backed PqFastScan Default Heap Rerank

Read `src/am/scan.rs::default_grouped_rerank_mode` at :1010–1018 and
`resolve_grouped_rerank_mode` at :1020+, the new regression
`test_pq_fastscan_default_source_rerank_emits_heap_scores` in
`src/lib.rs`, and the restart-wrapper default flip in
`scripts/restart_adr030_scratch.sh`. Cross-referenced against the
live-cluster artifacts in `tmp/real_corpus_runs/`.

### What's right

- **The default is now selected by layout, not by policy.** The source-
  backed default-to-`heap_f32` rule is driven by
  `build_source_column.is_some()`, which is a build-time fact persisted
  on the index — not a runtime guess. That means the default adapts
  to what the index actually has, so a source-less `pq_fastscan` index
  (no heap column) correctly stays on `quantized` instead of panicking
  trying to read a column it doesn't have.
- **Env precedence preserved, cleanly.** `resolve_grouped_rerank_mode`
  still short-circuits on `TQVECTOR_PQ_FASTSCAN_RERANK_MODE`, so
  operator intent beats layout-derived default. That is the only
  ordering that works — a user who set `quantized` explicitly for a
  latency reason should not have the layout override them.
- **Solves the exact recall gap measured on the live cluster.** The
  source-backed heap-rerank path lifts `m=8, ef=128` from `0.8826` to
  `0.9078` on the `50k` gate — the one row that was still missing
  `0.89`. Moving the default to this lane closes the last task-15
  real-corpus gap without changing the on-disk format or adding a
  new reloption.
- **Quantized path kept under test.** Pinning the quantized profile
  tests with `TQVECTOR_PQ_FASTSCAN_RERANK_MODE=quantized` is the right
  way to keep that branch covered after flipping the default — it
  turns the env override into a test-only escape hatch for the
  behavior that used to be the default.
- **Restart wrapper default aligned.** As with packet `401`, keeping
  the scratch wrapper's no-flag path in sync with the code default
  removes a whole class of "which lane did my benchmark hit" bug.

### Concerns

1. **No new recall number on the packaged test.** The `heap_f32`
   default was proven by a live-cluster rerun on `~/.pgrx`, which is
   the exact cluster packet `402` just finished teaching the wrappers
   to be suspicious of. The live readout is believable (artifacts
   exist, runtime-settings helper confirms the lane), but the packet
   does not include any `pg_fastscan` `50k` gate run using the
   now-hardened scratch wrappers. A single rerun through
   `scripts/bench_sql_latency_verified_scratch.sh` with explicit
   `TQV_PG_SOCKET_DIR` would close that loop cleanly.
2. **Source-column assumption.** `build_source_column.is_some()`
   decides the default, but having the reloption set is not the same
   as the column being readable from the heap tuple at scan time
   (dropped column, renamed column, type-changed column). The heap
   rerank path already has to handle that, but the new *default*
   makes this a hotter code path. Worth confirming that the
   heap-rerank error when the source column is missing/unreadable
   produces a clear operator message, not a bare panic — if an
   operator alters a source column and suddenly every scan starts
   erroring, the error needs to name the column.
3. **Regression coverage proves emission, not correctness.**
   `test_pq_fastscan_default_source_rerank_emits_heap_scores`
   confirms the default path emits heap-rerank counters. That's the
   right shape of unit test, but it doesn't assert the scores it
   emits actually match what the `heap_f32` override path would
   produce. A direct parity assertion (default path scores ==
   explicit-heap path scores on the same fixture) would make the
   default-flip self-evidently safe.
4. **Exported-default constant flipped to `heap_f32`.** The debug
   helpers now report `heap_f32` as the rerank default globally. An
   operator inspecting a *source-less* `pq_fastscan` index through
   the global debug helper will see "rerank default = heap_f32" even
   though their index will actually resolve to `quantized`. Packet
   `408`'s index-aware helper exists for exactly this reason, but
   the global helper is still misleading on this axis.
5. **`cargo pgrx test pg17` still never executed.** Same systemic
   gap as the rest of the arc. Since this packet added a pg test
   (`test_pq_fastscan_default_source_rerank_emits_heap_scores`) and
   modified the expectations of an existing test (the canonical
   rerank-profile SQL-surface test), both are unproven until CI runs
   them — and this project has no CI running `cargo pgrx test pg17`
   on any lane.

### Observation

Right lever pulled at the right time. This is the one remaining
runtime choice that was silently leaving recall on the table in the
default configuration, and flipping it converts the "miss one gate
row" problem into "cleared all four gate rows on the real-corpus
`50k`." Coupled with `408`'s per-index visibility helper, operators
will now be able to see *why* their `pq_fastscan` index gets the
rerank mode it does.
