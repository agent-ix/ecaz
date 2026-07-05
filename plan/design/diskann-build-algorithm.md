# DiskANN (Vamana) Build Algorithm — Design

This doc describes the Vamana build pipeline for `ecdiskann` (task 17,
ADR-034). It is authoritative for phase-2 build work. It references
pgvectorscale (Timescale) source files as the primary prior art, with
Microsoft's DiskANN and VectorChord cited where their design diverges.

## Goals

- Build a single-layer Vamana graph on top of the existing PqFastScan
  scoring kernel from task 15.
- Produce an index whose scan path (phase 3) can reach
  `Recall@10 ≥ 0.90` at `R = 32`, `L = 100`, `α = 1.2` on the real
  10k fixture.
- Keep the build module entirely inside `src/am/diskann/` so `tqhnsw`
  is untouched.
- Reuse — don't reimplement — grouped PQ codebook training, SRHT
  rotation, and the hot/cold payload page machinery that already
  lives in `src/am/build.rs` for the PqFastScan `tqhnsw` path.

## Non-goals

- OPQ rotation (ADR-036). First build uses SRHT only.
- Additive/residual quantization (ADR-037) and LSQ refinement
  (ADR-038).
- Fresh-Vamana delta-graph merging (FreshDiskANN streaming inserts
  that do not go through ADR-046's full live-insert path).
- Parallel build. First implementation is single-threaded end to end,
  matching pgvectorscale's default posture.

## Pipeline

```
heap scan
   │
   ▼
SRHT rotation  ── (reused from src/quant/fwht.rs / src/quant/prod.rs)
   │
   ▼
grouped PQ4 codebook training  ── (reused from src/quant/grouped_pq.rs)
   │
   ▼
per-vector grouped PQ4 encoding  ── (reused from src/am/build.rs PqFastScan path)
   │
   ▼
binary sidecar encoding (optional)  ── (reused, ADR-031)
   │
   ▼
Vamana graph construction        [new: phase-2 deliverable]
   │
   ▼
medoid approximation             [new]
   │
   ▼
persist pages (hot + cold)       [mostly reused, page layout new]
   │
   ▼
write metadata page              [new page struct]
```

Every stage above the "Vamana graph construction" line consumes
existing code. Any need to change a shared helper is flagged in the
task 17 subtask list — do not in-place-edit shared code from inside
`src/am/diskann/`.

## Vamana graph construction

### Algorithm

Vamana's `BuildIndex` runs two α-pruning passes over a random node
ordering. The canonical statement is in Subramanya et al., NeurIPS
2019, Algorithm 2 and Algorithm 3. pgvectorscale implements it in
`pgvectorscale/src/access_method/build.rs` — specifically the
`build_internal` and `build_graph` functions as of its current
published code — and VectorChord implements it in
`vectorchord/src/indexing/diskann.rs`.

The pseudocode is:

```
Input: vectors X[0..N], degree R, search list size L, relaxation α
Output: graph G with |E(G)| ≤ R * N

G ← empty graph of N nodes
medoid ← approximate medoid of X

for α in [1.0, configured_α]:
    permutation ← random shuffle of [0..N]
    for each i in permutation:
        (V, _) ← GreedySearch(G, medoid, X[i], k=1, L)
        N_i ← RobustPrune(G, i, V, α, R)
        for j in N_i:
            if |N(j) ∪ {i}| ≤ R:
                N(j) ← N(j) ∪ {i}
            else:
                N(j) ← RobustPrune(G, j, N(j) ∪ {i}, α, R)
```

`RobustPrune(G, p, V, α, R)`:

```
V ← V \ {p}
N ← {}
while V ≠ ∅ and |N| < R:
    p* ← argmin_{v in V} d(p, v)
    N ← N ∪ {p*}
    V ← { v in V : α · d(p*, v) > d(p, v) }
return N
```

`GreedySearch(G, start, q, k, L)`:

```
visited ← {}
frontier ← {start}
while frontier \ visited ≠ ∅:
    p* ← argmin_{p in frontier \ visited} d(p, q)
    visited ← visited ∪ {p*}
    frontier ← frontier ∪ N(p*)
    if |frontier| > L:
        frontier ← top-L nodes of frontier by d(·, q)
return (frontier, visited)
```

### What differs between prior-art implementations

| Aspect | Microsoft DiskANN (C++) | pgvectorscale (Rust) | VectorChord (Rust) | This design |
|---|---|---|---|---|
| Distance metric during build | L2 or IP | L2 on rotated+quantized vectors | L2 (w/ IP-aware ingest) | IP (negative) on PqFastScan-scored codes |
| Permutation | Single random | Single random | Single random | Single random, `seed` from reloption |
| Passes | 2 (α=1, α=1.2) | 2 | 2 | 2 |
| Medoid | Exact O(N²) approx | Random-sample approx | Random-sample approx | Random-sample approx |
| Storage during build | In-RAM adjacency | In-RAM adjacency, page-backed | In-RAM adjacency | In-RAM adjacency, page-backed |

Follow pgvectorscale's memory-shape because it lives in the same
Postgres/pgrx environment and uses the same page-layout discipline
we do. Diverge on distance function (IP) and scoring input
(`score_ip_encoded` against PqFastScan codes, not L2 on quantized
doubles).

### Reference source files

- **pgvectorscale**:
  - `pgvectorscale/src/access_method/build.rs` — top-level build
    driver. Consult `build_internal` for the two-pass structure.
  - `pgvectorscale/src/access_method/graph.rs` — in-RAM graph
    representation plus `insert_into_graph` entry for live-insert
    vs build.
  - `pgvectorscale/src/access_method/pruner.rs` — `RobustPrune`
    implementation.
  - `pgvectorscale/src/access_method/debugging.rs` — graph
    validity checks worth mirroring in tests.
- **VectorChord**:
  - `vectorchord/src/indexing/diskann.rs` — Vamana build driver.
  - `vectorchord/src/indexing/rabitq.rs` — their binary quantizer,
    for contrast; we use our existing ADR-031 sidecar.
- **Microsoft DiskANN**:
  - `DiskANN/src/index.cpp` — canonical `BuildIndex`. Heavier than
    we need because it also handles disk-resident adjacency; study
    `generate_frozen_point`, `prune_neighbors`, and `build` for the
    algorithm; skip the SSD resident-index parts.

### Distance function

Vamana's pseudocode uses an arbitrary distance. DiskANN papers use
L2. pgvectorscale supports both L2 and negative inner product;
VectorChord uses L2 after rotation.

`ecdiskann` uses **negative inner product** (the `<#>` operator) on
PqFastScan scores as its distance function, because:

- tqvector's scoring kernel is IP-first. Task 15 already validates
  PqFastScan IP scoring on the `<#>` surface.
- `RobustPrune`'s α-dominance rule is defined in terms of a
  distance `d` that must be nonnegative. We use
  `d(x, y) = max(0, -ip(x, y) + C)` with `C = 1` when vectors are
  unit-normalized, which preserves the ordering induced by `<#>`
  without breaking the α inequality.
- The same choice survives scan-time: greedy search and
  `RobustPrune` use the identical scoring wrapper, and the stored
  graph edges are consistent with the scan's scoring semantics.

See `spec/adr/ADR-007-query-scoring-and-payload.md` for the
existing wrapper contract. The build path reuses
`score_ip_codes_lite` for candidate-vs-candidate scoring during
`RobustPrune`, and the FastScan grouped scorer during the greedy
search's batch expand step.

### Medoid approximation

A true medoid requires `O(N²)` distance evaluations. For N in the
millions this is intractable. Use a random-sample approximation
matching pgvectorscale:

1. Draw `S = min(MEDOID_SAMPLE_CAP, N)` sample indices uniformly
   without replacement, where `MEDOID_SAMPLE_CAP = 1000`
   (Phase 5C-2 frozen value; mirrors pgvectorscale).
2. For each sampled index `s`, compute the sum of distances to all
   other samples in `S`.
3. Take `argmin_s (sum of distances)` as the medoid.

Cost: `O(S²)` distance evaluations — ~1M at the cap, bounded and
runs in well under a second with FastScan scoring.

Record the medoid TID in the Vamana metadata page. Both scan and
insert use it as the graph entry point (ADR-046 step 1).

### Page layout persistence

During build, the graph lives in heap-allocated adjacency lists.
Once both α passes complete, persist in one sweep:

- **Hot page chain** — node tuples containing:
  - `element_header` (element TID, gamma, payload-flags)
  - optional `binary_sidecar` (ADR-031, 1 bit per dimension,
    packed)
  - `grouped_search_code` (PqFastScan 4-bit grouped code)
  - `neighbor_list` (R × ItemPointer, INVALID when slot empty)

  One node occupies one tuple. Page-fit math mirrors the
  PqFastScan-on-tqhnsw layout but without the per-layer
  segmentation that HNSW requires.

- **Cold page chain — DEFERRED in V0.** ADR-045 reserves
  `rerank_tid` and ADR-044 keeps raw-f32 rerank on the heap
  `ecvector` row. V0 `ec_diskann` therefore persists only the hot
  chain; `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD` stays clear on V0 builds
  (ADR-046 frozen rule 1, ADR-047 frozen rule 4). A future ADR-044
  C1 reopen is the only path that re-introduces an index-side cold
  chain; that reopen ships its own ADR and flips the flag as part of
  the format extension.

- **Metadata page** — see `plan/tasks/17-diskann-access-method.md`
  phase-1 subtask for the struct definition.

Persistence order (V0):

1. Allocate hot pages and write node tuples in visit order from
   the medoid outward (roughly distance-ordered). This improves
   page-cache locality for scan's greedy walk from the medoid.
2. Write the metadata page last, with `entry_point_tid` =
   medoid, `graph_degree_R`, `build_list_size_L`,
   `alpha` (stored as `f32`, pgvectorscale-compatible),
   `format_version = INDEX_FORMAT_V4_DISKANN`, and
   `payload_flags` with `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD` **clear**.

The visit-order-from-medoid rule matters more for Vamana than for
HNSW. The scan always starts at the medoid and walks outward;
grouping nearby-in-graph nodes onto nearby pages means early-
search I/O is sequential.

## Validation plan

### Unit tests (pure, no pg runtime)

- `RobustPrune` property test: output size `≤ R`, output is a
  subset of the input candidate set minus the pivot, α relaxation
  obeyed (pgvectorscale has a direct analogue worth mirroring).
- `GreedySearch` property test: returns at most `L` candidates,
  visited set bounded, monotonic distance improvement invariant on
  the frontier head.
- Medoid approximation sanity: on uniform random 1000 vectors,
  approximated medoid's average distance is within 10% of exact
  medoid's.

### Integration tests (pgrx / pg-tap)

- `CREATE INDEX` on the 1000-row synthetic fixture produces a
  decodable metadata page and entry-point TID.
- Graph validity: every live node has between 1 and `R`
  neighbors; every neighbor TID points at a live node (modulo
  `INVALID` slots).
- Connectivity: BFS from the medoid reaches ≥ 95% of live nodes
  (full connectivity not guaranteed, but isolated islands should
  be rare at `R ≥ 32` on dense data).

### Recall smoke

- 10k real fixture, build with `R = 32`, `L = 100`, `α = 1.2`.
- Measure `Recall@10` at `ef_search ∈ {64, 128, 200}` against
  brute-force fp32 truth.
- Gate: `Recall@10 ≥ 0.90` at `ef_search = 128` is the hard
  baseline; `~0.95` is the preferred landing target. Below 0.90
  is a build-quality regression; do not ship.

## Open questions for reviewer

1. **α-pruning distance wrapper.** The `d = max(0, -ip + C)`
   wrapper above is the minimum-disruption way to use IP with
   α-dominance. Do reviewers prefer this, or switching to cosine
   (which is IP after unit-normalization and makes the α inequality
   trivially correct without a `C` offset)?
2. **Medoid-sample size.** pgvectorscale caps at 10 000 samples.
   For our 1536-dim/4-bit corpus with cheap FastScan scoring, is a
   higher cap worth the extra build time? Proposed baseline:
   10 000, tunable via a non-documented reloption for benchmarking.
3. **Persistence order.** Visit-order-from-medoid is a scan-I/O
   optimization that costs build time (full BFS before writes).
   Reviewers should confirm this is worth it at 1B-scale — the
   alternative is heap-order persistence, which matches
   pgvectorscale's default but loses cache locality on scan.
4. **Distance wrapper determinism.** `score_ip_codes_lite` returns
   f32. `RobustPrune`'s α test is defined in reals. Confirm that
   the f32 rounding does not cause nondeterministic graph builds
   across runs with the same `seed` reloption.

## References

- Subramanya et al., *DiskANN: Fast Accurate Billion-Point Nearest
  Neighbor Search on a Single Node*, NeurIPS 2019. Canonical Vamana
  definition.
- Singh et al., *FreshDiskANN*, 2021. Streaming-insert variant.
- pgvectorscale (Timescale), public Rust+pgrx implementation.
  Closest prior art. See source files cited above.
- VectorChord, public Rust implementation. Alternative trade-offs.
  See source files cited above.
- Microsoft DiskANN, canonical C++ implementation.
- ADR-030: FastScan Grouped Subvector Scoring.
- ADR-031: RaBitQ Binary Pre-Filter.
- ADR-034: DiskANN as Second Access Method.
- ADR-046: Vamana Live Insert Lock Ordering.
- ADR-047: Vamana Vacuum Graph Repair Lock Ordering.
- Task 15: PqFastScan First-Class (kernel consumed as-is).
- Task 17: DiskANN Access Method (this doc's execution vehicle).
