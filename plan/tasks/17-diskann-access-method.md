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

- **New AM module tree.** Code lives under `src/am/diskann/` parallel to
  existing AM code. Do not entangle with `tqhnsw`-specific structure in
  `src/am/scan.rs`, `src/am/insert.rs`, or `src/am/vacuum.rs`.
- **PqFastScan kernel consumed as-is.** Shared scoring infrastructure
  (`PqFastScanLayout`, the FastScan accumulate path, binary prefilter,
  heap-f32 rerank mode) is imported from the existing modules. If the
  shared scoring code is buried inside a `tqhnsw`-specific enum branch
  (e.g., `GraphStorageDescriptor::PqFastScan` arms in
  `src/am/scan.rs`), propose a refactoring subtask under this task before
  cross-consuming the inner helpers.
- **New wire tag.** `INDEX_FORMAT_V3_DISKANN` (exact spelling TBD at
  implementation time) is a separate version-tag space from the
  `tqhnsw` `INDEX_FORMAT_V1_SCALAR` / `INDEX_FORMAT_V2_GROUPED` values.
  Vamana metadata pages carry their own header distinct from the HNSW
  metadata page.
- **Lock ordering must ship with code.** ADR-041 (insert) and ADR-042
  (vacuum) are prerequisites for landing any mutation slice that
  rewrites neighbor arrays. Phases 4 and 5 below do not start before
  those ADRs are accepted.

## Phased subtasks

### Phase 0 — Planning (this packet)

- [x] Produce `plan/tasks/17-diskann-access-method.md` (this file).
- [x] Draft `spec/adr/ADR-041-vamana-insert-lock-ordering.md` as PROPOSED.
- [x] Draft `spec/adr/ADR-042-vamana-vacuum-lock-ordering.md` as PROPOSED.
- [x] Draft `plan/design/diskann-build-algorithm.md` with pgvectorscale
      references.
- [ ] Review packet 11001–11004 filed, review feedback processed.
- [ ] ADRs 041 and 042 move from PROPOSED to ACCEPTED before any
      insert/vacuum code lands.

### Phase 1 — AM skeleton and page-layout contract

Goal: `tqdiskann` AM loads, registers, and rejects every real operation
until subsequent phases fill them in.

- [ ] `src/am/diskann/mod.rs` with a `tqdiskann_handler` entry point
      parallel to `src/am/routine.rs`.
- [ ] `tqdiskann_handler` registered in `sql/bootstrap.sql` plus a
      `CREATE ACCESS METHOD tqdiskann` definition and matching opclass.
- [ ] Vamana-specific metadata-page struct drafted in
      `src/am/diskann/page.rs`, mirroring `src/am/page.rs::MetadataPage`
      but with single-layer fields (`graph_degree_R`,
      `build_list_size_L`, `alpha` stored as `f32`, entry-point TID,
      etc.).
- [ ] `INDEX_FORMAT_V3_DISKANN` wire-tag constant plus layout assertions
      (`LA-*` lines in tests mirroring the existing
      `INDEX_FORMAT_V2_GROUPED` coverage).
- [ ] Vamana neighbor tuple layout draft: fixed-capacity `R` neighbor
      slot list per node, no per-layer segmentation.
- [ ] Scan / insert / vacuum return "unimplemented for `tqdiskann`"
      errors at this phase. Build returns "unimplemented".
- [ ] `tqdiskann` reloption set: `graph_degree` (default 32),
      `build_list_size` (default 100), `alpha` (`f32`, default
      1.2), `storage_format` validated as `pq_fastscan` only.

Review packet: 11005 (phase 1 landing).

### Phase 2 — Build pipeline

Goal: `CREATE INDEX ... USING tqdiskann` produces a valid on-disk Vamana
graph plus PqFastScan codes. Scan, insert, and vacuum still error out.

- [ ] Port the training pipeline from the PqFastScan build path: SRHT
      rotation, grouped PQ codebook training, grouped PQ4 encoding.
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

Review packet: 11006 (phase 2 landing).

### Phase 3 — Scan

Goal: `SET enable_seqscan = off; ORDER BY v <#> q LIMIT k` on a
`tqdiskann` index returns distance-ordered heap TIDs.

- [ ] Greedy best-first search helper in `src/am/diskann/search.rs`
      mirroring `src/am/search.rs::beam_search` but on a single graph
      level. Visited set, frontier of size `L` (configurable via
      `tqdiskann.ef_search` GUC and per-index reloption, same control
      surface as ADR-016 for `tqhnsw`).
- [ ] `amgettuple` path in `src/am/diskann/scan.rs`. Cursor-owned
      traversal state, same ownership discipline as the post-A3
      `tqhnsw` scan (ADR-015).
- [ ] Consume the existing PqFastScan scoring kernel for candidate
      scoring. Binary prefilter + grouped FastScan + heap-f32 rerank
      pipeline. Do not duplicate the scoring code.
- [ ] Wire PgTAP regression test: real 10k fixture, measured
      `Recall@10 ≥ 0.90` at `ef_search = 128`, `R = 32`. Target
      `~0.95` preferred; below 0.90 is a phase-3 blocker.
- [ ] Planner `amcostestimate` stays gated under an ADR-011-style
      override for `tqdiskann` until recall is independently measured.

Review packet: 11007 (phase 3 landing).

### Phase 4 — Insert

Goal: `INSERT INTO ...` against a live `tqdiskann` index keeps graph
connectivity per ADR-041.

- [ ] Insert implementation in `src/am/diskann/insert.rs`. Reuse
      grouped PQ4 encoding. Candidate discovery via the same search
      helper as scan.
- [ ] α-pruning at insert time to choose the new node's neighbor list.
- [ ] Backlink installation on selected existing nodes per ADR-041
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

Review packet: 11008 (phase 4 landing).

### Phase 5 — Vacuum

Goal: `VACUUM` on a `tqdiskann` index strips dead heap TIDs and repairs
neighbor arrays per ADR-042.

- [ ] Vacuum implementation in `src/am/diskann/vacuum.rs`. Three-pass
      mirror of the `tqhnsw` vacuum shape from ADR-027:
      pass 1 strips dead heap TIDs, pass 2 repairs neighbor arrays,
      pass 3 finalizes fully-dead nodes to `deleted = true`.
- [ ] Repair candidate selection for pass 2: replan under shared lock
      first (ADR-042 step 6), then fill under the page exclusive lock.
      Fill-only; no live-neighbor eviction under the write lock.
- [ ] 60-second `scripts/diskann_vacuum_concurrency_scratch.sh`
      analogue proving concurrent INSERT + scan + VACUUM safety.
- [ ] Post-vacuum recall smoke: delete 10% of rows, VACUUM, confirm
      Recall@10 ≥ 0.80 of pre-vacuum (matching FR-010-AC-2 for
      `tqhnsw`).

Review packet: 11009 (phase 5 landing).

### Phase 6 — Cost model and planner opt-in

- [ ] Cost model entries in `src/am/cost.rs` (or a new
      `src/am/diskann/cost.rs`) for Vamana access pattern. Model inputs:
      `R`, `L`, `ef_search`, reltuples, `index_pages`, entry-point
      depth. Unit tests match the shape of
      `src/am/cost.rs::tests`.
- [ ] Strategy translation (FR-023) and custom EXPLAIN (FR-024) opt-in
      for `tqdiskann`, following the PG18 scaffolding already in
      `src/am/explain.rs` and `src/am/stats.rs`.
- [ ] Planner gate lift: remove ADR-011-style override for
      `tqdiskann` specifically once phase 3 recall is signed off.
      The gate on `tqhnsw` is unaffected.

Review packet: 11010 (phase 6 landing).

## Parallelization

Serial spine: phase 1 → 2 → 3. Insert and vacuum (phases 4 and 5) can
run in parallel once phase 3 is green and their ADRs are accepted,
because they share no runtime state.

Critical path: phases 1–3 gate everything downstream.

## Owns

- Execution of ADR-034.
- Authoring ADR-041, ADR-042 (this planning packet).
- Cost model entries for `tqdiskann`.

## Dependencies

- **Task 15 (PqFastScan first-class)** — shipped on `main`. The AM
  will consume the existing `PqFastScanLayout` and FastScan scoring
  helpers without modification.
- **ADR-041** (accepted before phase 4 insert work starts).
- **ADR-042** (accepted before phase 5 vacuum work starts).

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
