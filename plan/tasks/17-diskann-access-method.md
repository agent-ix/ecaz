# Task 17: DiskANN (Vamana) as Second Access Method

Status: proposed — planning slice only. No code changes land under this task
until the phased subtasks below are individually accepted.

Executes ADR-034.

## Scope

Add `tqdiskann` as a second index access method alongside `tqhnsw`. The new
AM consumes the PqFastScan scoring kernel unchanged and wraps it around a
single-layer Vamana graph instead of a multi-layer HNSW graph. TurboQuant is
explicitly not supported by `tqdiskann` in this task — `tqhnsw` remains the
only AM that serves TurboQuant-format indexes.

The target outcome is a per-index opt-in Vamana AM:

```sql
CREATE INDEX ... USING tqdiskann (embedding vector_ip_ops)
    WITH (storage_format = 'pq_fastscan');
```

End-of-task: planner still opts `tqdiskann` in only when the user asks for
it explicitly, the same way `tqhnsw` stays gated under ADR-011 today. No
default-AM flip.

## Out of scope

- Changing the PqFastScan scoring kernel (`src/quant/grouped_pq.rs`,
  `src/quant/prod.rs` FastScan paths). If a gap is discovered, file a
  follow-up against task 15 rather than silently mutating the kernel here.
- Touching TurboQuant code paths. That lane is task 16's territory.
- OPQ rotation (ADR-036), AQ/RVQ compression (ADR-037), LSQ refinement
  (ADR-038), SPANN (ADR-035). All orthogonal.
- Parallel index scan (ADR-040) for `tqdiskann`. Serial scan only in v0.
- Flipping the default AM to `tqdiskann`. Explicit opt-in stays.
- Writing an auto-upgrade path from `tqhnsw` indexes to `tqdiskann`.
  Rebuild-only, same posture as ADR-032's `INDEX_FORMAT_V2` migration.

## Architectural constraints

- **Module-structure ADR owns stages 0–3.** ADR-041 designates task 17
  as the forcing function for the broader multi-AM / multi-quantizer
  reshape. Task 17 rolls in stage 0 (trait extraction), stage 1
  (`crate::storage::*` move), stage 2 (`am/*` → `am/tqhnsw/*` plus
  `am/common/`), and stage 3 (the new `am/tqdiskann/` module).
  Phases 1–4 below are the execution order for those stages; the
  tqdiskann AM proper begins at phase 4.
- **New AM module tree.** Final code home is `src/am/tqdiskann/`
  (peer to `src/am/tqhnsw/` after the stage-2 rename). A phase-4
  preview may land in `src/am/diskann/` ahead of stage 2 for wiring
  verification; that preview is moved into the post-stage-2 tree
  before review packet 11008 is filed.
- **PqFastScan kernel consumed via the stage-0 `Quantizer` trait.**
  Shared scoring infrastructure is reached through `&dyn Quantizer` /
  `&dyn PreparedQuery`; no new direct imports of tqhnsw-specific
  `GraphStorageDescriptor::PqFastScan` arms. If the trait seam does
  not yet cover a helper that tqdiskann needs, extend the trait in
  phase 1 rather than forking the helper.
- **New wire tag.** `INDEX_FORMAT_V3_DISKANN` (exact spelling TBD at
  implementation time) is a separate version-tag space from the
  `tqhnsw` `INDEX_FORMAT_V1_SCALAR` / `INDEX_FORMAT_V2_GROUPED` values.
  Vamana metadata pages carry their own header distinct from the HNSW
  metadata page.
- **Lock ordering must ship with code.** ADR-042 (insert) and ADR-043
  (vacuum) are prerequisites for landing any mutation slice that
  rewrites neighbor arrays. Phases 7 and 8 below do not start before
  those ADRs are accepted.

## Phased subtasks

### Phase 0 — Planning (this packet)

- [x] Produce `plan/tasks/17-diskann-access-method.md` (this file).
- [x] Draft `spec/adr/ADR-042-vamana-insert-lock-ordering.md` as PROPOSED.
- [x] Draft `spec/adr/ADR-043-vamana-vacuum-lock-ordering.md` as PROPOSED.
- [x] Draft `plan/design/diskann-build-algorithm.md` with pgvectorscale
      references.
- [ ] Review packet 11001–11004 filed, review feedback processed.
- [ ] ADRs 042 and 043 move from PROPOSED to ACCEPTED before any
      insert/vacuum code lands.

### Phase 1 — Quantizer trait seam (ADR-041 stage 0)

Goal: route scan scoring through a `&dyn Quantizer` / `&dyn PreparedQuery`
seam so `tqdiskann` can consume PqFastScan without duplicating the
`GraphStorageDescriptor::PqFastScan` match arms. No file moves at this
stage — existing module layout stays put.

- [ ] Define `Quantizer` and `PreparedQuery` traits in `crate::quant`
      per ADR-041's "three load-bearing seams" section.
- [ ] Implement the traits for `ProdQuantizer` (TurboQuant family) and
      the grouped-PQ FastScan path. Wire `wire_format_version()` to the
      existing `INDEX_FORMAT_V1_SCALAR` / `INDEX_FORMAT_V2_GROUPED`
      constants.
- [ ] Thread `&dyn Quantizer` + `&dyn PreparedQuery` through the
      `src/am/scan.rs` scoring call sites that currently dispatch on
      `match GraphStorageDescriptor`. Leave the match on the outside as
      the selector; collapse the per-arm scoring work to a single
      trait-object call.
- [ ] ADR-041 validation gate: rerun the task-08 hot-path benchmarks
      (`prepare_ip_query/d1536_b4`, `score_ip_encoded/d1536_b4`) and
      confirm the trait-indirected scoring path matches the pre-reshape
      numbers within ±5% noise. If the virtual call shows up in
      profiles, pivot to generics (`scan.rs::<Q: Quantizer>`) per the
      ADR-041 "Revisit if the virtual call shows up in profiles" note.

Review packet: 11005 (phase 1 landing).

### Phase 2 — Storage-primitive move (ADR-041 stage 1)

Goal: move cross-AM physical storage primitives under `crate::storage::*`
so both `tqhnsw` and `tqdiskann` can reach them without either owning
the other's page framework.

- [ ] Move `src/am/page.rs::{ItemPointer, PageHeader, DataPage helpers,
      PAGE_HEADER_BYTES, FIRST_DATA_BLOCK_NUMBER, METADATA_BLOCK_NUMBER}`
      into `crate::storage::page`. Leave AM-specific tuple codecs
      (`TqElementTuple`, `TqGroupedHotTuple`, `TqRerankTuple`,
      `TqGroupedCodebookTuple`) where they are for now.
- [ ] Move `src/am/wal.rs` → `crate::storage::wal`.
- [ ] Carve out a `crate::storage::metadata` shell for the shared
      metadata-page framework. tqhnsw's `MetadataPage` keeps its
      current fields but inherits its page-header/special-area helpers
      from the shared shell.
- [ ] Import churn is allowed but everything must still compile after
      the move. `cargo test` + `cargo pgrx test pg17` unchanged count
      of passing tests.

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
      crate-level `quant::Family` enum. `tqhnsw` and `tqdiskann` both
      reference the shared enum; each AM carries its own reloption that
      resolves to it.
- [ ] `src/lib.rs` export surface (`pub mod bench_api`, re-exports) is
      preserved byte-for-byte at the public API level; only import paths
      move.
- [ ] ADR-041 validation gate: 50k warm real seam recall is bit-exact
      versus pre-reshape, latency is within noise. This is an
      equivalence check, not a performance run.

Review packet: 11007 (phase 3 landing).

### Phase 4 — `tqdiskann` AM skeleton and page-layout contract (ADR-041 stage 3 kickoff)

Goal: `tqdiskann` AM loads, registers, and rejects every real operation
until subsequent phases fill them in. A preview of this phase landed
in `src/am/diskann/` ahead of phases 1–3 to verify the
`IndexAmRoutine` wiring; that preview is re-homed under
`src/am/tqdiskann/` after phase 3 completes.

- [x] Preview: `src/am/diskann/{mod,routine,options,page,tuple}.rs`,
      `tqdiskann_handler` registered in `sql/bootstrap.sql`,
      `CREATE ACCESS METHOD tqdiskann` + `tqvector_ip_diskann_ops`
      opclass, six layout-assertion tests for the metadata page, eight
      for the node tuple, three pg_test cases for AM registration and
      unimplemented-error surfacing. Landed before stage-2 reshape.
- [ ] Re-home preview from `src/am/diskann/` into `src/am/tqdiskann/`
      as part of phase 3's atomic PR (imports follow the new
      `am/tqhnsw/` / `am/common/` / `quant::Family` paths).
- [ ] Remaining skeleton polish: implement `tqdiskann_amcostestimate`
      as a thin "disable_cost until phase 9" shim that plugs into the
      `am/common/cost.rs` shell instead of returning inline constants.
- [ ] Confirm `INDEX_FORMAT_V3_DISKANN` and `TQ_VAMANA_NODE_TAG` still
      hold unique integer values after the rename; layout-assertion
      tests run under the new module path.

Review packet: 11008 (phase 4 landing).

### Phase 5 — Build pipeline

Goal: `CREATE INDEX ... USING tqdiskann` produces a valid on-disk Vamana
graph plus PqFastScan codes. Scan, insert, and vacuum still error out.

- [ ] Port the training pipeline from the PqFastScan build path: SRHT
      rotation, grouped PQ codebook training, grouped PQ4 encoding.
      Reached through the phase-1 `Quantizer` trait rather than direct
      `ProdQuantizer` imports.
- [ ] Implement Vamana graph construction (α-pruning) per
      `plan/design/diskann-build-algorithm.md`. Initial ordering: random
      permutation over vector indices (matches pgvectorscale).
- [ ] First pass: build with `α = 1.0`, second pass: refine with
      configured `α` (typically 1.2). Two passes, same graph.
- [ ] Persist graph one page-aligned node at a time. Node layout:
      `[element header][binary sidecar?][grouped search code][R neighbor TIDs]`.
      Cold rerank payload lives on a parallel page chain, reused from
      the PqFastScan hot/cold split.
- [ ] Metadata page finalization: entry-point TID = medoid of the
      quantized dataset (approximate medoid per pgvectorscale).
- [ ] Build test: 10k-row fixture builds without error, metadata page
      decodes, entry point resolves to a live element.
- [ ] No live insert support yet — builds are snapshot-only.

Review packet: 11009 (phase 5 landing).

### Phase 6 — Scan

Goal: `SET enable_seqscan = off; ORDER BY v <#> q LIMIT k` on a
`tqdiskann` index returns distance-ordered heap TIDs.

- [ ] Greedy best-first search helper in `src/am/tqdiskann/search.rs`
      mirroring the post-rename `src/am/tqhnsw/search.rs::beam_search`
      but on a single graph level. Visited set, frontier of size `L`
      (configurable via `tqdiskann.ef_search` GUC and per-index
      reloption, same control surface as ADR-016 for `tqhnsw`).
- [ ] `amgettuple` path in `src/am/tqdiskann/scan.rs`. Cursor-owned
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
      override for `tqdiskann` until recall is independently measured.

Review packet: 11010 (phase 6 landing).

### Phase 7 — Insert

Goal: `INSERT INTO ...` against a live `tqdiskann` index keeps graph
connectivity per ADR-042.

- [ ] Insert implementation in `src/am/tqdiskann/insert.rs`. Reuse
      grouped PQ4 encoding via `Quantizer::encode`. Candidate
      discovery via the same search helper as scan.
- [ ] α-pruning at insert time to choose the new node's neighbor list.
- [ ] Backlink installation on selected existing nodes per ADR-042
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

Goal: `VACUUM` on a `tqdiskann` index strips dead heap TIDs and repairs
neighbor arrays per ADR-043.

- [ ] Vacuum implementation in `src/am/tqdiskann/vacuum.rs`. Three-pass
      mirror of the `tqhnsw` vacuum shape from ADR-027:
      pass 1 strips dead heap TIDs, pass 2 repairs neighbor arrays,
      pass 3 finalizes fully-dead nodes to `deleted = true`.
- [ ] Repair candidate selection for pass 2: replan under shared lock
      first (ADR-043 step 6), then fill under the page exclusive lock.
      Fill-only; no live-neighbor eviction under the write lock.
- [ ] 60-second `scripts/diskann_vacuum_concurrency_scratch.sh`
      analogue proving concurrent INSERT + scan + VACUUM safety.
- [ ] Post-vacuum recall smoke: delete 10% of rows, VACUUM, confirm
      Recall@10 ≥ 0.80 of pre-vacuum (matching FR-010-AC-2 for
      `tqhnsw`).

Review packet: 11012 (phase 8 landing).

### Phase 9 — Cost model and planner opt-in

- [ ] Cost model entries in `src/am/tqdiskann/cost.rs` plugging into
      the phase-3 `src/am/common/cost.rs` shell. Model inputs: `R`,
      `L`, `ef_search`, reltuples, `index_pages`, entry-point depth.
      Unit tests match the shape of the `tqhnsw` cost-model tests.
- [ ] Strategy translation (FR-023) and custom EXPLAIN (FR-024) opt-in
      for `tqdiskann`, following the PG18 scaffolding already in
      `src/am/common/{explain,stats}.rs`.
- [ ] Planner gate lift: remove ADR-011-style override for
      `tqdiskann` specifically once phase 6 recall is signed off.
      The gate on `tqhnsw` is unaffected.

Review packet: 11013 (phase 9 landing).

## Parallelization

Serial spine: phase 1 → 2 → 3 → 4 → 5 → 6. Insert and vacuum (phases 7
and 8) can run in parallel once phase 6 is green and their ADRs are
accepted, because they share no runtime state.

Critical path: phases 1–3 (ADR-041 module-structure stages) gate
phases 4–6 and everything downstream.

## Owns

- Execution of ADR-034.
- Authoring ADR-042, ADR-043 (this planning packet).
- Cost model entries for `tqdiskann`.

## Dependencies

- **Task 15 (PqFastScan first-class)** — shipped on `main`. The AM
  will consume the existing `PqFastScanLayout` and FastScan scoring
  helpers without modification.
- **ADR-041** (module structure — gates phases 1–3; shipped on `main`).
- **ADR-042** (accepted before phase 7 insert work starts).
- **ADR-043** (accepted before phase 8 vacuum work starts).

## Unblocks

- ADR-035 (SPANN). Vamana plus binary sidecar is the expected inner
  search shard for a SPANN implementation; `tqdiskann` is the
  prerequisite for that track.
- Informed comparison against pgvectorscale and VectorChord on real
  corpora, which in turn informs whether to flip the default AM.

## Definition of done

- `CREATE INDEX ... USING tqdiskann (embedding vector_ip_ops)
  WITH (storage_format = 'pq_fastscan')` succeeds on a 50k-row fixture.
- Insert + vacuum round-trip survives the concurrency scratch script.
- Recall@10 ≥ 0.90 at default tuning on the real 10k fixture
  (baseline); ~0.95 preferred.
- No `tqhnsw` code paths altered except for shared helpers that had to
  be factored out behind a tracked refactor subtask.
- ADR-034 moves from PROPOSED to ACCEPTED.
