# Task 28: IVF Access Method

Status: in progress - Phase 3 populated build smoke tests underway.

Working branch: `task28-ivf`

## Scope

Build a first-class IVF access method for tqvector as a sibling to
`ec_hnsw`. The first target is a plain IVFFlat-style index: train
centroids, assign each vector to one posting list, scan the nearest
`nprobe` lists, and return ordered results through the normal PostgreSQL
index scan path.

This task intentionally does not start with multi-list replication or
balanced hierarchical routing. Those are larger mechanisms that should get
their own ADR only if a simple IVF baseline proves useful.

SQL name: `ec_ivf`.

## Why Now

ADR-017 originally chose HNSW as the default because tqvector had to serve
heterogeneous and evolving embedding shapes. IVF was left as a reversible
future choice for workloads where its tradeoffs are valuable:

- lower write amplification than HNSW live insert
- sequential posting-list reads that are friendlier to cold storage
- simpler append behavior during high-ingest periods
- a useful baseline against VectorChord-style IVF + RaBitQ designs

Starting IVF as a separate access method keeps `ec_hnsw` intact while
giving us a concrete way to measure these tradeoffs.

## Design Outline

- **Access method.** Add `src/am/ec_ivf/` following ADR-041's multi-AM
  structure: `build`, `scan`, `insert`, `vacuum`, `page`, `options`,
  `routine`, and `training`.
- **Training.** Deterministic k-means over a bounded build sample, with
  explicit behavior for tiny tables, empty tables, and `nlists > rows`.
- **Metric.** Inner-product search is the current SQL surface. The centroid
  scoring contract must be fixed up front: either spherical k-means over
  normalized vectors or an explicitly documented inner-product centroid
  scorer.
- **Storage.** Metadata stores dimension, quantizer shape, `nlists`,
  training seed/version, per-list counts, and list head/tail block refs.
  Posting-list pages store candidate codes plus heap TIDs using existing
  storage/WAL primitives where possible. IVF is a posting-list AM over the
  existing TurboQuant, PqFastScan, and RaBitQ quantizer profiles; PqFastScan
  is expected to be the dense-list hot path, but each profile gets its own
  recall and latency gates.
- **Scan.** `amrescan` prepares the query, scores centroids, selects
  `nprobe` lists, scans those lists, scores candidates, and emits the
  best results through the existing ordered-scan contract.
- **Insert.** `aminsert` assigns a new row to its nearest centroid and
  appends it to that posting list. Centroids do not move online.
- **Vacuum.** Vacuum removes dead heap TIDs from posting lists and updates
  list stats without retraining centroids.
- **Planner.** Cost model is shaped by `nprobe * avg_list_size`, not graph
  traversal. EXPLAIN should expose `nlists`, `nprobe`, selected list count,
  candidate count, and rerank mode.

## Subtasks

### Phase 0 - design freeze and ADR refresh

- [x] **ADR-017 refresh.** Amend or supersede ADR-017 so IVF is no longer
  only a deferred option. Preserve HNSW as the default unless measurements
  justify changing the default.
- [x] **Name and SQL contract.** Decide final AM name and operator-class
  naming. Prefer PostgreSQL AM-scoped reuse where valid; otherwise use
  explicit `*_ivf_ip_ops` names.
- [x] **Metric contract.** Decide and document centroid scoring for inner
  product. This must be settled before training code lands.
- [x] **Reloptions and GUCs.** Define `nlists`, `nprobe`, training sample
  size, seed, and rerank/source options. Pick defaults with a small-table
  path that cannot fail awkwardly.
- [x] **Acceptance gates.** Define baseline gates for recall, latency,
  storage, live-insert WAL volume, and vacuum behavior against `ec_hnsw`
  and exact scan.

Phase 0 decisions are recorded in ADR-048. Summary: `ec_ivf` is opt-in,
HNSW remains default, centroid training uses spherical k-means for the
inner-product router, `storage_format` selects `turboquant`, `pq_fastscan`,
`rabitq`, or `auto`, `ec_ivf.nprobe` overrides the reloption when set, and
full-probe `nprobe = nlists` must match exact indexed-row scoring for any
profile/rerank mode that claims exact final scoring.

### Phase 1 - AM scaffold

- [x] **Module layout.** Add `src/am/ec_ivf/{mod,routine,options,page,build,scan,insert,vacuum,training}.rs`.
- [x] **SQL bootstrap.** Register handler, access method, operator classes,
  reloptions, and pgrx exports.
- [x] **Empty index behavior.** `CREATE INDEX ... USING ec_ivf` on an empty
  table writes valid metadata and scan callbacks return no rows.
- [x] **Skeleton callbacks.** Wire all AM callbacks with explicit
  not-implemented errors for unsupported populated paths, then replace
  each callback in later phases.
- [x] **Review packet.** Publish the scaffold contract before build logic
  starts.

Phase 1 scaffold checkpoint: `src/am/ec_ivf/` now compiles as a registered
AM with `ec_ivf.nprobe`, IVF reloptions, SQL bootstrap entries, and explicit
not-implemented callbacks.

Phase 1 empty-index checkpoint: empty `ec_ivf` builds now write a versioned
metadata page, preserve IVF reloptions in metadata, and return no tuples from
the heap-backed AM scan path after rescan. Populated builds still fail loudly
until Phase 2/3 storage and training land.

### Phase 2 - page and metadata layout

- [x] **Metadata page.** Encode/decode IVF metadata: format version,
  dimensions, quantizer format, `nlists`, `nprobe` default, centroid table
  location, and posting-list directory.
- [x] **Centroid storage.** Choose inline metadata vs dedicated centroid
  pages. Keep decoding independent from scan state.
- [x] **Posting-list tuple.** Define candidate tuple format for `tqvector`
  and `ecvector`, including duplicate heap TID handling.
- [ ] **List directory.** Track per-list head/tail pages and live tuple
  counts with WAL-safe updates.
- [x] **Layout tests.** Add roundtrip, length mismatch, small-table, and
  page-fit coverage.

Phase 2 layout-codec checkpoint: metadata now carries dimensions, training
version, centroid/directory heads, live/dead counts, and drift counters.
Centroids use dedicated data-page tuples, list directory entries have a fixed
codec for head/tail/count state, and posting tuples preserve duplicate heap
TIDs with a profile-neutral payload. WAL-safe directory updates are still
tracked separately because build/insert/vacuum do not consume the directory
yet.

### Phase 3 - training and build

- [x] **Training sample.** Heap-scan sample collection with deterministic
  seed, type validation, NULL rejection, and dimension checks.
- [x] **K-means trainer.** Implement bounded-iteration k-means with stable
  empty-cluster handling and tests for deterministic output.
- [x] **Bulk assignment.** Assign every row to one nearest centroid and
  append to the matching posting list.
- [ ] **Build stats.** Record per-list counts, empty-list count, centroid
  drift inputs, and source/quantizer metadata.
- [ ] **Build smoke tests.** Cover empty, singleton, tiny multi-row,
  duplicate-heavy, and multi-page list builds.

Phase 3 pure-training checkpoint: `src/am/ec_ivf/training.rs` now has
deterministic sample-index selection, auto-`nlists` resolution, finite
non-zero vector normalization, bounded spherical k-means, stable empty-cluster
fallback, and centroid assignment by normalized inner product. Heap scan sample
collection and populated index writes remain future Phase 3 slices.

Phase 3 heap-scan sample checkpoint: populated builds now scan heap tuples
through the IVF callback, reject NULLs and inconsistent dimensions before
training, decode `ecvector` source vectors directly, derive approximate
training vectors for `tqvector`, select deterministic training samples, and
train centroids before the still-explicit populated-write gate. Posting-list
writes and metadata updates remain future Phase 3 slices.

Phase 3 bulk-assignment checkpoint: populated builds now assign each collected
row to its nearest trained centroid, stage centroid/posting/directory tuples in
an in-memory data-page chain, set metadata dimensions/list heads/live totals,
and count empty lists. At this checkpoint, on-disk populated writes were still
gated until the directory update and WAL-safe physical write path landed.

Phase 3 populated-write checkpoint: non-empty builds now flush staged centroid,
posting, and directory data pages with GenericXLog full-image WAL, then rewrite
metadata with trained dimensions, resolved `nlists`, head pointers, training
version, and total live tuple count. Scans over populated IVF indexes still
return no rows until the Phase 4 routing and posting-list scan path lands.

### Phase 4 - scan path

- [ ] **Query prep.** `amrescan` validates the ORDER BY query, caches the
  prepared scorer, scores all centroids, and stores the selected `nprobe`
  list IDs.
- [ ] **Posting-list scan.** Read selected lists sequentially, score
  candidates, deduplicate duplicate heap TIDs, and maintain a top-k heap.
- [ ] **Result emission.** Reuse the ordered tuple production lifecycle:
  forward-only scan, order-by score output, exhaustion clearing, and rescan
  reset behavior.
- [ ] **Rerank mode.** Decide whether v1 always reranks from heap/source
  data or starts compressed-only with a reloption-controlled exact tail.
- [ ] **Recall tests.** Add small deterministic oracle tests and a real
  corpus recall smoke that compares exact scan, `ec_hnsw`, and `ec_ivf`.

### Phase 5 - live insert

- [ ] **Centroid assignment.** `aminsert` scores centroids and appends the
  row to the nearest posting list under a narrow list-tail lock.
- [ ] **Shape validation.** Reject mismatched dimension, quantizer bits,
  seed, and unsupported source layouts with clear errors.
- [ ] **List stats.** Update per-list live counts and insert-since-build
  drift counters.
- [ ] **Concurrency coverage.** Cover concurrent inserts into different
  lists, same-list tail append, empty-index first insert, and duplicate
  heap TID rejection.

### Phase 6 - vacuum and drift handling

- [ ] **Dead tuple cleanup.** Remove dead heap TIDs from posting lists and
  mark empty candidate tuples without changing centroid assignments.
- [ ] **Directory repair.** Keep list counts, head/tail refs, and empty-list
  stats consistent after cleanup.
- [ ] **Drift snapshot.** Expose centroid staleness indicators: inserted
  since build, changed row fraction, list imbalance, and recommended
  REINDEX threshold.
- [ ] **Vacuum safety tests.** Exercise repeated vacuum, insert plus vacuum,
  scan plus vacuum, and post-vacuum recall sanity.

### Phase 7 - planner, EXPLAIN, and admin surfaces

- [ ] **Cost model.** Estimate startup and total costs from centroid count,
  `nprobe`, average list size, scoring mode, and rerank mode.
- [ ] **EXPLAIN counters.** Report centroid scores, selected lists, posting
  pages read, candidates scored, rerank rows, and filtered duplicates.
- [ ] **Admin snapshot.** Add an IVF snapshot function for metadata,
  distribution, drift, and planner inputs.
- [ ] **PG18 hooks.** Wire strategy translation, tree height, ReadStream,
  and shared stats only after the PG18 `ec_hnsw` surfaces are stable enough
  to reuse cleanly.

### Phase 8 - validation and measurement

- [ ] **Unit gate.** `cargo test` for trainer, codec, list directory, and
  scan heap behavior.
- [ ] **Extension gate.** `cargo pgrx test pg17` for SQL callback behavior.
- [ ] **Lint gate.** `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`.
- [ ] **Recall gate.** Real `10K` and `50K` recall@10 sweeps over
  `nlists` and `nprobe`, compared with exact and `ec_hnsw`.
- [ ] **Latency gate.** Warm and cold p50/p95/p99 at equal recall.
- [ ] **Storage/WAL gate.** `pg_relation_size`, build WAL, and live-insert
  WAL compared against `ec_hnsw`.
- [ ] **Review packets.** Store raw logs under packet-local artifacts for
  any measurement claim.

## Owns

- New IVF access method implementation under `src/am/ec_ivf/`.
- IVF SQL bootstrap and operator classes.
- IVF metadata, centroid, and posting-list page formats.
- IVF planner/admin/EXPLAIN surfaces once the runtime is credible.
- ADR update that turns IVF from a deferred option into an active lane.

## Dependencies

- Existing `tqvector` and `ecvector` datum/scoring surfaces.
- Existing quantizer traits and PqFastScan/TurboQuant scoring kernels.
- Shared storage/WAL primitives from the AM module split.
- Planner/EXPLAIN/ReadStream common surfaces where they are already
  reusable across access methods.

## Defers

- Multi-list replication and balanced hierarchical k-means.
- Online centroid retraining or tuple reassignment outside REINDEX.
- Multi-metric support beyond the current inner-product operator surface.
- Making IVF the default index type.
- Cross-AM shared posting-list abstractions before the first IVF baseline
  proves the shape.

## Initial Review Packet

Create the first review packet after Phase 0 under the 30000 range with:

- final SQL naming decision
- ADR-017 refresh or replacement
- metric/training contract
- reloption defaults
- acceptance gates for the first runnable baseline
