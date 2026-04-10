# A4 Build Hierarchy Collapse Audit

## Status

Draft progress review while the deterministic `10k` fixture is rebuilding on the hierarchy-fix tree.

A4 remains open and blocked, but the current lane found a materially stronger explanation for the live graph failure than the earlier “upper hierarchy is too narrow” read.

## What Changed Since Review 211

Review 211 concluded:

- layer-0 search can recover exact quantized recall with enough good seeds
- oracle top-level `k` seeding looked strong
- the next likely lane was upper-hierarchy reachability or multi-entry metadata

That interpretation is now being revised, because a new coverage probe exposed that the persisted graph may not have had a real upper hierarchy at all.

## New Probe Added

I added a deterministic top-level seed coverage summary on the fixed `10k` fixture:

- counts all persisted “top-level” nodes
- counts the nodes reachable from the stored entry point at `metadata.max_level`
- counts unique oracle top-level seed ids across the fixed query set
- records the fraction of oracle seed slots reachable from the stored entry point

SQL surface:

- `tests.tqhnsw_graph_scan_recall_top_level_seed_coverage_rel(...)`

## Key Result On The Pre-Fix Tree

On deterministic `10k`, `m=8`, `ef_search=128`, `50` fixed queries, `k=10`:

- `top_level_node_count = 10000`
- `reachable_top_level_node_count = 9179`
- `unique_oracle_seed_id_count = 491`
- `reachable_unique_oracle_seed_id_count = 450`
- `reachable_oracle_seed_slot_fraction = 0.916`
- `fully_reachable_queries = 23`

Top frequent oracle seed ids were basically flat, with counts of only `1-2`.

## Why This Matters

`top_level_node_count = 10000` is the smoking gun.

That means the probe was not actually sampling a sparse upper layer. It was effectively operating on a graph whose persisted `metadata.max_level` was `0`, which makes the earlier oracle-`k` interpretation misleading:

- “top-level oracle `k=10` reaches exact” no longer means “the real upper hierarchy contains the right entry nodes”
- it may simply mean “if you score the entire layer-0 population and seed from the best 10 points, layer-0 search works”

That is still useful, but it is not the same claim.

## Root Cause Found In Build

The likely cause is in `src/am/build.rs`.

Both build paths used:

- `hnsw.get_point_indexation().get_layer_iterator(0)`

In `hnsw-rs`, that iterator walks **only layer 0**, not all points across all layers.

So the persisted build logic was:

1. initializing every node as `level = 0`
2. only overwriting nodes that happened to be visible through layer-0 iteration
3. leaving higher-layer points effectively collapsed back to `level = 0`

That directly explains the `top_level_node_count = 10000` result.

## Fix Applied Locally

I changed both build paths to iterate the full point indexation:

- `for point in hnsw.get_point_indexation()`

instead of:

- `for point in hnsw.get_point_indexation().get_layer_iterator(0)`

Files touched:

- `src/am/build.rs`

## Validation So Far

Targeted regression still passes on the hierarchy-fix tree:

```bash
cargo test --manifest-path /home/peter/dev/tqvector/Cargo.toml --no-default-features --features 'pg17 pg_test' tests::pg_test_tqhnsw_graph_first_scan_emits_distance_sorted_scores -- --exact --nocapture
```

I have not yet run the full required validation lanes on this in-progress tree.

## Post-Fix Results

After fixing the build iterator bug and rebuilding the deterministic `10k` fixture:

- index block counts moved from `1251 / 1369` to `1260 / 1379`
- the persisted top layer stopped looking like “every node”

Corrected top-level seed coverage on deterministic `10k`, `m=8`, `ef_search=128`, `50` fixed
queries, `k=10`:

- `top_level_node_count = 3`
- `unique_oracle_seed_id_count = 3`
- top oracle seed ids: `{1969, 9001, 9404}`
- top oracle seed query counts: `{50, 50, 50}`

This is the corrected interpretation of review 211’s oracle-k result:

- the real top layer is tiny, not 10,000-wide
- the useful top-level seed set is also tiny and query-stable

So fixed build-time entry sets remain viable in principle, but the hierarchy bug had to be fixed
before that conclusion meant anything.

## Carrydown Rerun On The Corrected Hierarchy

I then re-ran the previously failed “carry an upper-layer beam down into layer-0” idea on the
corrected hierarchy.

Result on deterministic `10k` graph gate:

- corrected hierarchy only, live graph:
  - `(m=8, ef=40)`: `12.7%`
  - `(m=8, ef=128)`: `27.6%`
  - `(m=8, ef=200)`: `35.7%`
  - `(m=16, ef=200)`: `60.6%`

- corrected hierarchy + upper-layer carrydown:
  - `(m=8, ef=40)`: `16.5%`
  - `(m=8, ef=128)`: `36.4%`
  - `(m=8, ef=200)`: `46.8%`
  - `(m=16, ef=200)`: `65.5%`

So the carrydown path is **not** a dead end on the corrected hierarchy. It is the first
directional live improvement after the hierarchy-collapse fix, though it is still far below the
A4 gate.

## Expected Decision Point

The hierarchy collapse was a real bug, but not the only bug.

Current read:

1. Build hierarchy collapse was materially harming the graph.
2. Fixing it restored a tiny, stable real top layer.
3. Carrying an upper-layer beam down now helps materially.
4. A4 still fails badly, so more seed acquisition / carrydown work is still needed.

## Failed / Superseded Interpretations To Keep On Record

- Review 211’s oracle-`k` result was useful, but its “upper hierarchy contains enough good seeds” interpretation is superseded by the discovery that the persisted hierarchy likely collapsed to level 0.
- The failed multi-seed carrydown runtime experiment remains valid as negative evidence on the pre-fix tree, but it should not be overinterpreted until rerun on the corrected hierarchy.
- After rerun on the corrected hierarchy, that same carrydown idea is no longer a failed lane; it is now a live improving lane.
