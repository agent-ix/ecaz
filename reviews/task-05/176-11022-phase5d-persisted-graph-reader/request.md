# Review Request: Phase 5D — Persisted-Graph Reader

Branch: `adr034-diskann-access-method`
Author: coder-2
Companion to: 11014 (ADR-045), 11015 (Phase 5A), 11016 (Phase 5B),
11017 (Phase 5C-1), 11018 (Phase 5C-2), 11021 (Phase 8A)

## What this slice is

Pure-Rust bridge from Phase 5C-1's persist output (a
`DataPageChain` of encoded `VamanaNodeTuple`s) to the eventual
Phase 6A scan algorithm. Exposes:

- `PersistedGraphReader<'a>` — a lightweight borrow over a
  `&DataPageChain` + the metadata `(R, W, C)` triple.
- `read_node` / `neighbors` — decode one tuple / its filled-prefix
  neighbor TIDs.
- `greedy_search_persisted` — same greedy best-first loop as Phase
  5A's `greedy_search`, but keyed on `ItemPointer` instead of dense
  `u32` node ids.

Once the native-build lane merges and Phase 5C-3 wires up pgrx,
`amgettuple` collapses to: open the relation, build a
`PersistedGraphReader` over its `DataPageChain`, call
`greedy_search_persisted` with a `Quantizer::prepare_scorer`-derived
closure, return TIDs.

## Scope

- `src/am/diskann/reader.rs` — new file, 551 lines incl. 10 tests.
- `src/am/diskann/mod.rs` — `pub mod reader;` declaration.

No other source files touched.

## What changed

### Public API

```rust
pub struct PersistedGraphReader<'a> {
    pub chain: &'a DataPageChain,
    pub graph_degree_r: u16,
    pub binary_word_count: usize,
    pub search_code_len: usize,
}

impl<'a> PersistedGraphReader<'a> {
    pub fn new(chain: &'a DataPageChain, r: u16, w: usize, c: usize) -> Self;
    pub fn read_node(&self, tid: ItemPointer) -> Result<VamanaNodeTuple, String>;
    pub fn neighbors(&self, tid: ItemPointer) -> Result<Vec<ItemPointer>, String>;
}

pub struct TidCandidate { pub tid: ItemPointer, pub distance: f32 }
pub struct PersistedGreedyResult {
    pub frontier: Vec<TidCandidate>,
    pub visited: Vec<ItemPointer>,
}

pub fn greedy_search_persisted<D: Fn(ItemPointer) -> f32>(
    reader: &PersistedGraphReader<'_>,
    entry_point: ItemPointer,
    list_size: usize,
    query_dist: D,
) -> Result<PersistedGreedyResult, String>;
```

### What it does

- `read_node(tid)` → page lookup, raw-tuple lookup, `decode`. Errors
  surface from each layer with the block number for context.
- `neighbors(tid)` → decoded tuple's `neighbors[..neighbor_count]`,
  tail `INVALID` slots dropped. Stable order (matches ADR-047 fill
  order).
- `greedy_search_persisted` — mirrors `vamana::greedy_search` line
  for line, except:
  - Frontier / visited tracked in `HashSet<ItemPointer>` because
    TIDs are sparse (Vec<bool> keyed on node id doesn't apply).
  - Entry point and `list_size` validated up front.
  - Neighbors fetched per expansion via `reader.neighbors(tid)?`.
  - Tie-break order on `TidCandidate` falls through to
    `(block_number, offset_number)` so the result is deterministic.

### Tests (10, all green)

- **RD-001** single-node persist + round-trip via `read_node`:
  `primary_heaptid`, `binary_words`, `search_code`, `neighbor_count`
  all match the input payload.
- **RD-002** `read_node` on an unknown block number surfaces
  `"page N not found in chain"`.
- **RD-003** `neighbors()` drops trailing `INVALID` slots — a
  chain-graph node with 1 neighbor returns a 1-element Vec even
  though the tuple has `R=8` slots.
- **RD-004** BFS via the reader reaches every node in a 12-node
  connected chain.
- **RD-005** adjacency via `reader.neighbors(tid)` matches the
  in-memory `VamanaGraph` for every node after persist (oracle: Vec
  equality per node).
- **RD-006** `greedy_search_persisted` agrees with
  `vamana::greedy_search` on the **top-1 result** and the **visited
  set** when driven with the same distance closure, on a real
  Vamana build over 40 synthetic 2D L2 points. Strong oracle for
  the loop body.
- **RD-007** greedy descends a distance gradient correctly on a
  chain; `list_size=3` carries the hops, top-1 lands on the target.
  (Earlier iteration with `list_size=1` hit a known chain-graph
  limitation — greedy can't traverse a chain when the frontier
  can't hold the next hop. Comment in the test documents the
  constraint.)
- **RD-008** `greedy_search_persisted` rejects `ItemPointer::INVALID`
  entry and `list_size = 0`.
- **RD-009** returned `frontier` is sorted ascending by distance
  (pairwise `<=` over windows).
- **RD-010** end-to-end: `build_and_persist_vamana` → construct a
  reader over `BuildOutput.persisted.chain` with
  `metadata.graph_degree_r` + `params.binary_word_count()` +
  `search_code_len()`, BFS from `metadata.entry_point` reaches every
  node. Confirms 5C-2 → 5D is a one-liner hand-off.

```
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured;
             573 filtered out; finished in 0.01s
```

Full diskann module: 64 tests pass (10 reader + 10 vacuum + 6 page
+ 13 tuple + 6 vamana + 11 persist + 8 build).
`cargo check --lib` clean (5 pre-existing dead-code warnings).

## Review focus

1. **Reader is a borrow, not an owned handle.** `PersistedGraphReader<'a>`
   holds `&DataPageChain`; the scan path will construct one per
   `amgettuple` (or per scan) and drop it at the end. Alternative
   was an owned handle that clones the chain — rejected because
   the pgrx integration will pass a buffer-pinned page view and a
   lifetime parameter is a cleaner expression of that. Reviewer
   confirm the borrow shape.
2. **`(R, W, C)` lives on the reader, not re-read per call.** The
   pgrx caller reads block 0 once, caches the triple, constructs
   one reader. Reviewer confirm this is the right granularity, vs.
   passing the triple to each `read_node` call (cheaper but noisier).
3. **`neighbors` returns `Vec<ItemPointer>`, not an iterator.** The
   original handoff sketch used `impl Iterator<Item = ItemPointer>
   + '_`. Returning an iterator requires either caching the decoded
   tuple inside the reader (stateful) or decoding every call + an
   `into_iter()` over an owned Vec (same cost). The owned Vec
   surfaces the decode error cleanly via `Result` and lets callers
   observe the length; it's the simpler shape at the same cost.
   Reviewer confirm the API choice.
4. **No tombstone filtering.** The reader deliberately does not
   skip nodes with `deleted = true`. Reason: ADR-047 keeps
   tombstone neighbors load-bearing for backlink discovery, and the
   scan layer owns the visibility decision (it needs the MVCC
   snapshot). If the reader filtered here, vacuum pass 2 orchestration
   (Phase 8B) would need a second, unfiltered reader. Reviewer
   flag if there's a cleaner home.
5. **Defensive `INVALID` skip inside `greedy_search_persisted`.**
   The fill-only invariant (ADR-045 Decision 3) says the filled
   prefix never contains `INVALID`. The loop still skips INVALID
   defensively because a future vacuum primitive (Phase 8A's
   `repair_neighbors` *does* swap in `INVALID` before compaction
   — but returns with the tail padded). Low-cost defense, keeps
   the loop robust to partially-vacuumed pages. Reviewer flag if
   this is over-defensive.
6. **`TidCandidate` tie-break on `(block, offset)`.** Makes the
   top-1 result deterministic when distances collide. Matches the
   intuition that `greedy_search_persisted` should be a pure
   function of `(chain, entry, L, query_dist)`. Reviewer confirm
   the tie-break choice (alphabetic-by-TID is arbitrary but stable;
   ignoring ties was the other option).

## Questions to answer

- **Should the reader expose a `decoded_len()` helper** so callers
  can sanity-check the `(R, W, C)` triple against the on-page
  tuple size before the first `read_node`? Currently the first
  decode fails with a length mismatch, which is clear enough.
  Held: no extra helper.
- **Should `greedy_search_persisted` take a `&mut VisitedSet`
  (caller-reusable) instead of allocating `HashSet<ItemPointer>`
  per call?** pgvectorscale reuses visited buffers across
  queries. Argument against: the scan layer can pass a
  caller-reusable set once this wires to pgrx; the current API is
  the minimum viable surface. Held: defer to Phase 6A.
- **Should `neighbors` return `&[ItemPointer]` via a decode
  cache?** Would require the reader to own a `RefCell<LruCache>`
  or similar. Argument against: no caching until there's a real
  access pattern to measure; the pgrx buffer manager caches pages,
  not decoded tuples. Held: no caching at this layer.

## Not doing in this packet

- **pgrx scan callbacks.** Phase 6A/B — deferred with the
  native-build lane.
- **Quantizer-backed query distance.** The scan layer will bind
  `Quantizer::prepare_scorer(...)` to an `Fn(ItemPointer) -> f32`;
  Phase 5D is distance-agnostic and tested with synthetic
  closures.
- **Prefetch / binary prefilter.** Phase 6A scan-algorithm shell.
- **Tombstone-aware iteration / MVCC.** Phase 6A owns this.

## Dependencies

- **ADR-045 ACCEPTED** — decode assumes fixed-length tuples at the
  stored `(R, W, C)` triple.
- **Phase 5B (11016)** — uses `VamanaNodeTuple::decode`.
- **Phase 5C-1 (11017)** — consumes `DataPageChain` as produced
  by `persist_vamana_graph`.
- **Phase 5C-2 (11018)** — the RD-010 bridge test drives
  `build_and_persist_vamana`.

## Companion packets

- **11014** — ADR-045 page-layout discipline.
- **11017** — Phase 5C-1 persist sequencer (reader's input).
- **11018** — Phase 5C-2 build orchestrator (reader's downstream
  consumer in tests).
- **11021** — Phase 8A vacuum primitives (sibling slice; both
  operate on the encoded tuple layer).

## Definition of ready

- ADR-045 ACCEPTED.
- 10 RD tests green (verified locally).
- Reviewer confirms the borrow-shape reader + `(R, W, C)`-on-the-reader
  API and the no-tombstone-filtering split.
- Phase 6A scan-algorithm shell does not start before this lands.

## Handoff notes

Two things make this a clean Phase 6A foundation:

1. `greedy_search_persisted` is line-for-line parallel to
   `vamana::greedy_search`. RD-006 locks that parallel with a
   side-by-side oracle test over a real Vamana build — any future
   divergence in the persisted loop (e.g., prefetch, binary
   prefilter) can be validated against the same oracle.
2. The distance closure is `Fn(ItemPointer) -> f32`. The scan
   layer's `query_dist` will be built from the metadata page's
   quantizer kinds (SRHT + grouped PQ4) + `Quantizer::prepare_scorer`
   — the reader doesn't need to know.

If reviewer wants the caller-reusable visited set *now* (instead
of in Phase 6A), it's a one-commit refactor: add a
`VisitedSet::clear()` helper and thread `&mut self` through the
API. Otherwise the minimum surface holds.
