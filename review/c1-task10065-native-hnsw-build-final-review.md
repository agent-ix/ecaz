# Final Review — c1-task10065-native-hnsw-build-path

Branch: `c1-task10065-native-hnsw-build-path`
Head at review: `85af073`
Scope: 19 commits vs `main`, replacing the `hnsw_rs`-backed BUILD path with a
native in-crate HNSW builder and adding stabilization / coverage / optimization
slices on top.

Note: this review was written before the later `458` and `459` packets landed.
Those follow-ons added the `ecvector` extension upgrade path and the cached
real-corpus baseline rerun packet; they did not reopen the native-build verdict
recorded here.

## Verdict

**Merge-ready.** The branch lands a coherent replacement:

- production BUILD is now on tqvector-owned HNSW primitives and reuses the
  same `ScoredBacklinkNode<NodeId>` + `select_best_backlink_candidates`
  selection logic that INSERT uses, so the two paths cannot drift on the
  tie-break ordering
- `hnsw_rs` is gone from `Cargo.toml`, `src/lib.rs`, and all test probes
  (`ab117ea`); the only remaining artifact is `vendor/hnsw_rs/`, left on disk
  per an earlier task constraint and documented in packet `449`
- every concern raised in the original `446` feedback has been addressed in
  code (see verification below), not just acknowledged
- helper-level coverage and a direct source-build parity gate are in place
  so the native heuristics are pinned without depending on end-to-end recall
  alone
- three repeated-work optimization slices (`455`/`456`/`457`) are
  behavior-preserving and leave ordering, tie-breaks, and persisted layout
  untouched

No blockers surfaced during the full-branch read.

## 446-concern verification (checked against current code, not just claims)

1. **Redundant upper-layer descent removed.** `build_native_hnsw_graph`
   (`src/am/build.rs:1507–1529`) now threads the `layer0_seeds` returned from
   `populate_native_upper_layer_forward_slots(...)` straight into the layer-0
   search. The separate `greedy_descend_with_successors(...)` call is gone.
2. **Build-time level cap respects `state.page_size`.**
   `choose_insert_level_for_page_size(...)` in `insert.rs` now takes the page
   size explicitly; `build.rs:1491` uses it, and
   `debug_assert_eq!(state.page_size, BLCKSZ, ...)` at `build.rs:1481` pins
   the current invariant for production.
3. **Silent slot clamping replaced with an invariant check.**
   `load_native_successor_candidates` (`build.rs:1618–1632`) now uses
   `debug_assert!(end <= nodes[source_idx].neighbor_slots.len(), ...)` and a
   plain `start..end` slice.
4. **Upper-layer `ef_construction` choice documented.** Comment at
   `build.rs:1574–1578` explains why native build keeps `ef_construction`-width
   successor search on upper layers instead of classical `ef=1` greedy
   descent, and flags FR-021 as the future revisit point.

Minor residual: `flatten_native_neighbor_slots` (`build.rs:1747`) still uses
`start.min(len)..end.min(len)` slicing. That helper runs off persisted slot
layout so the clamp is harmless, but for consistency with the load helper it
could also become a `debug_assert!` + plain slice in a follow-up. Not a
merge blocker.

## Helper coverage status

The in-tree unit tests now cover the helpers flagged as fragile by the 446
review (`src/am/build.rs` test module):

- `hnsw_graph_build_is_deterministic_for_scalar_codes`
- `hnsw_graph_build_is_deterministic_for_source_vectors`
- `source_scored_entry_point_prefers_raw_vectors`
- `native_forward_slot_packing_preserves_layer_boundaries_with_padding`
- `flatten_native_neighbor_slots_dedups_and_skips_origin`
- `add_native_backlinks_uses_free_slot_before_rewrite`
- `add_native_backlinks_rewrites_full_slice_for_better_candidate`
- `choose_insert_level_for_page_size_respects_supplied_page_size` (in
  `src/am/insert.rs`)

Plus the `test_tqhnsw_graph_scan_recall_source_gate_10k` ignored pg test at
25 queries for source-build parity reruns.

This matches the replacement-test list the 446 feedback asked for before
pulling the `hnsw_rs` proxy.

## Real-corpus evidence

From packet `448` artifacts (real 50k TurboQuant gate, head `89a8c46`):

- `m=8,  ef_search=40   → recall@10 = 0.886`
- `m=8,  ef_search=128  → recall@10 = 0.930, exact@10 = 0.890`
- `m=8,  ef_search=200  → recall@10 = 0.930`
- `m=16, ef_search=200  → recall@10 = 0.964`

These are consistent with the post-task16 TurboQuant surface and are the
primary merge-gate evidence.

## Residual measurement caveat (not a merge blocker)

The 448 feedback asked for either a native source-graph oracle lane or a
real-corpus run on a non-TurboQuant surface (grouped heap-f32 / pq_fastscan)
before closeout. Current state:

- the direct source gate at 25 queries (`454`) confirms source-build
  stability across reruns but does not compare against a brute-force oracle
- real-corpus recall evidence remains TurboQuant-only

If that coverage is already considered satisfied by other branch evidence,
fine. Otherwise, one follow-up packet with a grouped-heap-f32 or pq_fastscan
gate run is the lowest-cost way to turn the current "asserted by inference"
claim into measured evidence. Calling it out because the "proof not
assumptions" rule applies here — but it's a follow-on, not a block.

## Recommended merge-sequence follow-ups (post-merge, not blockers)

1. Delete `vendor/hnsw_rs/` once the earlier task constraint no longer
   applies, so `grep -R hnsw_rs .` comes back empty.
2. One-line cleanup of the `flatten_native_neighbor_slots` `.min(len)` slice
   for symmetry with the load helper.
3. The FR-021 parallel BUILD slice, when started, should revisit the
   `ef_construction`-width upper-layer walk (packet 450 already flagged
   this).
4. If BUILD allocator pressure ever becomes a measured hotspot, the
   `NativeBuildQueryScorer`'s per-insertion `Vec<Option<f32>>` allocation
   can be hoisted and reset with `.fill(None)` — but optimize then, with
   evidence.

## Individual packet feedback

Already written to:

- `review/455-c1-native-build-query-score-cache/feedback.md`
- `review/456-c1-native-build-backlink-score-cache/feedback.md`
- `review/457-c1-native-build-seed-copy-cleanup/feedback.md`
- `review/446-c1-native-hnsw-build-path/feedback.md` (original, verified
  against current code in this final pass)
- `review/448-c1-native-build-real-corpus-gate/feedback.md`

No changes to those individual verdicts as a result of the full-branch read.
