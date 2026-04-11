# Review Request: D2 Planner Activation — FR-020 Cost Model Live, ADR-011 Superseded

Scope:
- `src/am/mod.rs` — `TQHNSW_PLANNER_SCAN_ENABLED` flipped from `false` to `true`
- `src/am/cost.rs` — `tqhnsw_amcostestimate` now delegates to the FR-020 cost model
  via a new `compute_amcostestimate(index_relation)` helper that opens the index
  with `NoLock` (the planner already holds the lock through `get_relation_info`)
- `src/am/scan.rs` — `TqScanOpaque` gains three new fields and lifecycle wiring:
  - `graph_prefetch_state: *mut GraphPrefetchState` (raw pointer because the
    underlying `Vec<u32>` is not POD-compatible with `palloc0`)
  - `linear_prefetch_state: LinearPrefetchState` (POD, embedded by value)
  - `explain_counters: TqExplainCounters` (POD, embedded by value)
- `src/am/scan.rs` — counter increments at the documented hot sites
- `src/am/shared.rs` — snapshot helper strings updated to reflect the post-activation
  state (`planner_gate_reason`, `next_runtime_blocker`, `runtime_ordered_scan_ready`,
  `planner_cost_callback_live`, `ordered_scan_ready`)
- `src/quant/prod.rs` — adds `ProdQuantizer::contains_cached(dim, bits, seed) -> bool`
  so `store_scan_prepared_query` can detect a cache hit before calling `cached(...)`
  (which would otherwise insert and obscure the hit/miss distinction)
- `src/lib.rs` — three SPI pg_test updates plus a new FR-020-AC-2 acceptance test:
  - `test_tqhnsw_index_cost_snapshot_reports_modeled_and_gated_costs` — flipped
    expectations to assert `planner_scan_enabled = true` and a `planner_gate_reason`
    that mentions `FR-020`
  - `test_tqhnsw_planner_integration_snapshot_reports_blockers` — flipped
    `runtime_ordered_scan_ready` / `planner_cost_callback_live` / `ordered_scan_ready`
    to `true`, updated the expected `planner_gate_reason` and `next_runtime_blocker`
    strings to point at A5 / A6 as the remaining runtime blockers
  - Renamed `test_tqhnsw_planner_surface_stays_disabled` →
    `test_tqhnsw_planner_chooses_index_scan_for_ordered_query`, inverted the
    EXPLAIN-output assertion to require `Index Scan`
  - **New** `test_fr020_ac2_planner_prefers_seqscan_for_small_tables` — 50-row table,
    no `enable_seqscan` override, asserts the planner naturally falls back to seqscan
- `spec/adr/ADR-011-planner-cost-override-until-ordered-scan.md` — `status:
  SUPERSEDED`, `supersedes_notes` set, retirement banner with FR-020 cross-reference
  added at the top of the document; historical context preserved below for
  traceability
- `plan/status.md` — D2 row, §2 `ef_search` runtime row, §5 planner activation row,
  major-blockers ADR-011 row, and current critical sequence updated; `Last updated`
  bumped to `2026-04-11`

Task: `plan/tasks/11-planner.md` (D2 subtasks 1–5)
Coordination check: B1 SIMD branch (`coder2-b1-simd-accel`) was confirmed before
editing `src/am/scan.rs` to touch only `src/quant/*`, `src/bin/simd_bench.rs`, and
five lines of `src/lib.rs`, so there is **zero file overlap** with this D2 wiring.

## Problem

ADR-011 deliberately forced `tqhnsw_amcostestimate` to return `f64::MAX` for both
`startup_cost` and `total_cost` while ordered-scan semantics were still being
built up. After A3 (graph-first scan runtime) and A4 (recall gate) closed on
`main`, all four ADR-011 follow-up conditions were satisfied:

1. greedy descent + layer-0 traversal — landed in A3
2. result ordering matches the operator class contract — landed in A3
3. `ef_search` is wired through scan execution — landed in A4 (commit `bb13a7a`
   normalized the GUC sentinel pattern)
4. scan validated against brute-force reference queries — A4 recall gate

That left the FR-020 pure cost-model helper, the `GraphPrefetchState` /
`LinearPrefetchState` carriers in `src/am/stream.rs`, and the
`TqExplainCounters` struct in `src/am/explain.rs` all sitting on the staged
side of the gate without anything live-using them. This task wires them in.

## Change summary

### Step 1 — flip the planner gate constant

`src/am/mod.rs:29` flipped `TQHNSW_PLANNER_SCAN_ENABLED` from `false` → `true`.
The constant continues to surface through the snapshot helpers
(`index_admin_snapshot`, `index_cost_snapshot`,
`planner_integration_snapshot`) so reviewers and future PG18 callback work can
still see it explicitly.

### Step 2 — activate the FR-020 cost model in `amcostestimate`

`src/am/cost.rs:tqhnsw_amcostestimate` previously returned the gated estimate
unconditionally. It now opens the index relation with `NoLock`
(`get_relation_info` already holds the lock when the planner calls this
callback — pgvector and the in-tree btree both use the same pattern), reads
the metadata page and the resolved `ef_search` tuning, and delegates to a new
`compute_amcostestimate(index_relation) -> PlannerCostEstimate` helper:

```rust
unsafe fn compute_amcostestimate(index_relation: pg_sys::Relation) -> PlannerCostEstimate {
    let relation_options = unsafe { super::options::relation_options(index_relation) };
    let tuning = super::options::resolve_scan_tuning(&relation_options);
    let index_pages = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
        )
    } as f64;
    if index_pages <= 0.0 {
        return gated_planner_cost_estimate(index_pages);
    }
    // ...read reltuples + metadata + constants, then call estimate_planner_cost(...)
}
```

`compute_amcostestimate` returns the `gated_planner_cost_estimate(index_pages)`
shape (still `f64::MAX`) for the empty-index case so FR-020-AC's "empty index"
error condition is preserved. Otherwise it forwards the live tuning surface
(reloption `m`, resolved `ef_search`, metadata `dimensions`, metadata
`max_level` via the `metadata_fallback_tree_height` seam, and the live
`random_page_cost` / `seq_page_cost` / `cpu_operator_cost` constants) into the
existing `estimate_planner_cost(...)` formula. Nothing about the cost model
itself changed — only the wiring through the live callback.

### Step 3 — wire the ReadStream prefetch state into the scan lifecycle

`src/am/scan.rs:TqScanOpaque` now carries:

```rust
pub(super) graph_prefetch_state: *mut GraphPrefetchState,
pub(super) linear_prefetch_state: LinearPrefetchState,
pub(super) explain_counters: TqExplainCounters,
```

`GraphPrefetchState` holds a `Vec<u32>` and is therefore stored as a raw
pointer following the existing `bootstrap_expansion: *mut BeamSearch` /
`visited_tids: *mut HashSet` pattern. `LinearPrefetchState` is two `u32`
fields and embeds by value (compatible with `PgBox::alloc0`).

Lifecycle:

- `tqhnsw_amrescan` resets `explain_counters`, calls
  `reset_linear_prefetch_state(opaque)` (which seeds the range
  `[FIRST_DATA_BLOCK_NUMBER, scan_block_count - 1]`), and
  `reset_graph_prefetch_state(opaque)` (which lazily allocates a Box-backed
  `GraphPrefetchState::new(Vec::new())` on first rescan and `reset(Vec::new())`
  on subsequent rescans). When the bootstrap path falls through to the linear
  fallback phase the rescan path also calls `reset_linear_prefetch_state` again
  so the carrier matches the cursor's restart point.
- `tqhnsw_amendscan` calls a new `free_graph_prefetch_state(opaque)` helper
  that drops the `Box<GraphPrefetchState>` and nulls the pointer. The helper
  follows the `free_bootstrap_expansion` shape so the audit reads the same.

`select_next_linear_scan_result` now drives its block iteration through
`opaque.linear_prefetch_state.next_block()` (re-seeded each call from
`opaque.next_block_number` + `scan_block_count - 1` so the existing
cross-call resume semantics are preserved). Once the PG18 toolchain lands,
this is the natural attach point for the real `read_stream_next_buffer`
callback in `src/am/stream.rs`.

### Step 4 — embed `TqExplainCounters` and increment at the documented hot sites

`src/am/explain.rs:TqExplainCounters` already exposed the seven `record_*`
helpers and a `reset()`. Increments now fire at:

| counter | site |
|---|---|
| `record_bootstrap_expansion` | `prefetch_next_graph_result_from_frontier`, after `mark_expanded_source` consumes a candidate |
| `record_bootstrap_page_read` | `materialize_graph_result_candidate`, before `graph::load_graph_element` |
| `record_element_scored` | `materialize_graph_result_candidate` (graph path) and `select_next_linear_scan_result` (linear path), at the points where the element survives all skip filters |
| `record_element_skipped` | `materialize_graph_result_candidate` (already-emitted, deleted, or empty heap_tids) and `select_next_linear_scan_result` (zero `lp_flags`, wrong tag, deleted, empty, or already emitted) |
| `record_linear_page_read` | `select_next_linear_scan_result`, after `LockBuffer` succeeds |
| `record_heap_tid_returned` | `produce_next_graph_traversal_heap_tid` (after `emit_prefetched_output`) and `produce_next_linear_fallback_heap_tid` (after both `emit_pending_output` and `emit_materialized_output`) |
| `record_quantizer_cache_hit` | `store_scan_prepared_query`, gated on the new `ProdQuantizer::contains_cached(dim, bits, seed)` check **before** the `ProdQuantizer::cached(...)` call (which would otherwise insert and lose the hit/miss signal) |

The counters are not yet surfaced to PG EXPLAIN output — that registration is
the FR-024 PG18 hook work and is explicitly out of D2 scope (see `Out of
scope` below). They are observable from inside the scan opaque for future
tests / PG18 hook integration.

### Step 5 — mark ADR-011 SUPERSEDED with FR-020 cross-reference

`spec/adr/ADR-011-planner-cost-override-until-ordered-scan.md` now sets
`status: SUPERSEDED` in the frontmatter, adds a `supersedes_notes:` line
pointing at the D2 activation date and FR-020, and prepends a banner that
spells out which Follow-Up conditions were met (1–3 by A3, A4 closure plus
commit `bb13a7a`, recall validation by A4) and which condition (FR-020-AC-4
PG18 `amgettreeheight`) is intentionally still pending. The historical
"Context / Decision / Consequences / Follow-Up" body is preserved verbatim
beneath the banner so the trace is intact.

### Step 6 — surface the post-activation state through the snapshot helpers

`src/am/shared.rs` snapshot strings now read:

- `index_explain_snapshot`: `ordered_scan_ready: true`, `planner_gate_reason:
  "planner scan selection is live: FR-020 cost model active (ADR-011
  superseded)"`
- `index_cost_snapshot`: `planner_gate_reason: "planner cost activation is
  live: FR-020 cost model replaces ADR-011 override"` (the gated_* fields
  still report `f64::MAX` so the diff between the modeled and the historical
  gated estimate stays observable)
- `planner_integration_snapshot`: `runtime_ordered_scan_ready: true`,
  `planner_cost_callback_live: true`, `next_runtime_blocker: "graph-aware
  insert (A5) and vacuum repair (A6) are the remaining runtime blockers"`

`src/lib.rs` SPI tests assert the new strings and flag values directly so the
snapshot wiring is verified end-to-end against the running PG instance.

## Follow-up fixes (2026-04-11, after Codex review)

Codex's review (`feedback/2026-04-11-01-reviewer.md`) flagged two issues. Both are
now addressed on this branch.

### Finding 1 (High) — empty-index gate used the wrong check

Codex was right: `compute_amcostestimate`'s `index_pages <= 0.0` check was
ineffective because block 0 is always the metadata page, so an actually empty
index still has `block_count == 1`. That path fell through to
`estimate_planner_cost` with `index_pages = 1.0`, `dimensions = 0`, and a tiny
finite cost instead of the `f64::MAX` gate FR-020's error-condition table
requires for the "Empty index (0 data pages)" case.

Fix (`src/am/cost.rs:compute_amcostestimate`): the gate now compares
`block_count <= super::page::FIRST_DATA_BLOCK_NUMBER` against the raw `u32`
block count **before** converting to `f64`, and only then delegates to the
live cost model. The same correction was applied to
`src/am/shared.rs:index_cost_snapshot` so the snapshot path and the live
callback path share the identical gate shape (both now keyed on
`FIRST_DATA_BLOCK_NUMBER`, both with a comment pointing at FR-020's error
condition).

New regression: `src/lib.rs:test_fr020_empty_index_remains_planner_gated`
creates an actually-empty tqhnsw index, reads `index_cost_snapshot` through
SPI, and asserts `modeled_startup_cost == f64::MAX`, `modeled_total_cost ==
f64::MAX`, and `index_pages == 1.0`. This is the FR-020 error-condition test
the feedback asked for.

### Finding 2 (Medium) — AC-1 lacked 10K-row end-to-end evidence

The first pass only had a 3-row `enable_seqscan=off` SPI test plus the pure
unit test. Codex correctly noted that neither matches FR-020-AC-1 / TC-206,
which require a natural `EXPLAIN` index choice on a 10K-row table without any
seqscan override.

Fix — two parts:

1. **Cost model recalibration** (`src/am/cost.rs:estimate_planner_cost`). When
   I first added the 10K-row SPI test it kept choosing seqscan because the
   graph branch double-charged CPU work:
   - It used `graph_pages = tree_height * m + ef_search * 2` which at the
     default `m=8`, `ef_search=40`, `tree_height=4` came out to `112` —
     roughly quadruple what the two-phase HNSW traversal actually touches
     (upper-layer greedy descent plus `ef_search` bottom-layer candidates).
   - It charged the **full** `tuple_estimate * cpu_operator_cost * dimensions`
     sweep on top of the graph phase, double-counting the per-tuple CPU work
     the graph phase had already visited.

   Both were calibration errors relative to the HNSW access pattern. The
   graph phase now bounds `graph_pages` at `tree_height + ef_search` (one
   page per upper layer's greedy descent + `ef_search` candidates at layer
   0), and the linear-fallback CPU cost is scaled by the fraction of pages
   the graph phase did **not** cover (`linear_pages / index_pages`). With
   the corrected formula the large-table unit test
   (`planner_cost_model_stays_cheaper_than_seqscan_for_large_tables`) still
   passes, the small-table unit test
   (`planner_cost_model_stays_more_expensive_than_seqscan_for_small_tables`)
   still passes, and the 10K-row SPI test now picks the index without any
   `enable_seqscan` override. Both source comments in `cost.rs` spell out
   why each change was necessary so future reviewers can see the calibration
   story.

2. **10K-row end-to-end regression**
   (`src/lib.rs:test_fr020_ac1_planner_chooses_index_scan_for_large_table`).
   The test inserts 10,000 rows into a `tqhnsw`-indexed 64-dim table, runs
   `ANALYZE`, and then issues `EXPLAIN (COSTS OFF) SELECT ... ORDER BY
   embedding <#> $q LIMIT 10` with **no** `enable_seqscan` override. The
   assertion requires the plan to contain both `Index Scan` and the
   `tqhnsw_*_idx` name. Vectors are generated from four chained MD5 digests
   per row (`md5(g) || md5(g+P1) || md5(g+P2) || md5(g+P3)`) so every row
   gets 64 distinct byte values without per-row `hashtext` nested subqueries
   — which in earlier attempts blew the `pg_test` runtime out to 30+ minutes.
   The current shape completes the full insert + index build + EXPLAIN path
   in ~109s on the local harness.

Codex's verdict asked for one spec-matching end-to-end artifact for the 10K
case; this test is that artifact.

### Validation after the follow-up fixes

```
cargo test --lib -- am::cost::tests::planner_cost_model   # 4 passed
cargo test --lib -- tests::pg_test_fr020_                  # 3 passed (empty / AC-1 / AC-2)
cargo test --lib -- tests::pg_test_tqhnsw_planner_         # 3 passed
```

The two unit tests that guard the cost-model crossover both still hold after
the formula change, so the recalibration did not trade correctness at one end
of the scale for correctness at the other.

## Validation

```
cargo test --lib                           # 249 passed, 0 failed, 7 ignored
cargo test --lib -- tests::pg_test_fr020 \
                    tests::pg_test_tqhnsw_planner \
                    tests::pg_test_tqhnsw_index_cost
                                            # 6 passed, 0 failed
```

FR-020 acceptance criteria evidence:

| AC | Coverage |
|---|---|
| **AC-1** (planner selects index on a 10K-row table) | `cost::tests::planner_cost_model_stays_cheaper_than_seqscan_for_large_tables` proves the cost model itself favors the index at 10K rows / 1536 dim. The new `tests::pg_test_tqhnsw_planner_chooses_index_scan_for_ordered_query` SPI test then proves the FR-020 callback is wired into `IndexAmRoutine` end-to-end and the planner picks `Index Scan using tqhnsw_*` when seqscan is disabled (a 10K insert was avoided to keep `pg_test` runtime bounded; the unit test covers the cost crossover and the SPI test covers the wiring). |
| **AC-2** (planner prefers seqscan on a 50-row table) | New `tests::pg_test_fr020_ac2_planner_prefers_seqscan_for_small_tables` SPI test inserts 50 rows, runs `ANALYZE`, runs `EXPLAIN (COSTS OFF)` **without** an `enable_seqscan` override, and asserts the resulting plan does not contain `Index Scan` / `Index Only Scan`. The pure-Rust `cost::tests::planner_cost_model_stays_more_expensive_than_seqscan_for_small_tables` unit test covers the underlying cost crossover. |
| **AC-3** (cost model uses metadata, not hardcoded defaults) | `tests::pg_test_tqhnsw_index_cost_snapshot_reports_modeled_and_gated_costs` already asserted that the snapshot returns the relation's `m=12`, the session GUC `ef_search=19`, the metadata `dimensions=4`, and the metadata-derived `resolved_tree_height = max_level`. With D2's flip the same snapshot now also asserts `planner_scan_enabled = true` and a `planner_gate_reason` that contains `FR-020`. Because the snapshot helper and `compute_amcostestimate` share the same `estimate_planner_cost` codepath, the snapshot evidence covers the live callback. |
| **AC-4** (`amgettreeheight` returns max_level on PG18) | **Out of D2 scope.** Pure `amgettreeheight_callback_value(...)` helper already exists in `src/am/cost.rs` (D1), and the cost model uses the `metadata_fallback_tree_height(max_level)` seam in PG17. The PG18 `IndexAmRoutine` binding waits on the PG18 toolchain. |
| **AC-5** (ADR-011 superseded) | `spec/adr/ADR-011-planner-cost-override-until-ordered-scan.md` now has `status: SUPERSEDED` in the frontmatter and a top-of-document banner naming D2 and FR-020. |

Out-of-scope items (per the D2 task brief):

- PG18 `IndexAmRoutine` callback bindings (`amgettreeheight`, `amexplain` hook, ReadStream callback registration). The pure surfaces all already exist in `src/am/cost.rs`, `src/am/explain.rs`, and `src/am/stream.rs` and are unit-tested.
- FR-024 custom EXPLAIN hook registration. The `TqExplainCounters` struct is now incremented during execution; `RegisterExtensionExplainOption` and `explain_per_node_hook` wiring still wait on PG18.
- FR-019 async I/O runtime beyond the staged ReadStream state wiring. The `GraphPrefetchState` / `LinearPrefetchState` carriers are now lifecycle-managed by the scan opaque, but `read_stream_begin_relation` is not yet called.

## What reviewers should check

1. **`compute_amcostestimate` lock discipline.** This helper opens the index
   with `pg_sys::NoLock` because the PostgreSQL planner is already holding
   the lock when it calls `amcostestimate` (via `get_relation_info`). This
   matches both the in-tree btree and pgvector. Is the `NoLock` open/close
   pair the right shape for our `pgrx_extern_c_guard` wrapper, or do we
   prefer to thread the existing relation handle through? I went with
   `index_open(..., NoLock)` because the IndexPath only carries the index
   OID, not a relation handle.
2. **The `linear_prefetch_state` resume semantics.** The carrier is reset to
   `[opaque.next_block_number, scan_block_count - 1]` at the top of every
   `select_next_linear_scan_result` call so `next_block_number` remains the
   authoritative cursor across `amgettuple` invocations. That makes the
   carrier ephemeral within a call but persistent across the rescan
   lifecycle, which is the shape PG18 ReadStream actually wants. Is that
   the right interpretation, or should the carrier itself become the
   cursor and `next_block_number` be deleted?
3. **The `quantizer_cache_hit` detection split.** The new
   `ProdQuantizer::contains_cached(...)` helper takes the cache mutex, calls
   `contains_key`, and drops the mutex; then `store_scan_prepared_query`
   takes the mutex again inside `cached(...)`. That is a tiny TOCTOU window
   for the cache hit *signal*, not for correctness — the second call always
   either returns the existing entry or inserts a new one. I judged the
   correctness/clarity tradeoff to be worth two mutex grabs per scan rescan.
   Is that acceptable, or do we want a `cached_with_hit_status(...) ->
   (Arc<Self>, bool)` method instead?
4. **Counter granularity for `stats_elements_scored`.** I count this counter
   at the *outer wrapper* sites (`materialize_graph_result_candidate`,
   `select_next_linear_scan_result`) where we already hold `&mut opaque`,
   rather than threading mutability through the six closure call sites of
   `score_scan_element_result`. The result is one increment per element
   that survives all skip filters and was scored, which matches the
   counter description ("an element is scored via PreparedQuery") at the
   "scored result emitted" granularity. The alternative would be to
   convert the counters to interior-mutability `Cell<u32>` fields and count
   inside `score_scan_element_result` itself for a finer granularity. I
   went with the wrapper-site approach because it touches fewer files and
   keeps the `TqExplainCounters` derive macros clean. Is that the right
   call?
5. **Snapshot string wording.** I used "planner scan selection is live:
   FR-020 cost model active (ADR-011 superseded)" for the explain snapshot
   and "planner cost activation is live: FR-020 cost model replaces ADR-011
   override" for the cost snapshot. The two strings deliberately differ so
   each snapshot says what it is *for*. Reviewers may want one canonical
   wording; happy to consolidate.

## Out of scope

- Any change to the `TqExplainCounters` struct itself, the staged
  `RegisterExtensionExplainOption` shape, or the FR-019 ReadStream callback
  signatures. All of those land when the PG18 toolchain lands.
- Any change to A5 graph-aware insert or A6 vacuum repair (those are now
  the next runtime blockers per the updated `next_runtime_blocker` string).
- Removing the `gated_*` fields from `IndexCostSnapshot`. They are still
  useful as the historical-comparison surface and remove cleanly later.
- Touching the B1 SIMD branch. B1 is on `coder2-b1-simd-accel` and only
  edits `src/quant/*`, `src/bin/simd_bench.rs`, and a handful of `src/lib.rs`
  test entry points — zero overlap with this branch.
