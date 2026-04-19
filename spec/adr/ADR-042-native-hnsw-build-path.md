---
id: ADR-042
title: "Native HNSW build path — retire hnsw_rs from production BUILD"
status: PROPOSED
impact: Affects FR-008 (HNSW build), FR-016 (HNSW insert), FR-021 (parallel build), ADR-002, ADR-003, ADR-030
date: 2026-04-18
---
# ADR-042: Native HNSW build path — retire `hnsw_rs` from production BUILD

## Context

`hnsw_rs` was adopted early (ADR-002, ADR-003) to bootstrap the graph before
tqvector had its own insert/vacuum implementation. That job is now done at one
end of the lifecycle but not the other:

- **INSERT** (`src/am/insert.rs`) is fully native. It samples levels, discovers
  forward neighbors, mutates backlinks under the ADR-026 lock order, and scores
  candidates using the same quantized kernels used by scan.
- **BUILD** (`src/am/build.rs`) still delegates graph construction to `hnsw_rs`.
  `build_hnsw_graph` and `build_hnsw_graph_from_source` construct an in-memory
  `Hnsw` via `Hnsw::new_with_seed`, call `hnsw.insert(...)` per heap tuple, and
  then walk `get_point_indexation()` / `hnsw_rs::hnsw::Neighbour` to repack
  neighbors into the tqvector slot layout.

Two concerns have accumulated against keeping `hnsw_rs` on the production BUILD
path:

### Semantic drift between BUILD and INSERT

BUILD and incremental INSERT of the same rows should produce graphs with the
same recall characteristics. Today the two paths differ in:

- RNG / layer-assignment source (`hnsw_rs`'s internal generator vs. our seeded
  level sampler)
- neighbor-selection heuristic (`hnsw_rs`'s heuristic vs. the tqvector
  backlink-pruning rules used on INSERT)
- distance dispatch shape (`BuildCodeDistance` trait wrapper vs. direct calls
  into the quantized scoring kernels)

This drift is already visible in review history (`22-pin-hnsw-rs-version`,
`212-a4-build-hierarchy-collapse-audit`, `211-a4-upper-hierarchy-oracle-k`) and
complicates reasoning about why a graph from BUILD behaves differently from the
same corpus streamed through INSERT.

### Coupling to third-party types

`hnsw_rs::hnsw::Neighbour` and `PointId` leak into `src/am/build.rs` function
signatures (`flatten_point_neighbors`, `pack_point_neighbor_slots`,
`fill_point_neighbor_layer_slots`). Internal code paths are shaped around a
type we do not own, and deterministic BUILD depends on upstream not changing
traversal or RNG order across versions. The crate is currently vendored under
`vendor/hnsw_rs/` to pin that behavior.

### Performance

BUILD pays avoidable overhead:

- codes are re-decoded through the `Distance` trait wrapper instead of scored
  by the SIMD PQ/fastscan kernels INSERT already uses
- neighbor lists are allocated as `Vec<Vec<Neighbour>>` per point and then
  repacked into the final slot layout, instead of written directly
- insertion is serial (`for tuple in heap_tuples { hnsw.insert(...) }`) and
  does not compose with the heap-scan parallel worker model targeted by FR-021

A native builder is not expected to deliver an order-of-magnitude speedup on
the inner search loop, but it is expected to recover single-digit to roughly 2×
on representative workloads from (a) shared quantized kernels and (b)
parallelism aligned with the heap-scan model. The stronger motivation is
*correctness parity with INSERT*, not raw speed.

## Decision

Implement a **native HNSW build path** that replaces `hnsw_rs` for production
index construction, and demote `hnsw_rs` to a **test-only recall oracle**.

### Native builder responsibilities

The native builder will:

- accept the existing `BuildState` and produce the same `Vec<HnswBuildNode>`
  shape already consumed by the flush/serialization path
- perform level assignment using tqvector's seeded RNG (same source INSERT
  uses), so the same seed produces comparable topology in BUILD and INSERT
- perform forward-neighbor discovery and backlink pruning using the same rules
  used by `src/am/insert.rs`
- call quantized scoring kernels directly — the same kernels used by scan and
  INSERT — without a `Distance` trait wrapper
- write neighbor slots directly into the final slot layout, without an
  intermediate `Vec<Vec<Neighbour>>` per point
- compose with the heap-scan parallel worker model for FR-021

Both BUILD variants currently backed by `hnsw_rs` are in scope:

- code-graph BUILD (`build_hnsw_graph`)
- source-graph BUILD (`build_hnsw_graph_from_source`)

### `hnsw_rs` retained only as a test oracle

`hnsw_rs` remains a dependency, but only for:

- recall baselines / probes in `src/lib.rs`
  (`probe_hnsw_rs_code_graph_recall`, `probe_hnsw_rs_source_graph_recall`, and
  the `test_hnsw_rs_*_10k` tests that use them)
- any future reference-graph comparisons that want an independent implementation
  to cross-check recall characteristics

No production code path may depend on `hnsw_rs`. Specifically, `src/am/*` must
not reference `hnsw_rs`.

### Determinism

The native builder must be deterministic given a fixed `(seed, dimensions,
bits, options)` tuple. Determinism parity with INSERT (same seed + same row
stream → equivalent graph, up to the documented tolerance of the backlink
pruning heuristic) is a first-class acceptance criterion, not a follow-up.

### What this ADR rejects

#### Forking a second `hnsw_rs`-shaped builder inside `src/am/`

Rejected because the goal is to collapse two HNSW implementations into one
(native), not produce a third.

#### Swapping `hnsw_rs` for another third-party HNSW crate

Rejected for the same drift and coupling reasons that motivate removing
`hnsw_rs` from the production path. tqvector already owns an HNSW
implementation (INSERT); BUILD should reuse it, not import a new external one.

#### Removing `hnsw_rs` entirely

Rejected (for now). The recall oracle role is genuinely useful: an independent
implementation makes it possible to distinguish "tqvector graph is degraded"
from "HNSW on this corpus just behaves this way." Retiring it from production
does not require removing it from tests.

## Consequences

### Positive

- single authoritative HNSW implementation across BUILD and INSERT
- BUILD can use the same SIMD PQ/fastscan kernels scan and INSERT use
- determinism no longer depends on an upstream crate's RNG or traversal order
- unblocks FR-021 parallel BUILD aligned with the heap-scan worker model
- removes `hnsw_rs::Neighbour` / `PointId` from `src/am/build.rs` signatures

### Negative

- one-time engineering cost to port level assignment, neighbor discovery,
  pruning, and packing into a builder-shaped entry point
- recall must be re-baselined against the retained `hnsw_rs` oracle to show
  the native builder does not regress against the prior BUILD path
- grouped-graph work (ADR-030) has to decide whether to land against the native
  builder directly or against `hnsw_rs` first; this ADR argues for the former

### Neutral

- `hnsw_rs` remains in `Cargo.toml` and `vendor/` as a `[dev-dependencies]` /
  test-only crate; no user-visible API change
- ADR-002 and ADR-003 remain historically accurate but are superseded for the
  production BUILD path by this ADR

## References

- ADR-002: hnsw_rs has no delete — own the graph in Postgres pages
- ADR-003: Walk hnsw_rs graph and write to Postgres pages
- ADR-026: Live Insert Backlink Lock Ordering
- ADR-030: FastScan Grouped Subvector Scoring
- FR-008: HNSW build
- FR-016: HNSW insert
- FR-021: Parallel build
- Task 10065: Native HNSW build path (`plan/tasks/coder2/10065-native-hnsw-build-path.md`)
