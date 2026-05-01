# Task 30: SPIRE on a Single-Level IVF Foundation

Status: proposed — implements ADR-049 in two stages: first a debuggable
single-level IVF foundation with SPIRE-compatible assignment storage, then the
recursive SPIRE layer.

## Scope

Build SPIRE as an additive layer on top of a validated single-level IVF
foundation.

The first phase is not "build another unrelated IVF." It should reuse the
landed `ec_ivf` primitives wherever they are the right boundary: centroid
training, posting-list storage, quantizer profiles, candidate scoring, rerank,
admin snapshots, and local benchmark harnesses. The new SPIRE-specific
requirement is the assignment model from ADR-049: partition membership must be
stored as `(vec_id, partition_id)` rows so one vector can later belong to
multiple boundary partitions without a schema migration.

The second phase adds the SPIRE layer: recursive IVF-on-centroids, top-level
graph lookup, boundary replication, multi-level query routing, and
level-aware update propagation.

## Guiding Decisions

- ADR-049 is the governing design record.
- Build and validate a single-level foundation before recursion.
- Preserve one-to-many partition membership from the start.
- Keep SPIRE inside one Postgres extension with modular internal boundaries;
  do not introduce speculative pluggable index-strategy abstractions.
- Build SPIRE additions above/adjoining the IVF primitive, not as a replacement
  for working IVF code.

## Phase 0 — Reconcile Landed IVF With ADR-049

- [ ] **Inventory reusable IVF components.** Identify which `src/am/ec_ivf`
  modules can be consumed as-is by SPIRE and which need extraction into
  `src/am/common` or a SPIRE-owned module.
- [ ] **Assignment storage design note.** Decide the concrete Postgres storage
  shape for `(vec_id, partition_id)` rows: catalog table, auxiliary relation,
  index side table, or AM-owned sidecar. The invariant is one-to-many
  membership; the implementation must be reviewable and WAL-safe.
- [ ] **Compatibility note.** State whether current `ec_ivf` indexes keep their
  existing internal format while SPIRE gets a new format, or whether a future
  `ec_ivf` format bump will adopt the assignment table too.
- [ ] **Review packet.** Publish the Phase 0 design note before writing the
  persistence code.

## Phase 1 — Single-Level SPIRE-IVF Foundation

- [ ] **Module skeleton.** Add SPIRE-owned modules using ADR-041 boundaries,
  expected initial shape:
  - `src/am/spire/mod.rs`
  - `src/am/spire/build.rs`
  - `src/am/spire/assign.rs`
  - `src/am/spire/storage.rs`
  - `src/am/spire/scan.rs`
  - `src/am/spire/update.rs`
  - `src/am/spire/meta.rs`
- [ ] **SQL surface decision.** Decide whether the single-level foundation is
  exposed as a new `ec_spire` AM immediately or hidden behind internal tooling
  until recursion exists. Prefer exposing only a surface we are willing to
  support.
- [ ] **Assignment relation.** Implement `(vec_id, partition_id)` persistence
  with one row per vector in the initial single-level path.
- [ ] **Build path.** Reuse IVF centroid training, PQ/RaBitQ/PQ-FastScan
  encoding where applicable, and write posting-list membership through the
  assignment relation.
- [ ] **Scan path.** Route a query to top-`nprobe` partitions, score
  candidates, and rerank using the same correctness contract as local IVF.
- [ ] **Admin/diagnostics.** Expose centroid counts, assignment cardinality,
  posting-list row counts, quantizer profile, and build parameters.
- [ ] **Validation.** Add focused PG18 behavior tests for build, scan, empty
  index, insert-after-build, delete/vacuum cleanup, and assignment-table
  cardinality.
- [ ] **Review packet.** Land the single-level foundation with packet-local
  logs and a small recall/latency sanity row.

## Phase 2 — Update Mechanics

- [ ] **Cluster split-and-merge plan.** Translate the LIRE/SPFresh-style update
  mechanics into SPIRE's Postgres storage model.
- [ ] **Insert path.** Assign new vectors to one partition in the single-level
  path, update assignment rows, and make inserted rows visible to scans.
- [ ] **Delete/vacuum path.** Remove dead assignment rows and posting-list
  entries without breaking scan invariants.
- [ ] **Split trigger.** Define the partition growth/drift threshold that
  schedules a split.
- [ ] **Merge trigger.** Define the sparse/low-quality partition threshold that
  schedules a merge.
- [ ] **Concurrency validation.** Add a stress harness for insert/delete/scan
  overlap against the assignment relation and posting-list storage.

## Phase 3 — SPIRE Recursion

- [ ] **Hierarchy metadata.** Store levels, parent/child partition IDs,
  centroid dimensions, per-level `nprobe`, and build parameters.
- [ ] **Recursive build coordinator.** Run single-level IVF on input vectors,
  take resulting centroids as the next-level input, and repeat to target depth.
- [ ] **Centroid materialization.** Persist each level's centroids so rebuild,
  diagnostics, and query routing can inspect them.
- [ ] **Level-local scan primitive.** Given an input query and a parent
  partition, return child partitions to probe.
- [ ] **Review packet.** Demonstrate a small multi-level hierarchy where the
  same dataset can be queried as flat single-level IVF and recursive SPIRE.

## Phase 4 — Boundary Replication

- [ ] **Boundary predicate.** Define the threshold/rule for assigning a vector
  to multiple nearby partitions.
- [ ] **Assignment fanout.** Extend the assignment writer from one row per
  vector to multiple `(vec_id, partition_id)` rows.
- [ ] **Duplicate control.** Ensure scans deduplicate replicated vector IDs
  before final top-k.
- [ ] **Recall study.** Measure recall delta with boundary replication off/on
  at fixed storage overhead.
- [ ] **Storage accounting.** Report assignment-table and posting-list growth
  from replication.

## Phase 5 — Top-Level Graph

- [ ] **Graph choice.** Decide whether the top-level centroid graph uses HNSW,
  DiskANN, or a build-time-selectable option. Do not introduce a generic graph
  abstraction until there are two real consumers.
- [ ] **Build integration.** Build the top-level graph over top-level
  centroids after recursive centroid materialization.
- [ ] **Routing integration.** Replace flat top-level centroid scan with graph
  lookup, then descend through SPIRE levels.
- [ ] **Diagnostics.** Expose top-level graph size, degree, recall sanity rows,
  and routing fanout.

## Phase 6 — Product-Scale Measurement Gate

- [ ] **Local correctness matrix.** Keep local PG18 tests narrow and focused on
  correctness, WAL safety, and scan behavior.
- [ ] **Benchmark harness.** Extend `ecaz` to prepare/load/query SPIRE corpora
  and write packet-local artifacts.
- [ ] **Scale packet.** Run controlled AWS/RDS-class measurements before making
  product billion-scale claims.
- [ ] **Docs.** Update README/user docs only after a validated operator path
  exists.

## Dependencies

- ADR-049 — accepted staging and assignment-storage decision.
- Task 28 — landed IVF implementation and local benchmark substrate.
- Task 10 — benchmark result-capture discipline and packet-local artifacts.
- Task 19 — PG18 primary target and diagnostics surface.
- Task 26 — optional future scale hardware context; not a blocker for local
  correctness slices.

## Owns

- Future `ec_spire` access-method planning and implementation.
- SPIRE assignment storage, hierarchy metadata, and routing.
- SPIRE-specific `ecaz` operator workflows once an executable path exists.

## Out of Scope

- A generic pluggable ANN strategy framework.
- Product billion-scale claims without controlled hardware measurements.
- Rewriting landed `ec_ivf` unless Phase 0 explicitly justifies a format bump.
- GPU/offline training; that remains a separate future lane.

## Deliverables

- Phase 0 design packet for assignment storage and IVF reuse boundaries.
- Single-level SPIRE-IVF foundation with one-to-many-capable assignments.
- Recursive SPIRE build/query path.
- Boundary replication with deduped scans and recall/storage evidence.
- Top-level graph routing over top centroids.
- `ecaz` operator support and review-packet benchmark artifacts.

## Primary Validation

- Focused PG18 tests for each persistence/scan/update slice.
- `git diff --check` for docs/planning packets.
- Packet-local raw logs for every benchmark or measurement claim.

## Notes

- Keep the first implementation slice small. The highest-risk early decision is
  the assignment persistence shape, not recursive routing.
- If Phase 0 discovers that Postgres index AM mechanics make a literal
  user-visible assignment table inappropriate, write that down explicitly and
  preserve the same logical model in an auxiliary relation or AM-owned sidecar.
- Do not let the recursive SPIRE layer absorb bugs from the single-level
  primitive. Any unexpected scan/build behavior should first be reproducible in
  the single-level foundation.
