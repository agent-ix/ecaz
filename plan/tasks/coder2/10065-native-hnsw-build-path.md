# Task 10065: Native HNSW build path

## Context

`hnsw_rs` was adopted to bootstrap graph construction before tqvector had its
own HNSW implementation. INSERT is now fully native (`src/am/insert.rs`), but
BUILD still delegates to `hnsw_rs`:

- `src/am/build.rs:1218` — `build_hnsw_graph` constructs an `Hnsw` with
  `Hnsw::new_with_seed`, inserts each heap tuple serially, then walks
  `get_point_indexation()` and repacks `hnsw_rs::hnsw::Neighbour` lists into
  tqvector's slot layout.
- `src/am/build.rs:1276` — `build_hnsw_graph_from_source` does the same thing
  on the source-vector path.
- `src/am/build.rs:1401, 1422, 1453` — `flatten_point_neighbors`,
  `pack_point_neighbor_slots`, and `fill_point_neighbor_layer_slots` take
  `&[Vec<hnsw_rs::hnsw::Neighbour>]` arguments, leaking the crate's types into
  internal signatures.

The lifecycle split (`hnsw_rs` for BUILD, native for INSERT) has two practical
problems captured in ADR-034:

1. BUILD and INSERT can drift on level assignment, neighbor selection, and
   distance dispatch for the same row stream.
2. BUILD pays avoidable overhead: codes are re-decoded through the `Distance`
   trait wrapper rather than scored by the SIMD kernels INSERT and scan
   already use; neighbors are allocated as `Vec<Vec<Neighbour>>` per point and
   repacked; insertion is serial and does not compose with the heap-scan
   worker model targeted by FR-021.

ADR-042 (PROPOSED) decides to retire `hnsw_rs` from the production BUILD path
and keep it only as a test-only recall oracle. This task implements that ADR.

## Problem

Two HNSW implementations coexist inside tqvector. They must produce
behaviorally equivalent graphs for the same seed and row stream, but they
don't share code. Every backlink-pruning or neighbor-selection fix that lands
in INSERT has no counterpart in BUILD, and the `hnsw_rs`-shaped types still
shape the BUILD signatures.

## Proposed Fix

Implement a native builder in `src/am/build.rs` that reuses the HNSW primitives
already owned by INSERT, and remove `hnsw_rs` from every path under `src/am/`.

### Native builder

Add a builder that accepts `&BuildState` and returns `Vec<HnswBuildNode>` with
the same shape the existing flush path expects. The builder must:

- sample insert levels using tqvector's seeded level sampler (the same source
  INSERT uses), not `hnsw_rs`'s internal RNG
- perform forward-neighbor discovery and backlink pruning using the rules in
  `src/am/insert.rs` — shared helpers, not parallel copies
- score candidates via the quantized scoring kernels scan and INSERT already
  use, with no `Distance` trait wrapper and no per-comparison re-decode
- write neighbor slots directly into the final slot layout used by
  `HnswBuildNode`, without an intermediate `Vec<Vec<Neighbour>>`
- be deterministic for a fixed `(seed, dimensions, bits, options)` tuple
- support both the code-graph path (currently `build_hnsw_graph`) and the
  source-graph path (currently `build_hnsw_graph_from_source`)

### Parallelism

The builder must not foreclose FR-021. At minimum, structure the insertion
loop so that heap-scan workers can feed the builder in a way that matches the
planned parallel build model. A serial inner implementation is acceptable for
this task as long as the boundary is shaped to admit parallel feeding later;
landing actual parallelism is out of scope.

### Remove `hnsw_rs` from `src/am/`

After the native builder lands:

- drop `use hnsw_rs::...` lines from `src/am/build.rs`
- replace `&[Vec<hnsw_rs::hnsw::Neighbour>]` parameters in
  `flatten_point_neighbors`, `pack_point_neighbor_slots`, and
  `fill_point_neighbor_layer_slots` with a tqvector-owned neighbor type, or
  fold these helpers into the builder if they no longer have non-test callers
- remove the now-unused `BuildCodeDistance` / `BuildVectorDistance` adapters
- update the build.rs unit tests that currently construct
  `hnsw_rs::hnsw::Neighbour` values (`pack_point_neighbor_slots_preserves_layer_boundaries_with_padding`)
  to use the tqvector-owned neighbor type

### Keep `hnsw_rs` as a test oracle

`hnsw_rs` must remain available to `src/lib.rs` for the recall probes and the
`test_hnsw_rs_*_10k` tests. This means:

- move `hnsw_rs` out of the production dependency set and into
  `[dev-dependencies]` (or equivalent gating) so `src/am/` cannot accidentally
  re-import it
- keep the vendored crate under `vendor/hnsw_rs/` — do not remove

### Recall re-baseline

Re-run the existing recall probes and gates against the native builder before
claiming parity:

- `test_hnsw_rs_code_graph_recall_uniform_10k`
- `test_hnsw_rs_source_graph_recall_uniform_10k`
- `test_hnsw_rs_source_graph_recall_clustered_10k`
- `test_hnsw_rs_source_graph_recall_uniform_10k_m16_ef200`
- the four-config gate report used by the real `50k` harness (indirectly, via
  the rebuilt BUILD path)

Record the native-builder recall numbers alongside the `hnsw_rs` oracle
numbers in the review packet for this task. Gate acceptance on recall parity
within the tolerance documented by the existing NFR-001 targets.

## Scope

- `src/am/build.rs`
  - new native builder covering both code-graph and source-graph paths
  - `build_hnsw_graph` and `build_hnsw_graph_from_source` retargeted to the
    native builder
  - `hnsw_rs` imports and leaked types removed
  - unit tests updated to use the tqvector-owned neighbor type
- `src/am/insert.rs`
  - level sampler, neighbor-discovery, and backlink-pruning helpers made
    reusable by the builder (no behavior change on the INSERT side)
- `Cargo.toml`
  - `hnsw_rs` moved to `[dev-dependencies]` / test-only
- `src/lib.rs`
  - no change to `probe_hnsw_rs_*` or `test_hnsw_rs_*` tests beyond whatever
    the dependency-gating move requires

## What Is Not in Scope

- Removing `hnsw_rs` entirely or deleting the vendored crate.
- Landing parallel BUILD (FR-021). The builder must not foreclose it; actually
  implementing parallel feed is a separate task.
- Changing the persisted page/tuple layout produced by BUILD. The existing
  `HnswBuildNode` → page serialization path is unchanged.
- ADR-030 grouped-graph build. That work will land against the native builder
  but is tracked separately.
- Any user-visible SQL or GUC surface change.

## Expected Outcome

- `src/am/` has a single HNSW implementation shared between BUILD and INSERT.
- No production code path depends on `hnsw_rs`; `grep -R "hnsw_rs" src/am/`
  returns nothing.
- BUILD determinism is governed by tqvector's own seed and RNG, not the
  upstream crate's.
- Recall on the existing 10k and real-corpus probes is within the documented
  NFR-001 tolerance of the prior `hnsw_rs`-backed BUILD path.
- `hnsw_rs` remains usable as a test-only recall oracle for probes in
  `src/lib.rs`.

## References

- ADR-042: Native HNSW build path — retire hnsw_rs from production BUILD
- ADR-002: hnsw_rs has no delete — own the graph in Postgres pages
- ADR-003: Walk hnsw_rs graph and write to Postgres pages
- ADR-026: Live Insert Backlink Lock Ordering
- ADR-030: FastScan Grouped Subvector Scoring
- FR-008: HNSW build
- FR-016: HNSW insert
- FR-021: Parallel build
