# Review Request: Phase 5C-1 — Placeholder-then-Patch Persistence Sequencer

Branch: `adr034-diskann-access-method`
Author: coder-2
Companion to: 11014 (ADR-045), 11015 (Phase 5A), 11016 (Phase 5B)

## What this slice is

First sub-slice of task 17 **Phase 5C** (build → persist plumbing).
Pure-Rust sequencer that takes a built `VamanaGraph` plus per-node
payloads and writes them into a `DataPageChain` using the ADR-045
Decision 5 placeholder-then-patch pattern. No pgrx, no quantizer —
testable in isolation.

Phase 5C-2 (BFS-from-medoid + disconnected handling) is already
inlined here because `bfs_reachable` landed in Phase 5A and the
"persist unreached nodes too" rule is trivial given BFS. Phase 5C-3
(pgrx build callback: heap scan + SRHT/PQ + drive this module +
GenericXLog) is still ahead.

## Scope

- `src/am/diskann/persist.rs` — new file, 475 lines incl. 11 tests.
- `src/am/diskann/mod.rs` — `pub mod persist;` declaration.

No other source files touched. No metadata-page changes. No changes
to `tuple.rs`, `vamana.rs`, or `storage::page`.

## What changed

### Persistence sequence (ADR-045 Decisions 4 + 5)

1. **Pre-validate.** Non-empty graph, payload count matches node
   count, medoid in range, `graph.max_degree ≤ R`, every payload's
   `binary_words.len() == W` and `search_code.len() == C`.
2. **Compute order.** `bfs_reachable(graph, medoid)` yields the
   scan-locality prefix; any nodes not in the BFS set get appended
   in node-id order. Disconnected components are persisted, not
   rejected — the build callback (5C-3) is the right place to log
   the reachable fraction.
3. **Pass 1 — placeholders.** For each node in order, encode a
   fixed-length tuple with neighbor slots `INVALID` and
   `neighbor_count = 0`, insert via `DataPageChain::insert_raw_tuple`,
   record the returned TID into a dense `Vec<ItemPointer>` keyed by
   node id.
4. **Pass 2 — patch.** Walk the same order. For each node, re-encode
   with the resolved neighbor TIDs from the pass-1 map and replace
   the placeholder via `DataPageChain::update_raw_tuple`. Same encoded
   length is guaranteed by ADR-045 Decision 3 (`encoded_len(R, W, C)`
   is a pure function of metadata constants).

### Public API

```rust
pub struct NodePayload {
    pub primary_heaptid: ItemPointer,
    pub binary_words: Vec<u64>,   // len = W
    pub search_code: Vec<u8>,     // len = C
}

pub struct PersistedGraph {
    pub chain: DataPageChain,
    pub node_to_tid: Vec<ItemPointer>, // dense, keyed by node id
    pub entry_point_tid: ItemPointer,  // TID of the medoid
    pub persistence_order: Vec<u32>,   // BFS prefix + unreached suffix
    pub unreached: Vec<u32>,           // BFS-unreached node ids
}

pub fn persist_vamana_graph(
    graph: &VamanaGraph,
    medoid: u32,
    page_size: usize,
    payloads: &[NodePayload],
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
) -> Result<PersistedGraph, String>;
```

The `(R, W, C)` triple is threaded through from the caller (same
pattern as `VamanaNodeTuple::encode/decode/validate` from Phase 5B).
Callers read these off the metadata page.

### Tests (11, all green)

- **PE-001** empty graph errors
- **PE-002** payload count mismatch errors
- **PE-003** medoid out of range errors
- **PE-004** payload body size mismatch errors (W / C)
- **PE-005** `graph.max_degree > R` errors
- **PE-006** single node persists; BFS reaches everything
- **PE-007** connected 5-node chain; full round-trip decode recovers
  neighbors and payload bytes
- **PE-008** disconnected graph persists unreached nodes in node-id
  order after the BFS prefix; `unreached` field populated
- **PE-009** multi-page chain (tiny `page_size` forces spill); every
  node still decodes correctly across page boundaries
- **PE-010** placeholder-patched length invariant: total bytes
  allocated in pass 1 equal total bytes after pass 2 (ADR-045
  Decision 3 witnessed end-to-end)
- **PE-011** end-to-end with a real built graph (100 nodes, synthetic
  2D L2): build → persist → decode every tuple → verify neighbors
  and payload bytes match

```
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured;
             544 filtered out; finished in 0.01s
```

`cargo check --lib` clean (5 pre-existing dead-code warnings only).
Full diskann module: 36 tests pass (6 page + 13 tuple + 6 vamana +
11 persist).

## Review focus

1. **BFS prefix + unreached suffix vs. error on disconnected.** The
   sequencer persists every live node — disconnected components get
   appended after the BFS reached set and marked in `unreached`. The
   alternative (hard-fail) would surface algorithm regressions
   earlier but would also make build brittle on sparse inputs
   (pathological small tables, early prototypes). I went with
   persist-and-report; the build callback at 5C-3 decides whether to
   warn/fatal based on the reachable fraction. Reviewer preference?
2. **Two passes over the order, not one pass with fixup.** A single
   pass that resolves neighbor TIDs when both endpoints have been
   placed would save half the encode work but doesn't generalize
   when the graph has back-edges (nearly always the case after
   robust_prune). Two passes keeps the invariant simple: "after
   pass 1 every node has a TID; pass 2 is pure patching."
3. **Body-size pre-validation at the top.** Encode would catch these
   too (`VamanaNodeTuple::validate`), but pre-validating up front
   means we error before any DataPage allocations. Reviewer call:
   trust encode-time validation only, or keep the early check?
4. **`PersistedGraph.persistence_order` field.** Metadata page does
   not store this; it's diagnostic-only. Reviewer call: keep for
   observability (and for tests), or drop?
5. **`max_degree > R` as an error, not a silent cap.** A well-built
   Vamana graph guarantees `|neighbors[i]| ≤ R` because
   `robust_prune` caps at `max_degree`. Getting here means a bug
   upstream — fail loudly.

## Questions to answer

- **Should pass 2 use `update_raw_tuple` or rebuild the page?** Using
  `update_raw_tuple` is what ADR-045 Decision 5 specifies — same
  length, same slot — and keeps the `insert_raw_tuple` → TID mapping
  valid. A full rebuild would make pass 1 pure "allocate TIDs" which
  is cleaner but loses the guarantee that pass 1's TIDs survive.
  Held with `update_raw_tuple`.
- **Do we want a "dry run" that returns just the TID layout without
  writing bodies?** Would be useful for size estimation before
  committing to a GenericXLog transaction. YAGNI at 5C-3 time unless
  page budgeting becomes an issue; deferred.

## Not doing in this packet

- **pgrx build callback wiring.** Phase 5C-3: `ambuild` / `ambuildempty`,
  heap-scan plumbing, per-row SRHT / PQ encode, driving
  `persist_vamana_graph` from the resulting `NodePayload` vec, and
  wrapping the DataPage writes in a GenericXLog transaction.
- **Metadata page population.** 5C-3 writes `entry_point_tid` and the
  chain start into `VamanaMetadataPage` under the same transaction.
- **Concurrent/incremental insert.** Phase 7 (live insert) territory.

## Dependencies

- **ADR-045 ACCEPTED** — gate: Decisions 3, 4, 5 are what make this
  sequencer sound.
- **Phase 5A (11015)** — consumes `VamanaGraph` and `bfs_reachable`.
- **Phase 5B (11016)** — consumes `VamanaNodeTuple::encoded_len`,
  `placeholder`, `encode`, `decode`, and the fixed-length invariant.
- **`storage::page::DataPageChain`** — `insert_raw_tuple` /
  `update_raw_tuple` raw API landed under ADR-041 stage 1.

## Companion packets

- **11014** — ADR-045 page-layout discipline.
- **11015** — Phase 5A vamana algorithm core.
- **11016** — Phase 5B slim tuple.
- **11018** — Phase 5C-3 pgrx build callback (future).

## Definition of ready

- ADR-045 ACCEPTED.
- 11 PE tests green (verified locally).
- Reviewer confirms persist-and-report semantics for disconnected
  components.
- Phase 5C-3 does not start before this lands.

## Handoff notes

The sequencer is the smallest piece of Phase 5C and the one with the
clearest invariant to test. The placeholder-then-patch pattern
reduces to a loop-and-encode in pass 1 and a loop-and-re-encode in
pass 2 — no exotic control flow. The non-obvious pieces are:

- The BFS prefix + unreached suffix order (lets the scan path hit
  hot pages without rejecting unusual inputs).
- The body-size pre-validation (fail before any page allocation).
- The length-invariant test PE-010 (witnesses ADR-045 Decision 3
  end-to-end, not just at the tuple level).

If reviewer pushes back on the disconnected-graph semantics, the fix
is one line in `persist_vamana_graph`: return `Err` instead of
populating `unreached`. The rest of the sequencer does not change.
