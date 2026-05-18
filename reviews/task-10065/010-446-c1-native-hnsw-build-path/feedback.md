# Feedback: 446-c1-native-hnsw-build-path

Reviewed at branch `c1-task10065-native-hnsw-build-path` (main..HEAD: 3 commits). Companion feedback for 448 covers the real-corpus gate; this file focuses on the native builder itself.

## Verdict

**Approve with a follow-up direction.** `hnsw_rs` currently stays in `[dev-dependencies]` with a set of `#[ignore]`d comparison probes in `src/lib.rs` (`probe_hnsw_rs_code_graph_recall`, `probe_hnsw_rs_source_graph_recall`, `test_hnsw_rs_*_10k*`, and `use hnsw_rs::prelude::{AnnT, Distance, Hnsw}` at line 1106). Keeping it during native-build development is fine — it's a useful scaffold while the native builder is taking shape and you want a second implementation to sanity-check against. But it should not be a permanent fixture.

Important to be clear about what those probes are: they are **not oracle tests**. An oracle compares against a trusted reference, and on our corpus `hnsw_rs` scores *worse* than tqvector (code-graph 10k: hnsw_rs = 0.2900 vs tqvector build-code = 0.8050 vs exact quantized = 0.8400). They're a black-box comparison lane against a foreign implementation that doesn't share our distance encoding — useful as a development sanity check, not as a correctness signal.

**Departure plan to record now (ADR-042 or a follow-up task):**

1. **Exit criterion:** once the native build is stable and direct heuristic tests (below) are in place, `hnsw_rs` comes out entirely — `Cargo.toml` entry, `src/lib.rs:1106` import, `probe_hnsw_rs_*` helpers, `test_hnsw_rs_*` tests, and `vendor/hnsw_rs/`. `grep -R hnsw_rs src/ Cargo.toml vendor/` must come back empty.
2. **Replacement test coverage** (the thing that lets you pull the proxy with confidence):
   - `choose_insert_level` — fixed-seed determinism + distribution check against the geometric prior.
   - `selected_forward_slot_bounds` / `backlink_slot_bounds` — layer 0, layer == level, layer > level, m==1, tight page sizes.
   - `select_best_backlink_candidates` — hand-scored candidates, assert chosen slice + tie-break order (score → is_new → node_id).
   - `add_native_backlinks` — free-slot path vs. replacement path with forced full slice.
   - `populate_native_upper_layer_forward_slots` — hand-built upper-layer graph, assert forward slots per layer.
   - `flatten_native_neighbor_slots` — dedup + self-exclusion on a crafted slot array.
   - Small deterministic end-to-end golden (say 16 points, fixed seed) with expected adjacency recorded.
3. **Budget:** set a visible deadline (e.g. "by end of C1 native-build series" or a specific follow-up task number) so the dev-dep doesn't quietly become permanent.

Approve this packet as-is on the `hnsw_rs` dimension. Address the concerns below for the native builder itself.

## Strengths

- **True code reuse with INSERT.** `choose_insert_level`, `selected_forward_slot_bounds`, `backlink_slot_bounds`, and the new generic `ScoredBacklinkNode<NodeId>` + `select_best_backlink_candidates` are shared. No forked heuristic stack — matches the packet's claim.
- **Deterministic tie-breaking preserved.** `add_native_backlinks` sorts selections by `(node_idx, layer)` and dedups before rewriting; `select_best_backlink_candidates` keeps the `score → is_new → tie_break` ordering `insert.rs` used.
- **Clean metric seam.** `BuildGraphMetric::{Code,Source}` collapses the old `BuildCodeDistance`/`BuildVectorDistance` pair and removes the HNSW non-negativity `score_offset` hack (no longer needed once we own distance sign convention).
- **Neighbor slot layout unchanged** so `flush_build_state` and downstream page staging are untouched. Good blast radius.

## Concerns

1. **Redundant upper-layer traversal (`src/am/build.rs` ~L1440–1480).** `populate_native_upper_layer_forward_slots` already walks every layer from `entry_level` down using `ef_construction`, then the caller immediately runs `greedy_descend_with_successors(entry_candidate, max_level, …)` again before the layer-0 search. The second descent starts from the original `entry_candidate`, not from the seeds the first pass converged to — is this intentional, or should layer-0 seed from the first pass's final `seeds`? If intentional, add a one-line comment explaining *why*; otherwise it reads like duplicated work. Matters for Q1 (ADR-042 serial shape) since build cost scales with this.

2. **`ef_construction`-width walk on upper layers is non-standard.** Classical HNSW uses greedy `ef=1` descent above `insert_level`. Using `ef_construction` everywhere is probably *why* recall looks strong, but it inflates build cost vs. hnsw_rs and has no comment acknowledging the deviation. If this is the intended ADR-042 choice, record it (ADR or code comment); it will surface again under FR-021 parallelization.

3. **Level-cap divergence.** Old builder clamped via `page::max_level_that_fits(m_u16, state.page_size)`. Native builder delegates to `insert::choose_insert_level`, which caps via `max_insert_level_that_fits(m, code_len, pg_sys::BLCKSZ as usize)`. If `state.page_size != BLCKSZ` in any test harness (mocked page size), the cap drifts. Add an assert that `state.page_size == BLCKSZ` for build, or route the cap through `state.page_size`.

4. **`load_native_successor_candidates` silent clamping.** The slice `start.min(len)..end.min(len)` swallows out-of-range slot bounds instead of panicking. `layer_slot_bounds` already returns `Option`, so this double-guard hides the one real invariant (slots length consistent with node level). Prefer `debug_assert!(end <= slots.len())`.

## Answers to review questions

1. **Right serial shape for ADR-042?** Shape is fine; the cost profile (ef_construction on every upper layer × redundant descent) will need revisiting for FR-021. Capture Concerns 1–2 in the ADR or a TODO so the parallel feed work doesn't inherit them silently.
2. **INSERT backlink semantics preserved?** Yes — shared helper + explicit selection dedup/sort matches. One edge: `add_native_backlinks` skips re-insertion if `layer_slice.contains(&Some(new_node_idx))` *before* the free-slot scan, which would shadow a stale `Some(new_node_idx)` from a prior iteration. Given `selections` dedup this can't occur within one insert; just noting.
3. **Source-graph parity gaps?** Yes, evidence gap: the oracle lanes (0.30 / 0.2850 / 0.6550) are hnsw_rs-vs-oracle readouts, not native-vs-oracle — this packet doesn't report native source-graph recall directly. Add one native source-graph line to the next packet so recall is measured, not inferred.

## Nits

- `build.rs` imports `HashSet` only for `flatten_native_neighbor_slots`; if that's the sole use, a small `Vec` + `contains` is likely faster at M-scale and keeps the `use` list tighter.
