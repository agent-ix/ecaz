# Review Request: Phase 6A — Scan Algorithm Shell

Branch: `adr034-diskann-access-method`
Author: coder-2
Companion to: 11014 (ADR-045), 11017 (Phase 5C-1), 11018 (Phase
5C-2), 11021 (Phase 8A), 11022 (Phase 5D)

## What this slice is

Pure-Rust composition of Phase 5D's `PersistedGraphReader` into the
two-stage scan flow `amgettuple` will call:

1. **Greedy descent with a cheap prefilter.** Walk the persisted
   graph under an `L_search`-bounded frontier; score each visited
   node via a caller-supplied `prefilter: Fn(&VamanaNodeTuple) -> f32`
   closure (typically binary-Hamming or grouped-PQ4).
2. **Exact rerank on the top candidates.** Take the top
   `rerank_budget` from the greedy frontier, call a caller-supplied
   `rerank: Fn(ItemPointer) -> f32` (typically an ecvector cold
   path / heap fetch), re-sort by exact distance, truncate to
   `top_k`.

No pgrx, no quantizer, no heap access — those are all injected via
closures. This is the layer Phase 6B's pgrx `amgettuple` will drive.

## Scope

- `src/am/diskann/scan.rs` — new file, 622 lines incl. 10 tests.
- `src/am/diskann/mod.rs` — `pub mod scan;` declaration.

No other source files touched.

## What changed

### Public API

```rust
pub struct ScanParams {
    pub entry_point: ItemPointer,
    pub list_size: usize,       // L_search
    pub rerank_budget: usize,   // K_rerank  ≤ L_search
    pub top_k: usize,           // K          ≤ K_rerank
}

pub struct ScanResult {
    pub tid: ItemPointer,
    pub primary_heaptid: ItemPointer,
    pub distance: f32,
}

pub struct ScanCandidate {
    pub tid: ItemPointer,
    pub primary_heaptid: ItemPointer,
    pub score: f32,
}

pub fn vamana_scan<Pre, Re>(
    reader: &PersistedGraphReader<'_>,
    params: ScanParams,
    prefilter: Pre,
    rerank: Re,
) -> Result<Vec<ScanResult>, String>
where
    Pre: Fn(&VamanaNodeTuple) -> f32,
    Re: Fn(ItemPointer) -> f32;

pub fn greedy_descent<Pre>(
    reader: &PersistedGraphReader<'_>,
    entry_point: ItemPointer,
    list_size: usize,
    prefilter: &Pre,
) -> Result<Vec<ScanCandidate>, String>
where
    Pre: Fn(&VamanaNodeTuple) -> f32;
```

### What it does

- `vamana_scan` validates parameters (`list_size > 0`, `rerank_budget
  ≤ list_size`, `top_k ≤ rerank_budget`), runs `greedy_descent`,
  applies rerank to the top `rerank_budget` candidates, sorts
  ascending by exact distance, truncates to `top_k`.
- `greedy_descent` mirrors `reader::greedy_search_persisted` but
  scores nodes via the prefilter closure (not a raw `Fn(ItemPointer)
  -> f32`). It caches `primary_heaptid` on each `ScanCandidate` so
  the rerank stage does not re-decode. `ScanCandidate` is public so
  Phase 6B can drive batched rerank across `amgettuple` calls.

### Tests (10, all green)

- **SC-001** end-to-end top-1 on a distance-gradient chain graph.
- **SC-002** **rerank can reorder prefilter** — prefilter ranks
  node 2 first, rerank ranks node 5 first; final result is node 5.
  Locks the two-stage contract.
- **SC-003** `rerank_budget` caps rerank-closure call count
  exactly. Uses `Cell<usize>` to observe call count.
- **SC-004** `top_k` truncates the reranked output.
- **SC-005** `INVALID` entry rejected with a clear error.
- **SC-006** parameter validation — zero sizes and ordering
  violations (`rerank_budget > list_size`, `top_k > rerank_budget`)
  all error.
- **SC-007** **end-to-end matches brute force** — 64 synthetic 2D
  L2 points, real Vamana build, `list_size=20`, `rerank_budget=10`,
  `top_k=5`. Top-1 exact match vs brute-force nearest neighbor;
  top-5 recall overlap ≥ 4/5.
- **SC-008** `ScanResult.primary_heaptid` comes from the decoded
  tuple (what `amgettuple` returns to Postgres).
- **SC-009** results sorted ascending by rerank distance.
- **SC-010** `greedy_descent` exposed standalone, drives a real
  build via `build_and_persist_vamana`, frontier is sorted and
  length `min(list_size, N)`.

```
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured;
             583 filtered out; finished in 0.01s
```

Full diskann module: 74 tests pass (10 scan + 10 reader + 10 vacuum
+ 6 page + 13 tuple + 6 vamana + 11 persist + 8 build).
`cargo check --lib` clean (5 pre-existing dead-code warnings).

## Review focus

1. **Two-stage flow with two injected closures.** `prefilter`
   operates on the decoded tuple (has the binary sidecar + search
   code bytes available), `rerank` operates on the heap TID (cold
   path). Reviewer confirm this is the right split — the
   alternative is a single `Fn(&VamanaNodeTuple) -> f32` for both
   stages with the caller responsible for heap prefetch inside.
   The two-closure split makes rerank cost observable (SC-003) and
   gives Phase 6B latitude to batch heap fetches.
2. **`ScanParams` validates the ordering `top_k ≤ rerank_budget ≤
   list_size`.** The three-knob API mirrors the DiskANN paper
   nomenclature. Reviewer confirm this is what pgrx reloptions
   will expose, or flag if one of the three should default and the
   API should shrink.
3. **`greedy_descent` is public alongside `vamana_scan`.** Reason:
   Phase 6B may want to run descent once per cursor and stream
   reranked results across `amgettuple` calls (rather than collect
   all top-K up front). Reviewer confirm exposing both; the
   alternative is keeping `greedy_descent` private until a caller
   needs it.
4. **`ScanCandidate` is a named type, not a tuple.** Carries
   `(tid, primary_heaptid, score)`. Reviewer flag if you'd rather
   see it inline as a (ItemPointer, ItemPointer, f32) tuple.
5. **Rerank happens in-process, not pushed to the pgrx caller.**
   The shell owns the re-sort + truncate so both tests and the
   pgrx caller see the same top-K shape. Alternative: return the
   pre-rerank candidates + let the caller rerank. Held: in-process
   so the shell is the canonical top-K producer.
6. **No tombstone filtering.** Consistent with Phase 5D's
   decision: scan does not skip `deleted = true` nodes because
   MVCC visibility is the pgrx layer's job. A prefilter closure
   *could* return `+∞` for tombstones, but the shell doesn't force
   that.

## Questions to answer

- **Should the shell expose a builder that binds a
  `Quantizer`-derived prefilter + an ecvector-derived rerank?**
  Held: no — the shell is quantizer-agnostic, and the Phase 6B
  pgrx layer is the single site that knows which quantizer / heap
  path is in play.
- **Should `vamana_scan` return an iterator/stream for lazy top-K
  consumption?** Held: Vec of top-K fits the pgrx
  `amgettuple` pattern (it stores the result cursor and pulls one
  at a time). A future streaming variant is a thin change.
- **Should `list_size` saturate to `N` (the graph's node count)
  silently, or error?** Currently neither — greedy terminates
  naturally when the frontier is exhausted (this is what SC-010
  asserts for `N < list_size`). Reviewer confirm this is the
  right behavior.

## Not doing in this packet

- **pgrx `amgettuple` callback.** Phase 6B — deferred with the
  native-build lane.
- **Quantizer-backed prefilter.** The SRHT + grouped-PQ4 scoring
  lives in `src/quant/*`; the prefilter closure will be bound in
  Phase 6B (`Quantizer::prepare_scorer(...)`). This shell is
  distance-agnostic.
- **Heap rerank.** Phase 6B binds the rerank closure to the
  ecvector cold path.
- **Binary-prefilter microbenchmarks.** The shell doesn't assume a
  specific prefilter cost; Phase 6B sets the reloption defaults
  for `list_size` / `rerank_budget` and benchmarks end-to-end.
- **Iterator / streaming variant.** Defer until a caller needs it.

## Dependencies

- **ADR-045 ACCEPTED** — decode relies on fixed-length tuples.
- **Phase 5B (11016)** — uses `VamanaNodeTuple`.
- **Phase 5D (11022)** — consumes `PersistedGraphReader`. This
  packet is the first external caller of Phase 5D beyond its own
  tests.
- **Phase 5C-2 (11018)** — SC-007 and SC-010 drive
  `build_and_persist_vamana` to produce the test fixture.

## Companion packets

- **11022** — Phase 5D persisted-graph reader (direct dependency).
- **11021** — Phase 8A vacuum primitives (sibling pure-Rust slice).
- **Future** — Phase 6B pgrx `amgettuple` wiring (deferred with
  native-build lane merge).

## Definition of ready

- ADR-045 ACCEPTED.
- 10 SC tests green (verified locally).
- Reviewer confirms the two-closure shape and the `ScanParams`
  ordering invariants.
- Phase 6B does not start before this lands.

## Handoff notes

Once the native-build lane merges, Phase 6B collapses to:

1. In `amgettuple`, read the metadata page, open the
   `DataPageChain`, construct a `PersistedGraphReader`.
2. Bind `prefilter = quantizer.prepare_scorer(query)` — a closure
   that reads `tuple.binary_words` and `tuple.search_code`.
3. Bind `rerank = |hip| ecvector::exact_distance(relation, hip,
   query)`.
4. Call `vamana_scan`, return the `ScanResult`s as TIDs via the
   index cursor.

The shell is deliberately small (~180 lines non-test) so the pgrx
glue is thin. If reviewer pushes back on the two-closure shape,
the surface is easy to redraw — but SC-002 and SC-003 are the
tests that anchor the current split.
