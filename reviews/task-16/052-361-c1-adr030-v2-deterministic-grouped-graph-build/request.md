# Review Request: C1 ADR-030 V2 Deterministic Grouped Graph Build

## Context

Packet `360` produced the first credible canonical `50k` grouped-v2 operating
point on the binary traversal lane:

- grouped-v2 runtime stayed on the ADR030 v2 storage/layout path
- live rerank widened to `window=64`
- canonical `50k` grouped `window=64` reached `0.860 Recall@10 @ ef=64`
  in `1.007ms`, versus scalar `0.860 @ 1.398ms`

But the packet also exposed an unexplained build-surface gap:

- an isolated grouped-only `50k` copy built a much stronger grouped index than
  the canonical grouped index
- both tables had the same logical row order by `id`
- the branch still could not tell whether the gap came from table layout,
  planner surface, or graph build instability

That ambiguity blocked the next decision. If grouped quality was drifting with
the build itself, then packet `360`'s direct frontier was not a stable
operating point yet.

## Problem

The strongest clue was that the grouped-only copy and the canonical table had
the same row order but very different grouped recall. I checked the actual HNSW
build path and found the deeper issue:

1. `src/am/build.rs` inserts build tuples into `hnsw_rs::Hnsw` in stable heap
   tuple order
2. upstream `hnsw_rs` assigns point levels from `StdRng::from_os_rng()`
3. rebuilding the same grouped index on the same canonical `50k` table with the
   same reloptions produced materially different recall curves

Representative pre-fix canonical `50k` grouped curves on the same table:

| build | ef=64 | ef=128 | ef=200 | ef=320 |
|------|------:|-------:|-------:|-------:|
| old canonical grouped build | `0.844` | `0.860` | `0.870` | `0.874` |
| later trial grouped build | `0.874` | `0.900` | `0.902` | `0.904` |

That is graph lottery, not a runtime tuning signal. Until the build seed is
stable, the branch cannot make meaningful recall or latency claims about the
grouped lane.

## Planned Slice

Batch the stability fix and the remeasurement together:

1. vendor `hnsw_rs` so the branch owns the layer-RNG seam
2. add seeded constructors to the vendored HNSW layer generator,
   point indexation, and top-level `Hnsw`
3. derive a deterministic build seed from persisted build metadata plus a small
   domain tag, separately for scalar-code and source-vector builds
4. route both tqvector graph builders through the seeded HNSW constructor
5. add unit coverage proving repeated builds from the same input state produce
   identical `HnswBuildNode` output for both scalar and source builds
6. reinstall the scratch extension and rebuild canonical grouped `50k` indexes
   multiple times on the same table to verify the direct recall frontier stops
   drifting

This slice intentionally does not change traversal scoring again. The runtime
lane stays:

`grouped-v2 storage + binary traversal score + live window=64`

## Implementation

Updated:

- `Cargo.toml`
- `src/am/build.rs`
- `vendor/hnsw_rs/src/hnsw.rs`

Concrete changes:

1. vendored `hnsw_rs` into `vendor/hnsw_rs` and pointed the crate dependency at
   the local copy
2. added `new_with_seed(...)` constructors in vendored `hnsw_rs` for:
   - `LayerGenerator`
   - `PointIndexation`
   - `Hnsw`
3. kept the existing `new(...)` constructors as thin wrappers that still choose
   a fresh random seed when callers do not care about determinism
4. changed `build_hnsw_graph(...)` and `build_hnsw_graph_from_source(...)` to
   call `Hnsw::new_with_seed(...)`
5. added `deterministic_hnsw_build_seed(...)` in `src/am/build.rs`, which
   mixes:
   - recorded build seed
   - dimensions
   - bits
   - tuple count
   - `m`
   - `ef_construction`
   - a domain tag distinguishing scalar-code vs source-vector graph builds
6. added a small local `splitmix64(...)` mixer to produce the final HNSW layer
   seed
7. derived `PartialEq, Eq` for `HnswBuildNode` and added unit tests proving
   repeated builds return identical graph-node output for:
   - scalar-code builds
   - source-vector builds

## Validation

Compile and lint validation on the final code:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

The required lib-test commands were run and still hit the same local linker
environment failure on this workstation:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Both fail during final test-binary link with unresolved PostgreSQL symbols such
as `CurrentMemoryContext`, `PG_exception_stack`, `error_context_stack`,
`CopyErrorData`, `pfree`, and `errstart`.

Scratch runtime validation:

1. install current pg17 scratch build:
   - `./scripts/install_adr030_pg17_pg_test.sh`
2. restart scratch on the verified grouped runtime lane:
   - `./scripts/restart_adr030_scratch.sh --window 64 --grouped-score-mode binary`
3. refresh scratch debug helpers:
   - `./scripts/refresh_adr030_scratch_debug_helpers.sh`
4. verify runtime settings:
   - grouped scan gate enabled
   - build gate enabled
   - `grouped_scan_window = 64`
   - `grouped_scan_score_mode = binary`

## Measurements

All measurements below use:

- corpus: `tqhnsw_real_50k_corpus`
- queries: `tqhnsw_real_50k_queries_50`
- runtime: `window=64`, grouped score mode `binary`

### Same-table rebuild reproducibility

Built two fresh grouped indexes on the same canonical table with the same
reloptions:

- `tqhnsw_real_50k_grouped_m8_det_a_idx`
- `tqhnsw_real_50k_grouped_m8_det_b_idx`

Direct recall sweeps:

| index | ef=64 | ef=128 | ef=200 | ef=320 |
|------|------:|-------:|-------:|-------:|
| `det_a` | `0.904 @ 1.120ms` | `0.910 @ 1.561ms` | `0.912 @ 2.292ms` | `0.914 @ 3.656ms` |
| `det_b` | `0.904 @ 1.135ms` | `0.910 @ 1.848ms` | `0.912 @ 2.854ms` | `0.914 @ 4.071ms` |

The recall curve and the mean returned-score aggregate matched exactly between
the two builds. Only wall-clock timing drifted slightly run-to-run, which is
expected.

That is the proof the packet needed: same-table grouped rebuilds no longer
wander onto different graphs.

### Canonical grouped index after deterministic reindex

After `reindex index tqhnsw_real_50k_grouped_m8_idx`, the canonical grouped
index landed on the same frontier as the fresh deterministic trial builds:

| path | ef=64 | ef=128 | ef=200 | ef=320 |
|------|------:|-------:|-------:|-------:|
| canonical grouped | `0.904 @ 1.177ms` | `0.910 @ 1.601ms` | `0.912 @ 2.365ms` | `0.914 @ 3.561ms` |
| scalar baseline | `0.876 @ 2.141ms` | `0.890 @ 3.202ms` | `0.894 @ 4.775ms` | `0.898 @ 7.029ms` |

Interpretation:

- the packet `360` canonical grouped frontier was being dragged down by random
  graph construction, not by the binary traversal lane itself
- on the stabilized grouped build, canonical `50k` grouped now beats scalar on
  both recall and direct latency across the measured `ef=64..320` sweep
- the best direct comparison in this batch is:
  - grouped canonical `ef=128`: `0.910 Recall@10 @ 1.601ms`
  - scalar `ef=128`: `0.890 Recall@10 @ 3.202ms`

This does not solve the shared-table planner preference issue from packet
`360`, but it does remove the biggest source of runtime uncertainty. The branch
now has a stable canonical grouped build surface for future SQL/planner
measurement work.

## Risk / Follow-up

Remaining follow-up after this packet:

1. verify whether the same deterministic-build improvement holds on the other
   real-corpus grouped indexes used in staging (`10k`, `1k`)
2. return to the planner lane with the now-stable canonical grouped build,
   because shared-table SQL still prefers the scalar index by default
3. decide whether ADR030 should explicitly depend on deterministic graph builds
   as part of the durable runtime contract, since the grouped lane is sensitive
   enough that random level assignment materially changed corpus-scale recall

The immediate next batch should stay focused on measurement honesty:

- keep the deterministic build
- keep the binary traversal lane
- use the stabilized canonical grouped index to answer the planner/SQL surface
  question cleanly
