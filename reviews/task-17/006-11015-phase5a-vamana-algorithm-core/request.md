# Review Request: Phase 5A — In-Memory Vamana Algorithm Core

Branch: `adr034-diskann-access-method`
Author: coder-2
Companion to: 11001 (task 17 plan), 11004 (build-algorithm design),
11014 (ADR-045)

## What this slice is

First sub-slice of task 17 **Phase 5** (build pipeline): the pure-Rust
algorithmic core for Vamana graph construction. No pgrx dependencies,
no page layout, no quantizer wiring — just the algorithm with abstract
distance closures. Phase 5B (slim tuple rewrite, packet 11016) and
Phase 5C (build → persist plumbing, packet 11009) consume this module
and ship behind it.

## Scope

- `src/am/diskann/vamana.rs` (new file, ~564 lines including tests)
- `src/am/diskann/mod.rs` — `pub mod vamana;` declaration

Implements the algorithm from
`plan/design/diskann-build-algorithm.md` §"Vamana graph
construction": canonical Subramanya et al. NeurIPS 2019 Algorithms 2
and 3, two-pass α=1.0 then α=configured, with the pgvectorscale-shape
random-sample medoid approximation.

## What changed

### `src/am/diskann/vamana.rs`

Five public items:

```rust
pub struct VamanaGraph {
    pub neighbors: Vec<Vec<u32>>,
    pub max_degree: usize,
}

pub struct Candidate { pub node: u32, pub distance: f32 }

pub fn greedy_search<D: Fn(u32) -> f32>(
    graph: &VamanaGraph, start: u32, list_size: usize, query_dist: D
) -> GreedySearchResult;

pub fn robust_prune<D: Fn(u32, u32) -> f32>(
    pivot: u32, candidates: Vec<Candidate>, alpha: f32,
    max_degree: usize, dist: D
) -> Vec<u32>;

pub fn approximate_medoid<D: Fn(u32, u32) -> f32>(
    node_count: usize, sample_cap: usize, seed: u64, dist: D
) -> u32;

pub fn build_vamana_graph<D: Fn(u32, u32) -> f32 + Copy>(
    node_count: usize, medoid: u32, max_degree: usize,
    list_size: usize, alpha_final: f32, seed: u64, dist: D
) -> VamanaGraph;

pub fn bfs_reachable(graph: &VamanaGraph, start: u32) -> Vec<u32>;
```

**Distance is abstract.** Callers pass `Fn(u32) -> f32` (query-time) or
`Fn(u32, u32) -> f32` (build-time) closures. Phase 5C will bind these
to `score_ip_codes_lite` (candidate-vs-candidate) and the FastScan
grouped scorer (query-vs-candidate). The algorithm core does not
depend on the quantizer family.

**Determinism.** Both shuffling and reservoir sampling use
`ChaCha8Rng::seed_from_u64(seed)`. Same `seed` reloption ⇒ bit-exact
graph across runs.

### Tests (six unit tests, all green)

- `robust_prune_respects_max_degree` — prune output bounded by R, pivot
  excluded.
- `robust_prune_excludes_alpha_dominated` — α-dominance rule fires
  correctly on a hand-built case.
- `greedy_search_finds_nearest` — linear-chain graph, search converges
  on the target.
- `build_small_graph_is_connected` — 100-point synthetic build, every
  node has between 1 and R neighbors, BFS from medoid reaches every
  node.
- `approximate_medoid_within_10pct_of_exact` — full-population
  reservoir sample equals the exact O(N²) medoid; sub-sampled medoid
  total-distance within 10% of exact.
- `build_recall_at_10_meets_baseline` — 500-point synthetic, build at
  R=16 / L=64 / α=1.2, query 50 random points, Recall@10 ≥ 0.80
  sanity floor. **Not** the production target — the production target
  (Recall@10 ≥ 0.90 at 1536d on real PqFastScan codes) lands at
  Phase 6 once scan is wired.

## Review focus

1. **Frontier truncation cost.** `greedy_search` truncates the
   frontier to `L` after each expansion via `sort + truncate`
   (O(F log F)) rather than a max-heap-based eviction
   (O(F log L)). For L ≤ 200 this is fine; profiling at Phase 6
   may flip it. The ADR-041 stage 0 test gate is recall-bit-exact,
   not perf, so this is intentionally deferred. Reviewer call:
   accept the simpler shape now, or pre-emptively switch to the
   heap variant.
2. **`BinaryHeap` import retained but unused.** A `#[allow(dead_code)]`
   const at line 354 keeps the import for the heap-variant pivot
   above. Reviewer call: drop it now (one-line follow-up when
   needed) or keep as an in-place breadcrumb.
3. **Shuffle inside the build loop.** `build_vamana_graph` re-shuffles
   the permutation between α passes. pgvectorscale uses a single
   shuffle for both passes. Differences in the two approaches are
   below the seed-reproducibility threshold, but this is a place
   where we could match prior art exactly. Reviewer preference?
4. **`robust_prune` retain comparison uses strict `>`.** Per
   pgvectorscale, exact ties prune (i.e. `<=` on the drop side).
   The implementation reads `alpha * dist > v.distance`, which is
   the same semantics (keep when *not* strictly dominated). Confirm
   the alignment is what reviewer expects.
5. **Synthetic-recall floor at 0.80, not 0.90.** The 0.90 production
   target is on real 1536d data with PqFastScan codes — Recall@10
   on flat 2D L2 is harder because the dataset is dense and the
   graph is sparse relative to embedding dims. 0.80 is the sanity
   floor that catches catastrophic algorithm bugs without becoming
   flaky. Phase 6's PgTAP test owns the real 0.90 gate.

## Questions to answer

- **Distance-closure design vs. trait.** The algorithm core takes
  closures rather than a `trait Distance`. Argument for: testable
  with synthetic L2 in isolation, no plumbing through `Quantizer`
  during phase 5A. Argument against: at integration time (5C) the
  closures will capture `&PqFastScanQuantizer` and call
  `score_ip_codes_lite` — a trait would express that dependency
  more honestly. I went with closures because the algorithmic
  contract is just "(node, node) → nonnegative f32" and a trait
  would over-specify. Reviewer confirm.
- **Graph in `Vec<Vec<u32>>` vs. flat `Vec<u32>` with offsets.**
  Adjacency-list shape is convenient for build (variable-length
  during construction, settled to ≤R after final prune). For very
  large graphs the flat shape saves a `Vec` header per node (~24B
  × N). At task-17 scale (10k–10M) this is negligible; for SPANN /
  ADR-035 it would matter. Out of scope here.

## Not doing in this packet

- **Page layout and persistence.** Phase 5B (slim tuple, packet
  11016) and Phase 5C (build → persist, packet 11009).
- **PqFastScan integration.** The algorithm core uses abstract
  closures; the closure binding to `Quantizer::prepare_scorer` /
  `score_ip_codes_lite` happens in Phase 5C.
- **Live insert.** Phase 7. The build path is snapshot-only.
- **Optimized greedy frontier (BinaryHeap).** Reserved for
  profile-driven follow-up at Phase 6.

## Dependencies

- **`rand = "0.8"`, `rand_chacha = "0.3"`** — already in
  `Cargo.toml`. No new deps.
- **ADR-034** — task 17 ADR.
- **`plan/design/diskann-build-algorithm.md`** — algorithmic spec.
- **ADR-045** — gate for Phase 5B/5C, not strictly for 5A (5A is
  layout-agnostic). Land 5A independently or together with the 5B
  rewrite per reviewer preference.

## Companion packets

- 11014 — ADR-045 page-layout discipline (companion design).
- 11016 — Phase 5B slim tuple rewrite (filed alongside this packet).
- 11009 — Phase 5C build → persist plumbing (future).

## Definition of ready

- Reviewer accepts algorithm shape and abstraction boundaries.
- Six unit tests green (verified locally).
- Cargo lib check clean (5 pre-existing dead-code warnings only).
- Phase 5C does not start before this lands.

## Handoff notes

The algorithm-core is the pure piece of the Phase-5 work — every
later sub-slice consumes it. If reviewer pushes back on the
closure-vs-trait shape (review focus #1 in §Questions to answer),
the 5C wiring is the only consumer that has to change; the unit
tests stay valid.

The 0.80 Recall@10 sanity floor is intentionally loose. If this test
goes flaky (rare on `seed = 17` + ChaCha8), the right move is to
bump the seed or the dataset size, not to lower the floor — a real
recall regression in the algorithm should be caught at Phase 6
against the production fixture, not papered over here.
