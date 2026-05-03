# Task 30: SPIRE on a Partition-Object IVF Foundation

Status: in progress — Phase 0 storage design checkpoint recorded in
`plan/design/spire-phase0-partition-object-storage.md`; Phase 1 now has
SPIRE-owned partition-object codecs, placement/epoch metadata, in-memory
single-level route maps, root routing objects, and per-centroid leaf-object
draft publication. Segmented, column-major V2 leaf-object codecs and an
in-memory local-store V2 write/read path now back base leaf build output, and
quantized routed scans batch-score V2 column segments directly while retaining
V1 row-object helpers for compatibility tests. Scan helpers can now route to
top-`nprobe` leaves, collect ranked candidates through injected and concrete
quantized scorer paths, dedupe by `vec_id`, and consume the resolved
single-level scan plan through the helper-level quantized scoring and
exact-rerank path. Routed top-`nprobe` selection and final candidate limiting
now use bounded heaps with deterministic tie-breaks, and the scan plan carries
an explicit dedupe mode so the Phase 1 primary-only path skips the `vec_id` map
until boundary replicas or remote merge require it. Build/update publication
helpers and delta-from-snapshot logic now consume the validated snapshot wrapper
rather than rebuilding published snapshots internally. Routing partition
objects now store child PIDs, centroid ordinals, and centroid values in flat
arrays instead of one owned centroid vector per child. A cursor-to-scan-output
bridge now maps ranked candidates to heap TID plus ORDER BY score output for
future `amgettuple` wiring, and scan callbacks now have allocated opaque state
plus cursor-drain emission once `amrescan` can populate candidates. Root routing
metadata can now provide the single-level leaf count that scan option resolution
needs, and scan opaque state now carries a validated query object for future
`ScanKey` parsing. The relation-backed root/control page can now persist and
read the empty SPIRE state, empty `ambuild`/`ambuildempty` initialize that page,
and live `amrescan` can return an empty cursor for an empty active epoch while
populated relation-backed snapshot loading remains blocked. Relation-backed
object tuple append/read helpers can now store encoded SPIRE object bytes in
data blocks after the root/control page and round-trip an encoded routing
object from an `ec_spire` index relation. Assignment payload scoring now reuses
the existing TurboQuant and
RaBitQ quantizers behind a SPIRE-owned row scorer, while PQ-FastScan remains
deferred until grouped-PQ model metadata is persisted. AM option/GUC plumbing
exists for single-level build and scan parameters. A pre-persistence
architecture gate from the first foundation review is now recorded in
`plan/design/spire-foundation-architecture-feedback-response.md`; live
PostgreSQL relation-backed build/scan persistence remains intentionally
unwired until that gate is cleared. Task 30 implements
ADR-049 in stages: first a debuggable single-level IVF foundation with
SPIRE-compatible partition-object storage, then recursive SPIRE routing, local
multi-NVMe placement, and later multi-machine placement.

## Scope

Build SPIRE as an additive layer on top of a validated single-level IVF
foundation.

The first phase is not "build another unrelated IVF." It should reuse the
landed `ec_ivf` primitives wherever they are the right boundary: centroid
training, quantizer profiles, candidate scoring, rerank, admin snapshots, and
local benchmark harnesses. The new SPIRE-specific requirement is the storage
model from ADR-049: vector membership must be stored as logical `(vec_id, pid)`
rows inside SPIRE partition objects so one vector can later belong to multiple
boundary partitions without a schema migration.

The second phase adds the SPIRE layer: recursive IVF-on-centroids, top-level
graph lookup, boundary replication, multi-level query routing, and level-aware
update propagation. Later phases add placement: first local partition stores
striped across physical NVMe devices, then multi-machine PID routing.

## Guiding Decisions

- ADR-049 is the governing design record.
- Build and validate a single-level foundation before recursion.
- Use "SPIRE partition" only for index-internal clusters/PIDs; do not confuse it
  with PostgreSQL declarative table partitions.
- Preserve one-to-many vector-to-PID membership from the start.
- Treat PID-addressed partition objects as the storage unit; do not design
  around one monolithic index relation as the only durable shape.
- Preserve a placement map that starts as `pid -> local_store_id` and can later
  extend to `pid -> node_id -> local_store_id`.
- Version partition objects with a published SPIRE epoch so root metadata,
  hierarchy metadata, placement metadata, and partition objects are compatible
  during a query.
- Keep SPIRE inside one Postgres extension with modular internal boundaries;
  do not introduce speculative pluggable index-strategy abstractions.
- Build SPIRE additions above/adjoining the IVF primitive, not as a replacement
  for working IVF code.

## Phase 0 — Reconcile Landed IVF With ADR-049

Decision record:
`plan/design/spire-phase0-partition-object-storage.md`.

- [x] **Inventory reusable IVF components.** Identify which `src/am/ec_ivf`
  modules can be consumed as-is by SPIRE and which need extraction into
  `src/am/common` or a SPIRE-owned module.
- [x] **Partition-object storage design note.** Decide the concrete Postgres
  storage shape for PID-addressed partition objects and their logical
  `(vec_id, pid)` assignment rows: one control/root relation plus bounded
  partition-store relations, a single-store prototype format, or another
  AM-owned sidecar. The invariant is one-to-many membership; the implementation
  must be reviewable and WAL-safe.
- [x] **PID identity note.** Define `pid`, `vec_id`, local heap TID, parent PID,
  child PID, boundary-replica flags, and how local `vec_id` maps to future
  global vector IDs. The note must bound encoded `vec_id` width, state
  uniqueness scope, and reserve or justify the local/global discriminator.
- [x] **Heap locator update note.** Decide how stored local heap TIDs interact
  with PostgreSQL UPDATE/HOT movement and vacuum: repair in place, tombstone and
  reinsert through an epoch-safe path, resolve by `vec_id`, or suppress stale
  candidates with diagnostics.
- [x] **Placement note.** Define the initial `pid -> local_store_id -> object`
  placement map and the extension point for `pid -> node_id -> local_store_id`.
  State explicitly that SPIRE does not use PostgreSQL table partitions for
  vector partition selection.
- [x] **Epoch/version note.** Decide whether Phase 1 stores immutable
  `(pid, epoch)` objects directly or stores per-partition versions referenced by
  an epoch manifest. State old-epoch retention and cleanup expectations.
- [x] **Insert/delete lifecycle note.** Document whether the first local path
  uses live deltas, mutable partition objects, or replacement epochs, and map
  that choice to strict-mode visibility/failure behavior.
- [x] **Compatibility note.** State whether current `ec_ivf` indexes keep their
  existing internal format while SPIRE gets a partition-object format, or
  whether a future `ec_ivf` format bump will adopt partition objects too.
- [x] **Phase 1 surface note.** Decide whether Phase 1 exposes `ec_spire` and
  document the planned opclass names.
- [x] **Review packet.** Publish the Phase 0 design note before writing the
  persistence code. Packet target: `review/30162-spire-phase0-partition-object-storage/`.

## Phase 1 — Single-Level SPIRE-IVF Foundation

- [x] **Module skeleton.** Add SPIRE-owned modules using ADR-041 boundaries,
  expected initial shape:
  - `src/am/ec_spire/mod.rs`
  - `src/am/ec_spire/build.rs`
  - `src/am/ec_spire/assign.rs`
  - `src/am/ec_spire/storage.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
  - `src/am/ec_spire/meta.rs`
  Initial callbacks are explicit unsupported stubs until persistence lands.
- [x] **SQL surface decision.** Decide whether the single-level foundation is
  exposed as a new `ec_spire` AM immediately or hidden behind internal tooling
  until recursion exists. Phase 0 chooses an opt-in `ec_spire` AM for Phase 1.
- [x] **Opclass documentation.** If `ec_spire` is exposed in Phase 1, register
  `ecvector_spire_ip_ops` and `tqvector_spire_ip_ops` in `spec/spec.md`;
  otherwise keep them explicitly marked as deferred.
- [x] **Architecture feedback response.** Process the first holistic foundation
  review before live persistence. The response checkpoint is
  `plan/design/spire-foundation-architecture-feedback-response.md`; it keeps
  relation-backed persistence blocked until the pre-persistence hardening
  items below are implemented or superseded by an accepted design update.
- [x] **Segmented column-major leaf objects.** Replace the in-memory
  row-contiguous base-leaf format with `LeafPartitionObjectV2`: one metadata
  tuple plus page-sized row segments containing column-major flags,
  fixed-stride `vec_id`s, heap TIDs, gammas, and payload bytes. Keep small
  deltas row-encoded until compaction rewrites a V2 base object. V2
  metadata/segment codecs plus an in-memory local-store segmented write/read
  path have landed; single-level and partitioned build drafts now write V2 base
  leaves, object-header dispatch understands V2 metadata placements, and scan
  helpers can read either V1 compatibility leaves or V2 base leaves.
- [x] **Borrowed leaf reads and batch scoring.** Add borrowed V2 column views,
  borrowed row references for row-encoded deltas, one shared assignment
  visibility predicate, and batch assignment scorer entry points before
  persisted scan callbacks consume leaf objects. Borrowed V1 row references and
  shared visibility predicates have landed, and the prepared assignment scorer
  now has a shape-checked batch scoring entry point for TurboQuant and RaBitQ.
  V2 leaf segments now expose borrowed column views plus row accessors over
  flags, fixed-stride vec_ids, heap TIDs, gammas, and payload chunks. Quantized
  routed candidate scans now batch-score V2 payload blocks directly from those
  column views, while retaining V1 row-scoring fallback for compatibility tests.
- [x] **Validated snapshot lookup cache.** Introduce a validated epoch snapshot
  wrapper with PID-indexed manifest/placement lookups. Internal scan, update,
  and diagnostics helpers should consume the wrapper instead of repeatedly
  rebuilding `SpirePublishedEpochSnapshot`. Scan, diagnostics, build
  publication, and delta-update publication helpers now use
  `SpireValidatedEpochSnapshot`; delta-from-snapshot helper logic uses cached
  PID lookup for base placement and assignment-ID collection.
- [x] **Flat routing object layout.** Replace per-child `Vec<f32>` routing
  entries with flat `child_pids`, `centroid_ordinals`, and centroid block arrays
  before root/internal routing objects become relation-backed. Constructors
  still accept `SpireRoutingChildEntry`, but `SpireRoutingPartitionObject`
  stores parallel `child_pids`, `centroid_ordinals`, and one flat centroid block
  with borrowed child views for scan iteration.
- [x] **Bounded routing and candidate heaps.** Replace sort/truncate
  top-`nprobe` and candidate top-k selection with bounded heaps and a documented
  deterministic tie-break contract. The routed scan helper now keeps a bounded
  route heap ordered by higher inner product, lower centroid index, then lower
  child PID. Candidate ranking dedupes by `vec_id`, then keeps a bounded heap
  ordered by lower ORDER BY score, newer serving epoch, primary assignment
  before boundary replica within an epoch, heap TID, PID, row index, and
  `vec_id` bytes.
- [x] **Explicit dedupe mode.** Carry a scan/snapshot dedupe mode so Phase 1's
  primary-only path skips the `vec_id` HashMap, while boundary replicas and
  future remote merge re-enable `vec_id` dedupe explicitly. The single-level
  scan plan now defaults to `NoReplicaDedupeDisabled`; lower-level helper tests
  can opt into `VecIdDedupeEnabled` for boundary-replica and merge semantics.
- [x] **Publish coordinator.** Add a typed publication state machine for object
  writes, placement writes, manifest writes, validation, active-epoch advance,
  and failed-publish cleanup before live relation-backed writes are enabled.
  Build and delta publish-bundle helpers now run through typed
  `WritingObjects -> WritingPlacements -> WritingManifest -> Validating ->
  PublishingActiveEpoch` states. Failed transitions return a staged
  `SpirePublishFailed` and cannot construct root/control bytes that would
  advance the active epoch.
- [x] **Architecture follow-up cleanups.** Add object epoch back-references,
  a `SpireObjectReader` trait shared by in-memory and buffer-cache readers,
  byte diagnostics by object kind, allocator near-exhaustion diagnostics,
  explicit placement constructors, and a single source for primary/replica
  visibility semantics. Core partition-object codecs now use explicit header
  and assignment wire-shape validation helpers instead of encode-as-validation;
  object validators check header identity directly, and encoders reuse the
  post-validation encode path. Placement entries now have explicit local
  single-store constructors for available, stale, unavailable, and skipped
  states. PID and local vec_id allocators now expose non-mutating
  near-exhaustion diagnostics, with a root/control helper that reports both
  cursors from persisted allocator state. Snapshot diagnostics now split
  available object bytes into routing, leaf, and delta buckets. A
  `SpireObjectReader` trait now defines the shared object read contract, and
  snapshot diagnostics, scan helpers, and read-only delta-update collection
  consume that trait instead of the concrete in-memory store. Partition-object
  headers now carry a `published_epoch_backref` stamped
  by local-store insertion and verified as not newer than the placement epoch
  on reads; V2 leaf metadata and segments inherit the same header
  back-reference. Assignment payload encoding no longer returns a discarded
  dimension value; the helper validates source shape and returns only scoring
  metadata plus payload bytes.
- [x] **Leaf assignment rows.** Implement logical `(vec_id, pid)` assignment
  rows inside leaf partition objects with one row per vector in the initial
  single-level path. Foundation codecs and draft builders now store validated
  row identity, heap locators, payload/scoring metadata, and role flags inside
  PID-addressed leaf objects; live AM callback wiring remains covered by the
  build and scan path tasks below.
- [ ] **Single-store placement.** Persist a PID placement directory even if the
  first executable path maps every PID to one local store. Foundation metadata
  now includes placement-entry and placement-directory codecs, local
  single-store object placements, exact object-manifest/placement PID-set
  validation, and fail-closed delta publication from non-available base
  placements. Partitioned build drafts now publish root and leaf PID placements
  into the local object store. Relation-backed object tuple append/read helpers
  now write and read encoded object bytes from index data blocks after the
  root/control page; placement-directory persistence and populated build
  integration remain open.
- [ ] **Build path.** Reuse IVF centroid training, PQ/RaBitQ/PQ-FastScan
  encoding where applicable, and write posting-list membership through leaf
  partition objects. The spherical k-means training helper is now factored into
  `src/am/common/training.rs` with `ec_ivf` compatibility wrappers so SPIRE can
  consume the centroid training boundary without importing private `ec_ivf`
  modules. The in-memory build draft now validates a single-level centroid plan,
  allocates one root PID plus one leaf PID per centroid, writes a root routing
  object, writes per-centroid leaf partition objects including empty leaves, and
  publishes a strict object/placement manifest snapshot before committing
  allocator cursors. The assignment payload seam now encodes TurboQuant and
  RaBitQ row payloads through the existing quantizer implementations and keeps
  PQ-FastScan explicit but blocked on persisted grouped-PQ model metadata. A
  source-vector helper now builds quantized leaf assignment inputs from heap
  locators plus source vectors for future AM build/insert wiring. Live
  relation-backed empty build now initializes the persisted root/control page
  and rejects populated heap builds with an explicit not-implemented error;
  populated relation-backed build writes remain blocked on object persistence.
- [ ] **Scan path.** Route a query to top-`nprobe` partitions, score
  candidates, and rerank using the same correctness contract as local IVF. The
  foundation now has helper-level root routing object discovery, strict/degraded
  placement handling for routed leaves, single-route query-to-leaf collection,
  top-`nprobe` leaf selection over root child centroids, visible-primary
  candidate scoring through an injected scorer, explicit dedupe mode, bounded
  candidate limiting, deterministic score ordering, and an injected exact-rerank
  seam. Stored assignment payload
  scoring now has TurboQuant and RaBitQ prepared-scorer support, and the routed
  scan helper can prepare that scorer directly for real encoded assignment rows.
  The helper-level scan path can now consume a resolved single-level scan plan
  and compose route, quantized score, dedupe mode, candidate limiting, and
  exact-rerank callback application. It also has a cursor-to-output bridge for
  heap TID plus ORDER BY score emission, plus scan opaque lifecycle allocation
  and cursor-drain `amgettuple` behavior for future populated scans. `amrescan`
  can now derive the scan-plan leaf count from root routing metadata once a
  published snapshot is loaded, and scan state now stores a validated
  non-empty, finite, non-zero query object. Live `amrescan` now parses and
  validates the ORDER BY query, reads the relation-backed root/control page, and
  returns no rows for an empty active epoch. Populated live snapshot loading,
  heap rerank callback implementation, and PQ-FastScan scorer binding remain
  open.
- [x] **Scan/build option plumbing.** Register SPIRE-owned reloptions and
  session GUCs for the single-level foundation before AM callbacks consume
  them. The AM routine now exposes `amoptions` for `nlists`, `nprobe`,
  `rerank_width`, `training_sample_rows`, `seed`, `pq_group_size`,
  `storage_format`, and `quantizer`; session overrides exist for
  `ec_spire.nprobe` and `ec_spire.rerank_width`. These settings now resolve to
  a helper-level single-level scan plan carrying effective `nprobe`, assignment
  payload format, rerank width, and pre-rerank candidate limit, and the scan
  helper now consumes that plan before live AM callback wiring. Option
  consumption by live build/scan callbacks remains part of those open tasks.
- [ ] **Admin/diagnostics.** Expose centroid counts, assignment cardinality,
  leaf partition object counts, posting-list row counts, placement map state,
  quantizer profile, and build parameters. The foundation now has an internal
  snapshot diagnostics helper that reports epoch/consistency mode, object and
  placement counts, local-store count, placement-state counts, object-kind
  counts, routing-child count, assignment counts, and available object bytes
  for available local placements. SQL exposure, quantizer/build-parameter
  reporting, and relation-backed admin reads remain open.
- [ ] **Validation.** Add focused PG18 behavior tests for build, scan, empty
  index, insert-after-build, delete/vacuum cleanup, and leaf-assignment
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
  overlap against leaf assignment rows and partition-object storage.

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

## Phase 4 — Local Multi-NVMe Placement

- [ ] **Partition-store relation layout.** Define bounded store relations and
  how each maps to a PostgreSQL tablespace expected to live on a physical NVMe
  device.
- [ ] **Hash placement.** Place leaf and internal partition objects by
  `hash(pid) % local_store_count`.
- [ ] **Parallel local fetch.** Fetch selected PIDs grouped by local store and
  keep scoring close to the partition object bytes.
- [ ] **Placement diagnostics.** Expose per-store object count, bytes,
  candidate rows, and scanned PID counts.
- [ ] **Local placement benchmark.** Measure one-store vs multi-store behavior
  on a machine with multiple physical NVMe devices before making any product
  claim.

## Phase 5 — Boundary Replication

- [ ] **Boundary predicate.** Define the threshold/rule for assigning a vector
  to multiple nearby partitions.
- [ ] **Assignment fanout.** Extend the assignment writer from one row per
  vector to multiple `(vec_id, pid)` rows.
- [ ] **Duplicate control.** Ensure scans deduplicate replicated vector IDs
  before final top-k.
- [ ] **Recall study.** Measure recall delta with boundary replication off/on
  at fixed storage overhead.
- [ ] **Storage accounting.** Report leaf-assignment and posting-list growth
  from replication.

## Phase 6 — Top-Level Graph

- [ ] **Graph choice.** Decide whether the top-level centroid graph uses HNSW,
  DiskANN, or a build-time-selectable option. Do not introduce a generic graph
  abstraction until there are two real consumers.
- [ ] **Build integration.** Build the top-level graph over top-level
  centroids after recursive centroid materialization.
- [ ] **Routing integration.** Replace flat top-level centroid scan with graph
  lookup, then descend through SPIRE levels.
- [ ] **Diagnostics.** Expose top-level graph size, degree, recall sanity rows,
  and routing fanout.

## Phase 7 — Multi-Machine Placement

- [ ] **Remote node model.** Define node identity, placement-map membership,
  remote health, and stale-node behavior.
- [ ] **Remote search API.** Add a SPIRE remote search SQL function on storage
  nodes that accepts query vector, selected PIDs, requested epoch, and top-k
  budget, then returns compact candidate rows.
- [ ] **Coordinator transport.** Use libpq pipeline mode first for
  coordinator-to-node fanout; do not invent a custom network protocol until the
  SQL/protocol shape fails measurement.
- [ ] **Distributed epoch manifest.** Publish root/hierarchy/placement metadata
  only after all nodes can serve the requested epoch or report an explicit
  stale-node state.
- [ ] **Graceful degradation policy.** Define strict fail-closed and degraded
  recall modes for unavailable or stale nodes/stores, with degraded mode
  reporting skipped placements explicitly.
- [ ] **Merge semantics.** Merge remote candidates by stable `vec_id`, dedupe
  boundary replicas, and define how local heap row resolution works after
  remote candidate selection.
- [ ] **Replica deferral.** Record replicated partition objects as future work
  for read throughput and availability; v1 assumes one primary placement per
  PID.

## Phase 8 — Product-Scale Measurement Gate

- [ ] **Local correctness matrix.** Keep local PG18 tests narrow and focused on
  correctness, WAL safety, and scan behavior.
- [ ] **Benchmark harness.** Extend `ecaz` to prepare/load/query SPIRE corpora
  and write packet-local artifacts.
- [ ] **Scale packet.** Run controlled AWS/RDS-class measurements before making
  product billion-scale claims.
- [ ] **Docs.** Update README/user docs only after a validated operator path
  exists.

## Dependencies

- ADR-049 — accepted staging and partition-object storage decision.
- Task 28 — landed IVF implementation and local benchmark substrate.
- Task 10 — benchmark result-capture discipline and packet-local artifacts.
- Task 19 — PG18 primary target and diagnostics surface.
- Task 26 — optional future scale hardware context; not a blocker for local
  correctness slices.

## Owns

- Future `ec_spire` access-method planning and implementation.
- SPIRE partition-object storage, hierarchy metadata, placement, and routing.
- SPIRE-specific `ecaz` operator workflows once an executable path exists.

## Out of Scope

- A generic pluggable ANN strategy framework.
- Product billion-scale claims without controlled hardware measurements.
- Rewriting landed `ec_ivf` unless Phase 0 explicitly justifies a format bump.
- GPU/offline training; that remains a separate future lane.

## Deliverables

- Phase 0 design packet for partition-object storage, placement, epoch, and IVF
  reuse boundaries.
- Single-level SPIRE-IVF foundation with one-to-many-capable assignments.
- Recursive SPIRE build/query path.
- Local multi-NVMe partition-store placement.
- Boundary replication with deduped scans and recall/storage evidence.
- Top-level graph routing over top centroids.
- Multi-machine coordinator and remote partition-store prototype.
- `ecaz` operator support and review-packet benchmark artifacts.

## Primary Validation

- Focused PG18 tests for each persistence/scan/update slice.
- `git diff --check` for docs/planning packets.
- Packet-local raw logs for every benchmark or measurement claim.

## Notes

- Keep the first implementation slice small. The highest-risk early decision is
  the partition-object persistence and placement shape, not recursive routing.
- If Phase 0 discovers that Postgres index AM mechanics make direct
  user-visible assignment rows inappropriate, write that down explicitly and
  expose diagnostics through read-only functions over partition-object storage.
- Phase 0 chose index-local `vec_id` allocation rather than heap-TID-derived
  `vec_id`s. Heap TIDs remain local row locators only.
- Phase 0 chose per-partition object versions referenced by epoch manifests,
  with immutable published objects and epoch-published delta/replacement objects
  for inserts, vacuum cleanup, split, merge, and rebalance.
- Do not let the recursive SPIRE layer absorb bugs from the single-level
  primitive. Any unexpected scan/build behavior should first be reproducible in
  the single-level foundation.
- Do not use PostgreSQL declarative table partitions for SPIRE vector
  partitions. If the implementation uses multiple relations for local NVMe
  placement, they are bounded partition stores, not one relation per PID and not
  planner-pruned table partitions.
