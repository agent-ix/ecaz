# Task 17: DiskANN (Vamana) as Second Access Method

Status: **substantially complete on branch — callback buildout phases 1–9
landed by 2026-04-20, and 2026-04-21 follow-up recovery/signoff work
closed the remaining local DiskANN runtime smoke gaps.** The remaining
work is review, merge, and final faster-machine signoff rather than
another missing `ec_diskann` AM callback slice. Working branch:
`adr034-diskann-rebased`.

Executes ADR-034.

## Current status (2026-04-21)

### Landed on `adr034-diskann-rebased`

- Phase 1 (Quantizer trait seam), Phase 2 (storage move), and Phase 4
  (AM skeleton) are already absorbed into main via the refactor lane.
- The `ec_diskann` branch now has the full pgrx callback surface:
  - build: Phase 5C-3 (`ambuild`, persisted grouped codebooks)
  - scan: Phase 6B-1 / 6B-2
  - insert: Phase 7A through 7I
  - vacuum: Phase 8B-1 through 8B-3
  - planner: Phase 9 cost activation
- Prep items from packets 11004 / 11018 / 11023 / 11027 / 11028 are
  resolved in-tree.
- Option B `VisitedState` scratch is live for allocation-free scan
  reuse.
- ADR-045, ADR-046, and ADR-047 are all accepted and reflected by the
  landed code.
- Module path: `src/am/ec_diskann/`. SQL AM name: `ec_diskann`. FFI
  handler: `ec_diskann_handler`.

### 2026-04-21 recovery / signoff follow-up

- Real-corpus pg18 Recall@10 recovered to the expected range after the
  post-buildout runtime fixes in packet `11078`:
  - `list_size=64` → `0.9280`
  - `list_size=128` → `0.9310`
  - `list_size=200` → `0.9315`
- Vacuum/runtime follow-ups landed directly in the DiskANN AM:
  - packet `11081` bounded repair search to `R` and restored interrupt
    servicing in long scan/vacuum loops
  - packet `11082` removed the redundant exact-rerank stage from the
    vacuum repair frontier planner
- Local pg18 post-vacuum smoke now completes on the slower development
  machine (packet `11083`) using the canonical `ecaz` path on the real
  10k fixture:
  - pre-vacuum `Recall@10 = 0.9310` at `list_size=128`
  - delete 10% of corpus rows
  - `VACUUM (ANALYZE)` completes instead of stalling in the index phase
  - post-vacuum `Recall@10 = 0.9285` at `list_size=128`

### Remaining signoff work

- Review / merge hygiene on `adr034-diskann-rebased`
- Final performance benches on the faster machine
- Any reviewer-driven follow-up packets that surface during integration

### Resolved prep from the 2026-04-19 review batch

- [x] **Strengthen the live-tuple predicate** across reader / entry
      resolution / scan emission. Landed through packets 11023, 11027,
      and 11028 follow-up code already in this branch.
- [x] **Clear `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD` in V0 build.** Landed
      in the Phase 5C-2 / Phase 5C-3 prep follow-up that also updated
      the metadata tests.
- [x] **Refresh `plan/design/diskann-build-algorithm.md`** for V0
      hot-only persistence and the `MEDOID_SAMPLE_CAP = 1000`
      decision.

### Buildout phases, in order

1. **Prep** — complete.
2. **Phase 5C-3** — complete.
3. **Phase 6B** — complete.
4. **Phase 7** — complete.
5. **Phase 8B** — complete.
6. **Phase 9** — complete.

### Historical planning note

The detailed phase sections below are retained as the original buildout
plan. Current landed state is summarized above and in packets 11029
through 11045.

### ADR text edits for ACCEPTED flip

ADR-046 and ADR-047 both carry a "Frozen implementation rules
(2026-04-19 review)" section capturing the reviewer's answers. The
earlier step lists remain as historical context; where a frozen rule
differs, the frozen rule wins.

## Scope

Add `ecdiskann` as a second index access method alongside `tqhnsw`. The new
AM consumes the PqFastScan scoring kernel unchanged and wraps it around a
single-layer Vamana graph instead of a multi-layer HNSW graph. TurboQuant is
explicitly not supported by `ecdiskann` in this task — `tqhnsw` remains the
only AM that serves TurboQuant-format indexes.

The target outcome is a per-index Vamana AM:

```sql
CREATE INDEX ... USING ec_diskann (embedding ecvector_diskann_ip_ops)
    WITH (storage_format = 'pq_fastscan');
```

End-of-task: Postgres may naturally select `ec_diskann` for sufficiently
large ordered queries once the index exists, but there is still no
"default AM" flip. Users opt into Vamana by creating an `ec_diskann`
index explicitly.

## Out of scope

- Changing the PqFastScan scoring kernel (`src/quant/grouped_pq.rs`,
  `src/quant/prod.rs` FastScan paths). If a gap is discovered, file a
  follow-up against task 15 rather than silently mutating the kernel here.
- Touching TurboQuant code paths. That lane is task 16's territory.
- OPQ rotation (ADR-036), AQ/RVQ compression (ADR-037), LSQ refinement
  (ADR-038), SPANN (ADR-035). All orthogonal.
- Parallel index scan (ADR-040) for `ecdiskann`. Serial scan only in v0.
- Flipping the default AM to `ec_diskann`. Natural planner selection for
  explicitly-created `ec_diskann` indexes is in scope; making it the
  default index AM is not.
- Writing an auto-upgrade path from `tqhnsw` indexes to `ecdiskann`.
  Rebuild-only, same posture as ADR-032's `INDEX_FORMAT_V2` migration.

## Architectural constraints

- **Module-structure ADR owns stages 0–3.** ADR-041 designates task 17
  as the forcing function for the broader multi-AM / multi-quantizer
  reshape. Task 17 rolls in stage 0 (trait extraction), stage 1
  (`crate::storage::*` move), stage 2 (`am/*` → `am/tqhnsw/*` plus
  `am/common/`), and stage 3 (the new `am/ecdiskann/` module).
  Phases 1–4 below are the execution order for those stages; the
  ecdiskann AM proper begins at phase 4.
- **New AM module tree.** Final code home is `src/am/ecdiskann/`
  (peer to `src/am/tqhnsw/` after the stage-2 rename). A phase-4
  preview may land in `src/am/diskann/` ahead of stage 2 for wiring
  verification; that preview is moved into the post-stage-2 tree
  before review packet 11008 is filed.
- **PqFastScan kernel consumed via the stage-0 `Quantizer` trait.**
  Shared scoring infrastructure is reached through `&dyn Quantizer` /
  `&dyn PreparedQuery`; no new direct imports of tqhnsw-specific
  `GraphStorageDescriptor::PqFastScan` arms. If the trait seam does
  not yet cover a helper that ecdiskann needs, extend the trait in
  phase 1 rather than forking the helper.
- **New wire tag.** `INDEX_FORMAT_V4_DISKANN` (exact spelling TBD at
  implementation time) is a separate version-tag space from the
  existing `INDEX_FORMAT_V1_SCALAR` / `INDEX_FORMAT_V2_GROUPED` /
  `INDEX_FORMAT_V3_TURBO_HOT_COLD` values (the last belongs to task 16
  and occupies the V3 slot). Vamana metadata pages carry their own
  header distinct from the HNSW metadata page.
- **Lock ordering must ship with code.** ADR-046 (insert) and ADR-047
  (vacuum) are prerequisites for landing any mutation slice that
  rewrites neighbor arrays. Phases 7 and 8 below do not start before
  those ADRs are accepted.
- **Page-layout discipline per ADR-045.** Graph-AM tuples follow the
  five rules from ADR-045: per-index-constant fields live on the
  metadata page (not in every tuple), tuple bodies are codec-opaque
  raw byte runs, encoded tuple length is fixed per (R, W, C),
  persistence walks BFS-from-entry-point order, and the build-time
  persist sequence is placeholder-then-patch on `DataPageChain`.
  Phase 5 is the first consumer; future graph AMs inherit the same
  baseline.
- **Heap row type defaults to `ecvector` (full f32).** Task 16's
  `ecvector` row type (ADR-043 on task 16's branch, post-merge ADR
  number TBD) is the default heap vector format. ecdiskann stores
  `ecvector` in heap unless there is a concrete reason to keep a
  quantized vector in heap instead. Disk-based ANN topologies may
  eventually justify quant-in-heap for specific operating points
  (page-fetch cost beats rerank cost), but that is a per-scenario
  decision — not the default. Re-evaluate the page-layout and
  source-fetch sections of phases 4–8 against the post-task-16
  `ecvector` surface once task 16 merges.

## Phased subtasks

### Phase 0 — Planning (this packet)

- [x] Produce `plan/tasks/17-diskann-access-method.md` (this file).
- [x] Draft `spec/adr/ADR-046-vamana-insert-lock-ordering.md` as PROPOSED.
- [x] Draft `spec/adr/ADR-047-vamana-vacuum-lock-ordering.md` as PROPOSED.
- [x] Draft `plan/design/diskann-build-algorithm.md` with pgvectorscale
      references.
- [ ] Review packet 11001–11004 filed, review feedback processed.
- [ ] ADRs 042 and 043 move from PROPOSED to ACCEPTED before any
      insert/vacuum code lands.

### Phase 1 — Quantizer trait seam (ADR-041 stage 0)

Goal: route scan scoring through a `&dyn Quantizer` / `&dyn PreparedQuery`
seam so `ecdiskann` can consume PqFastScan without duplicating the
`GraphStorageDescriptor::PqFastScan` match arms. No file moves at this
stage — existing module layout stays put.

- [x] Define `Quantizer` and `PreparedQuery` (renamed `QueryScorer`)
      traits in `crate::quant` per ADR-041's "three load-bearing
      seams" section. Landed `7f49d1d`. Naming deviation
      (`QueryScorer` vs. ADR's `PreparedQuery`) documented in
      `src/quant/traits.rs` module header — collides with existing
      `crate::quant::prod::PreparedQuery` struct; revisit at stage 2.
- [x] Implement the traits for `ProdQuantizer` (TurboQuant family) and
      the grouped-PQ FastScan path. Landed `7f49d1d` (ProdQuantizer)
      and `10f5469` (PqFastScanQuantizer). `wire_format_version()`
      wired to `INDEX_FORMAT_V1_SCALAR` / `INDEX_FORMAT_V2_GROUPED`.
- [x] Thread grouped-PQ LUT scoring through `QueryScorer` at
      `src/am/scan.rs:2171` via `impl QueryScorer for
      PreparedGroupedScanQuery`. Static dispatch on concrete type,
      no `&dyn` indirection. TurboQuant split-payload sites (2006 and
      4026, `score_ip_from_parts(gamma, code)`) deferred to follow-up
      — payload shape `(gamma: f32, code: &[u8])` does not fit the
      flat `score(&[u8])` trait contract; see 11005 handoff for the
      three resolution options (prepend-copy, `score_split` trait
      method, or keep specialized). Per ADR-041 stage 0, ecdiskann can
      consume the trait without reaching into family internals with
      only site 2171 threaded; that is the minimum this phase needs.
- [x] ADR-041 validation gate: trivially satisfied. The only
      hot-path change is a static-dispatch move of the grouped-PQ
      LUT scoring body into a trait impl; TurboQuant paths exercised
      by `prepare_ip_query/d1536_b4` and `score_ip_encoded/d1536_b4`
      are untouched. No virtual call introduced. Re-evaluate the
      bench gate when the deferred TurboQuant split-payload sites
      land.

Review packet: 11005 (phase 1 landing).

### Phase 2 — Storage-primitive move (ADR-041 stage 1)

Goal: move cross-AM physical storage primitives under `crate::storage::*`
so both `tqhnsw` and `ecdiskann` can reach them without either owning
the other's page framework.

- [x] Move `src/am/page.rs::{ItemPointer, DataPage raw API, DataPageChain
      raw API, PAGE_HEADER_BYTES, FIRST_DATA_BLOCK_NUMBER,
      METADATA_BLOCK_NUMBER, HEAPTID_INLINE_CAPACITY, ITEM_POINTER_BYTES,
      raw_tuple_storage_bytes, page-byte/alignment helpers}` into
      `crate::storage::page`. Tqhnsw-typed convenience methods on
      `DataPage` / `DataPageChain` (`insert_element`, `read_neighbor`,
      …) remain in `src/am/page.rs` as additional inherent `impl`
      blocks, so AM-specific tuple codecs (`TqElementTuple`,
      `TqGroupedHotTuple`, `TqRerankTuple`, `TqGroupedCodebookTuple`,
      `TqNeighborTuple`, `TqTurboHotTuple`) stay in their owning AM.
      `crate::am::page` re-exports the moved primitives for tqhnsw
      consumers; phase 3 (`am/tqhnsw/` rename) rewrites those imports
      to point at `crate::storage::page` directly and drops the
      re-export.
- [x] Move `src/am/wal.rs` → `crate::storage::wal` (whole-file move via
      `git mv`; module declaration moved from `am::mod` to
      `storage::mod`; consumers add `use crate::storage::wal;`).
- [x] Drop the `crate::storage::metadata` shell (YAGNI per ADR-041).
      Task 16's V3 metadata changes did not introduce a second
      MetadataPage consumer; keep tqhnsw's `MetadataPage` in
      `crate::am::page` until ecdiskann's metadata page is wired in
      phase 5 and the shared shape is concrete.
- [x] `cargo check --lib` clean (5 pre-existing dead-code warnings),
      `cargo pgrx test pg17` 526 tests passing (+2 vs. pre-Phase-2
      baseline of 524 from new `storage::page::tests`).

Review packet: 11006 (phase 2 landing).

### Phase 3 — `am/tqhnsw/` rename and `am/common/` extraction (ADR-041 stage 2)

Goal: reshape `src/am/` so that a second AM is a peer module, not a
fork of the first. Lands as one atomic PR per ADR-041's "Stage 2 must
land as one atomic PR" rule.

- [ ] Rename `src/am/{build,cost,explain,graph,insert,options,
      routine,scan,scan_debug,search,shared,source,stats,stream,vacuum}.rs`
      into `src/am/tqhnsw/` and update all imports.
- [ ] Extract `src/am/common/` for cross-AM scaffolding: `cost.rs` shell
      (per-AM impls plug in), `explain.rs`, `stats.rs`, `stream.rs`,
      parallel-scan coordinator placeholder (ADR-040 forward-compat),
      and the shared reloption parser helpers.
- [ ] Float `StorageFormat` out of `am/tqhnsw/options.rs` and into a
      crate-level `quant::Family` enum. `tqhnsw` and `ecdiskann` both
      reference the shared enum; each AM carries its own reloption that
      resolves to it.
- [ ] `src/lib.rs` export surface (`pub mod bench_api`, re-exports) is
      preserved byte-for-byte at the public API level; only import paths
      move.
- [ ] ADR-041 validation gate: 50k warm real seam recall is bit-exact
      versus pre-reshape, latency is within noise. This is an
      equivalence check, not a performance run.

Review packet: 11007 (phase 3 landing).

### Phase 4 — `ecdiskann` AM skeleton and page-layout contract (ADR-041 stage 3 kickoff)

Goal: `ecdiskann` AM loads, registers, and rejects every real operation
until subsequent phases fill them in. A preview of this phase landed
in `src/am/diskann/` ahead of phases 1–3 to verify the
`IndexAmRoutine` wiring; that preview is re-homed under
`src/am/ecdiskann/` after phase 3 completes.

- [x] Preview: `src/am/diskann/{mod,routine,options,page,tuple}.rs`,
      `ecdiskann_handler` registered in `sql/bootstrap.sql`,
      `CREATE ACCESS METHOD ecdiskann` + `ecvector_ip_diskann_ops`
      opclass, six layout-assertion tests for the metadata page, eight
      for the node tuple, three pg_test cases for AM registration and
      unimplemented-error surfacing. Landed before stage-2 reshape.
- [ ] Re-home preview from `src/am/diskann/` into `src/am/ecdiskann/`
      as part of phase 3's atomic PR (imports follow the new
      `am/tqhnsw/` / `am/common/` / `quant::Family` paths).
- [ ] Remaining skeleton polish: implement `ecdiskann_amcostestimate`
      as a thin "disable_cost until phase 9" shim that plugs into the
      `am/common/cost.rs` shell instead of returning inline constants.
- [ ] Confirm `INDEX_FORMAT_V3_DISKANN` and `TQ_VAMANA_NODE_TAG` still
      hold unique integer values after the rename; layout-assertion
      tests run under the new module path.

Review packet: 11008 (phase 4 landing).

### Phase 5 — Build pipeline

Goal: `CREATE INDEX ... USING ecdiskann` produces a valid on-disk Vamana
graph plus PqFastScan codes. Scan, insert, and vacuum still error out.

Phase 5 is split into three sub-slices that land in order. Each is
small enough to review independently; together they cover the build
pipeline end to end.

#### Phase 5A — In-memory Vamana algorithm core

Pure-Rust algorithmic core in `src/am/diskann/vamana.rs`. No pgrx
deps, no page layout, no quantizer wiring — just the graph
construction with abstract distance closures.

- [ ] `VamanaGraph` adjacency list, `Candidate` struct,
      `greedy_search`, `robust_prune`, `approximate_medoid`,
      `build_vamana_graph`, `bfs_reachable`.
- [ ] Unit tests: prune respects max_degree, prune excludes
      α-dominated, greedy converges on linear chain, build is
      connected, approximate medoid within 10% of exact, synthetic
      Recall@10 ≥ 0.80 sanity floor.

#### Phase 5B — Slim tuple rewrite per ADR-045

Replace `VamanaNodeTuple` with the ADR-045 reference layout: 16-byte
header, drop `graph_degree_r` / `binary_word_count` /
`search_code_len` from per-tuple storage (read from metadata page),
single `primary_heaptid` + `has_overflow_heaptids` flag instead of
10 inline slots, fixed encoded length per (R, W, C). Reserve the
`rerank_tid` slot per ADR-045's note for ADR-044 forward compat.

- [ ] Rewrite `src/am/diskann/tuple.rs` to slim layout.
- [ ] Update layout-assertion tests; add a fixed-length invariant
      test.
- [ ] `cargo check` clean; `cargo pgrx test pg17` green.

#### Phase 5C — Build → persist plumbing

Tie the algorithm core, slim tuple, and metadata page together
inside the AM build callback. Persistence uses the placeholder-then-
patch pattern from ADR-045 Decision 5, walked in BFS-from-medoid
order (Decision 4).

Split into three sub-slices. 5C-1 and 5C-2 land independently of
the native-build lane; 5C-3 is **deferred** until the native-build
lane merges (see "Parallelization" below).

##### Phase 5C-1 — Persistence sequencer (pure-Rust)

- [x] `src/am/diskann/persist.rs`: `NodePayload`, `PersistedGraph`,
      `persist_vamana_graph(graph, medoid, page_size, payloads,
      R, W, C)`. Two-pass placeholder-then-patch on a fresh
      `DataPageChain`; BFS-from-medoid prefix + node-id suffix for
      the unreached set.
- [x] 11 unit tests (PE-001..PE-011) — pre-validation, single node,
      connected chain, disconnected, multi-page spill, length-
      alignment invariant, end-to-end with built graph.

Review packet: 11017.

##### Phase 5C-2 — Build orchestrator (pure-Rust)

- [x] `src/am/diskann/build.rs`: `BuildParams`, `BuildOutput`,
      `build_and_persist_vamana(params, payloads, build_dist)`.
      Runs medoid → build → persist → assembles
      `VamanaMetadataPage`. Locks `(W, C)` derivation rules at one
      site (ADR-045 reference layout).
- [x] 8 unit tests (BO-001..BO-008) — validation errors, derivation
      rules, end-to-end metadata + persisted graph, deterministic
      output for fixed seed, per-tuple round-trip with metadata-
      derived `(R, W, C)`.

Review packet: 11018.

##### Phase 5C-3 — pgrx ambuild + quantizer wiring (DEFERRED)

**Status: deferred until the native-HNSW-build lane merges.** The
SRHT + grouped-PQ training pipeline currently lives in
`src/am/build.rs`, the same file the native-build lane is actively
reshaping (see Phase 3 deferral). Landing 5C-3 now would either
(a) duplicate training inside `src/am/diskann/`, creating exactly
the divergence the Phase 3 defer was meant to prevent, or
(b) extract training into a shared module, which is a `src/am/build.rs`
edit and merges brutally with the native-build lane. Resume after
that lane lands and a shared training surface is concrete.

When unblocked:

- [ ] Port the training pipeline from the PqFastScan build path:
      SRHT rotation, grouped PQ codebook training, grouped PQ4
      encoding. Reached through whatever shared surface the
      native-build merge settles on (`Quantizer` trait or a
      dedicated training module under `src/quant/`).
- [ ] pgrx `ambuild` callback: heap scan via
      `table_index_build_scan`, per-row encode, drive
      `build_and_persist_vamana(...)`, write metadata page +
      DataPageChain to relation under one GenericXLog transaction.
- [ ] Codebook chain persistence: write the trained codebook into
      its own page chain, patch `metadata.grouped_codebook_head`.
- [ ] Build test: 10k-row fixture builds without error, metadata
      page decodes, entry point resolves to a live element, BFS
      from entry point reaches ≥ 95% of nodes.
- [ ] No live insert support yet — builds are snapshot-only.

Review packets: 11015 (5A vamana core), 11016 (5B slim tuple),
11017 (5C-1 persist), 11018 (5C-2 orchestrator), 11019 (5C-3
pgrx + quantizer landing — future).

#### Phase 5D — Persisted-graph reader (pure-Rust)

Goal: bridge Phase 5C-2's persisted artifacts to Phase 6 scan
without depending on the deferred 5C-3 quantizer wiring. A reader
that walks the `DataPageChain` by TID, decodes nodes with the
metadata-derived `(R, W, C)`, and exposes a TID-keyed greedy
search adapter (so 5A's `greedy_search` can be driven against a
real persisted graph rather than a `Vec<Vec<u32>>` adjacency list).

- [ ] `src/am/diskann/reader.rs`: `PersistedGraphReader { chain,
      metadata, R, W, C }`. Constructed from a `BuildOutput` (or
      from raw `DataPageChain` + `VamanaMetadataPage`).
      `read_node(tid) -> NodeView` (decoded VamanaNodeTuple +
      payload references). `neighbors(tid) -> impl Iterator<TID>`.
- [ ] `greedy_search_persisted(reader, entry_point, list_size,
      query_dist)` — TID-keyed analog of 5A's `greedy_search`.
- [ ] Tests on a built+persisted fixture: BFS over the reader
      reaches every node; greedy_search finds the entry point's
      nearest neighbor; deterministic for fixed inputs.

Isolated from native-build; can land before 5C-3.

Review packet: 11020.

### Phase 6 — Scan

Goal: `SET enable_seqscan = off; ORDER BY v <#> q LIMIT k` on a
`ecdiskann` index returns distance-ordered heap TIDs.

- [ ] Greedy best-first search helper in `src/am/ecdiskann/search.rs`
      mirroring the post-rename `src/am/tqhnsw/search.rs::beam_search`
      but on a single graph level. Visited set, frontier of size `L`
      (configurable via `ecdiskann.ef_search` GUC and per-index
      reloption, same control surface as ADR-016 for `tqhnsw`).
- [ ] `amgettuple` path in `src/am/ecdiskann/scan.rs`. Cursor-owned
      traversal state, same ownership discipline as the post-A3
      `tqhnsw` scan (ADR-015).
- [ ] Consume the PqFastScan scoring kernel through the phase-1
      `Quantizer` / `PreparedQuery` trait seam. Binary prefilter +
      grouped FastScan + heap-f32 rerank pipeline; no direct
      `GraphStorageDescriptor` match.
- [ ] Wire PgTAP regression test: real 10k fixture, measured
      `Recall@10 ≥ 0.90` at `ef_search = 128`, `R = 32`. Target
      `~0.95` preferred; below 0.90 is a phase-6 blocker.
- [ ] Planner `amcostestimate` stays gated under an ADR-011-style
      override for `ecdiskann` until recall is independently measured.

Review packet: 11010 (phase 6 landing).

### Phase 7 — Insert

Goal: `INSERT INTO ...` against a live `ecdiskann` index keeps graph
connectivity per ADR-046.

- [ ] Insert implementation in `src/am/ecdiskann/insert.rs`. Reuse
      grouped PQ4 encoding via `Quantizer::encode`. Candidate
      discovery via the same search helper as scan.
- [ ] α-pruning at insert time to choose the new node's neighbor list.
- [ ] Backlink installation on selected existing nodes per ADR-046
      lock ordering (single-layer, ordered-page backlink writes).
- [ ] Full-slot eviction policy: when a target node's neighbor list
      is full, prune with the same α rule rather than HNSW's
      score-only top-M eviction.
- [ ] `inserted_since_rebuild` bookkeeping on the Vamana metadata
      page.
- [ ] Concurrent-insert regression: 60-second `scripts/
      vacuum_concurrency_scratch.sh` analogue
      (`scripts/diskann_insert_concurrency_scratch.sh`).
- [ ] `build_source_column` rejection at insert, same posture as
      `tqhnsw`.

Review packet: 11011 (phase 7 landing).

### Phase 8 — Vacuum

Goal: `VACUUM` on a `ecdiskann` index strips dead heap TIDs and repairs
neighbor arrays per ADR-047.

ADR-047 must be ACCEPTED before the pgrx-side vacuum implementation
ships, but the pure-Rust tuple-level deletion / neighbor-repair
primitives can land independently — they have no quantizer or pgrx
conflict surface and can be filed as Phase 8A while the rest of the
vacuum work waits on ADR-047 sign-off and the native-build merge.

#### Phase 8A — Tuple-level deletion + repair primitives (pure-Rust)

Isolated from native-build. Self-contained in `src/am/diskann/`.

- [ ] `src/am/diskann/vacuum.rs`: `mark_deleted(&mut tuple)`,
      `strip_dead_primary_heaptid(&mut tuple, dead_pred)`,
      `repair_neighbors(&mut tuple, &dead_tid_set)` (replace dead
      neighbor TIDs with `INVALID`, compact prefix, update
      `neighbor_count`).
- [ ] Round-trip tests on encoded tuples: deletion state machine
      (alive → primary stripped → marked deleted), repair preserves
      length invariant (ADR-045 Decision 3 still holds), neighbor
      compaction is stable for fixed inputs.

Review packet: 11021.

#### Phase 8B — Three-pass vacuum (pgrx)

Blocks on: ADR-047 ACCEPTED + native-build merge (the vacuum
callback sits next to `ambuild` in the AM routine; same conflict
surface as Phase 5C-3).

- [ ] Vacuum implementation in `src/am/ecdiskann/vacuum.rs`. Three-pass
      mirror of the `tqhnsw` vacuum shape from ADR-027:
      pass 1 strips dead heap TIDs, pass 2 repairs neighbor arrays,
      pass 3 finalizes fully-dead nodes to `deleted = true`.
- [ ] Repair candidate selection for pass 2: replan under shared lock
      first (ADR-047 step 6), then fill under the page exclusive lock.
      Fill-only; no live-neighbor eviction under the write lock.
- [ ] 60-second `scripts/diskann_vacuum_concurrency_scratch.sh`
      analogue proving concurrent INSERT + scan + VACUUM safety.
- [ ] Post-vacuum recall smoke: delete 10% of rows, VACUUM, confirm
      Recall@10 ≥ 0.80 of pre-vacuum (matching FR-010-AC-2 for
      `tqhnsw`).

Review packet: 11012 (phase 8 landing).

### Phase 9 — Cost model and planner opt-in

- [x] Cost model entries in `src/am/ec_diskann/cost.rs` plugging into
      `src/am/common/cost.rs`. Landed in packet 11045 with empty-index
      gating plus natural small-table / large-table planner proofs.
- [x] Planner gate lift: the old `disable_cost` shim is gone for
      `ec_diskann`; the gate on `ec_hnsw` is unaffected.
- [ ] Strategy translation (FR-023) and custom EXPLAIN (FR-024) remain
      PG18 follow-up work rather than a blocker for the v0 DiskANN AM
      callback surface.

Review packet: 11045 (phase 9 landing).

## Parallelization

Serial spine: phase 1 → 2 → (3 deferred) → 4 → 5 → 6. Phase 3 (the
atomic `am/tqhnsw/` rename + `am/common/` extraction) collides with
in-flight native-HNSW-build work on `tqhnsw`. Deferred until that lane
quiets; phases 4–6 proceed against the preview `src/am/diskann/` tree
in the meantime, and the rehome to `src/am/ecdiskann/` becomes a
mechanical sed pass folded into phase 3 when it lands. Insert and
vacuum (phases 7 and 8) can run in parallel once phase 6 is green and
their ADRs are accepted, because they share no runtime state.

Critical path: phases 1–2 + 4–5 (in `src/am/diskann/`) → 6. Phase 3
gates the final rehome but not the build / scan implementation.

**Native-build conflict surface (historical, now resolved).** This note
was accurate on 2026-04-19 while the native-build lane was still
in-flight. That lane has since merged, the callback buildout landed on
`adr034-diskann-rebased`, and this branch now owns `src/am/ec_diskann/*`
end-to-end. The remaining off-limits rule is the narrower one used
during the landed slices: do not edit foreign-lane code outside
`src/am/ec_diskann/`, `plan/`, and `review/` without explicit
coordination.

## Owns

- Execution of ADR-034.
- Authoring ADR-045, ADR-046, ADR-047 (this planning packet).
- Cost model entries for `ecdiskann`.

## Dependencies

- **Task 15 (PqFastScan first-class)** — shipped on `main`. The AM
  will consume the existing `PqFastScanLayout` and FastScan scoring
  helpers without modification.
- **ADR-041** (module structure — gates phases 1–3; shipped on `main`).
- **ADR-045** (page-layout discipline — gates phase 5B/5C). Once
  ACCEPTED, the slim-tuple format is the V1 wire format; further
  changes require an ADR-045 amendment or a format bump.
- **ADR-046** (accepted before phase 7 insert work starts).
- **ADR-047** (accepted before phase 8 vacuum work starts).

## Unblocks

- ADR-035 (SPANN). Vamana plus binary sidecar is the expected inner
  search shard for a SPANN implementation; `ecdiskann` is the
  prerequisite for that track.
- Informed comparison against pgvectorscale and VectorChord on real
  corpora, which in turn informs whether to flip the default AM.

## Definition of done

- `CREATE INDEX ... USING ecdiskann (embedding vector_ip_ops)
  WITH (storage_format = 'pq_fastscan')` succeeds on a 50k-row fixture.
- Insert + vacuum round-trip survives the concurrency scratch script.
- Recall@10 ≥ 0.90 at default tuning on the real 10k fixture
  (baseline); ~0.95 preferred.
- No `tqhnsw` code paths altered except for shared helpers that had to
  be factored out behind a tracked refactor subtask.
- ADR-034 moves from PROPOSED to ACCEPTED.
