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
and live `amrescan` can return an empty cursor for an empty active epoch or
load an active relation-backed epoch into a scan cursor. Relation-backed object
tuple append/read helpers can now store encoded SPIRE object bytes in
data blocks after the root/control page and round-trip an encoded routing
object from an `ec_spire` index relation through a relation object store that
emits local single-store placement entries. The same relation object store can
now write and read segmented V2 leaf metadata plus segment chains for local
single-store placements, and implements the shared `SpireObjectReader`
interface for future snapshot scan loading. Encoded manifest bundles can now be
persisted as relation tuples and used to publish a new root/control active
epoch. Populated relation-backed builds now write routing objects, V2 leaf
objects, durable placement-entry rows, and manifest bundles before advancing
root/control to the active epoch. The publish coordinator now requires write
evidence for object and placement stage transitions, and relation object pages
guard root/control initialization, special-area reads, and FSM reuse.
Assignment payload scoring now reuses the existing TurboQuant and
RaBitQ quantizers behind a SPIRE-owned row scorer, while PQ-FastScan remains
deferred until grouped-PQ model metadata is persisted. AM option/GUC plumbing
exists for single-level build and scan parameters. A pre-persistence
architecture gate from the first foundation review is now recorded in
`plan/design/spire-foundation-architecture-feedback-response.md`; live
PostgreSQL relation-backed build, initial quantized scan with heap rerank, and
active snapshot cardinality diagnostics now have a strict single-store path,
post-build inserts can publish strict delta epochs, and the first insert into
an empty active epoch can bootstrap a strict one-leaf root/leaf epoch. Vacuum
can now publish strict row-encoded delete-delta epochs for
callback-dead visible assignments, and live scans suppress base and
delta-insert candidates whose `vec_id`s are covered by a routed delete delta;
vacuum cleanup can now compact active delta objects into replacement V2 base
leaves while removing delta placements from the active directory. The first
SQL diagnostics surface now exposes active epoch/object/placement cardinality
through `ec_spire_index_active_snapshot_diagnostics`, and relation build/scan
options plus effective scan option resolution through
`ec_spire_index_options_snapshot`, including whether the resolved assignment
payload format is currently scannable and the explicit PQ-FastScan grouped-PQ
metadata deferral when applicable; the health snapshot now reports
conservative status/recommendation rows, including active delta compaction
recommendations, and the first placement snapshot exposes per-local-store
active placement/object/byte counts; a query-specific scan placement snapshot
now exposes per-store routed leaf PID, delta PID, and candidate-row counts; a
root routing snapshot now exposes active centroid-to-child PID rows and has
unit coverage for malformed active manifests with zero or multiple root
objects; relation storage diagnostics now quantify active-referenced and
cleanup-candidate object tuples after insert-delta and vacuum-compaction
replacement epochs while physical reclamation remains deferred;
scan sanity diagnostics now expose resolved scan preconditions for exact leaf
coverage and full-frontier rerank; replacement-epoch publishes now write a
retired manifest copy for the previous active epoch, and epoch diagnostics
now cover post-insert and post-vacuum-compaction retired manifest rows,
partial-publish retired/bundle residue, and cleanup eligibility blockers; leaf
diagnostics now expose per-leaf base/delta/effective assignment counts plus
read-only split/merge threshold recommendations for follow-up scheduling;
insert-debt diagnostics now expose per-leaf delta fanout and batching
recommendations while actual insert batching remains open. Packet
`review/30530-spire-phase1-recall-latency-gate/` now records local real 10k
SPIRE recall/latency evidence for the single-store `nlists=32`,
`rerank_width=25` foundation: recall@10 is `0.9985` at `nprobe=8` and
`1.0000` from `nprobe=16` through `32`, with latency p50/p95 `62.1/70.2 ms`
at `nprobe=8`. Phase 4 relation-backed live insert, delete-delta publication,
delta compaction, and live-assignment counting now route through the active
local store set instead of falling back to the root/control relation store;
packet `review/30531-spire-mutation-local-store-routing/` covers that
checkpoint. Packet `review/30533-spire-local-placement-benchmark/` now records
the first local placement benchmark: same-device two-store and `/mnt/e`
two-store lanes preserve recall, keep build time near the one-store baseline,
and show comparable local latency while explicitly stopping short of production
multi-NVMe claims. Relation-backed scan prefetch now batches selected leaf and
delta placements through the object-reader contract and uses PG18 `ReadStream`
per local store relation before scoring, giving Phase 4 a real asynchronous
store-local fetch surface without backend worker threads. PG18 SQL VACUUM
coverage now exercises a two-store relation-backed index through post-build
insert, delete, PostgreSQL's real `VACUUM` callback path, placement
diagnostics, and ordered scan after cleanup, proving both local store relation
placements survive while the deleted row stays invisible and the inserted row
remains fetchable. Relation storage-debt diagnostics now aggregate the
root/control relation and all auxiliary local store relations referenced by
the active placement directory, so cleanup-candidate tuple counts and bytes
reflect the full local store set instead of only the root relation.
PQ-FastScan scorer binding and physical object reclamation/old-epoch cleanup
remain open. Task 30 implements
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

Phase 1 landing scope is the local single-store, single-level `ec_spire`
foundation with TurboQuant and RaBitQ as scannable assignment payload formats.
RaBitQ is the compact scannable target for the Phase 1 storage/recall/speed
tradeoff, with final measured claims still gated on the landing review packet.
Populated PQ-FastScan SPIRE indexes are explicitly deferred to a post-Phase-1
grouped-PQ metadata/scorer slice; this is not a Phase 1 landing blocker. Empty
`pq_fastscan` SPIRE indexes remain supported because they expose options and
diagnostics without scoring assignments.

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
  advance the active epoch. The epoch-manifest codec now carries an explicit
  magic prefix so diagnostic tuple scans can identify epoch manifests
  structurally rather than by tuple length alone.
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
- [x] **Single-store placement.** Persist a PID placement directory even if the
  first executable path maps every PID to one local store. Foundation metadata
  now includes placement-entry and placement-directory codecs, local
  single-store object placements, exact object-manifest/placement PID-set
  validation, and fail-closed delta publication from non-available base
  placements. Partitioned build drafts now publish root and leaf PID placements
  into the local object store. Relation-backed object tuple append/read helpers
  now write and read encoded object bytes from index data blocks after the
  root/control page, and a relation object store can emit/read local
  single-store routing-object placements plus V2 leaf metadata/segment chains
  from those tuples. The relation store now implements `SpireObjectReader` for
  future live snapshot loading. Encoded epoch/object/placement manifest bundles
  can now be written to relation tuples and published through root/control; the
  relation append/read helpers have initial reviewer hardening for stage
  evidence, root/control initialization, root/control special-area bounds, and
  FSM reuse. Populated builds now persist one root routing object, one V2 leaf
  object per centroid, durable placement-entry tuples, manifest bundles, and an
  active root/control state for the initial strict local epoch. Insert,
  delete-delta, and vacuum-compaction replacement epochs now carry forward or
  rewrite relation-backed placement entries under the same strict local
  single-store shape; physical cleanup of no-longer-active object tuples
  remains tracked separately under validation/vacuum cleanup.
- [x] **Build path.** Reuse IVF centroid training, PQ/RaBitQ/PQ-FastScan
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
  locators plus source vectors for AM build wiring. Live relation-backed empty
  build initializes the persisted root/control page. Live populated build now
  collects heap rows, trains the single-level centroid plan using the build
  sample setting, writes relation-backed routing and V2 leaf objects, persists
  placement-entry locators, writes manifest bundles, and publishes the active
  root/control state. TurboQuant and RaBitQ populated builds are supported
  through row-local assignment payloads; populated PQ-FastScan builds now
  report the explicit grouped-PQ model metadata deferral until SPIRE persists
  that model. Active-epoch scan loading and relation-backed snapshot
  diagnostics now consume the persisted epoch.
- [x] **Scan path (TurboQuant/RaBitQ).** Route a query to top-`nprobe`
  partitions, score candidates, and rerank using the same correctness contract
  as local IVF. The
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
  validates the ORDER BY query, reads the relation-backed root/control page,
  loads active epoch/object/placement manifests, reads relation-backed routing
  and V2 leaf objects, exact-reranks the resolved candidate window from the
  heap indexed column for `ecvector`/`tqvector`, and fills the scan cursor.
  Routed scans now also include row-encoded delta insert objects whose parent
  PID is one of the probed leaves, and suppress base or delta-insert candidates
  covered by row-encoded delete deltas for the same base leaf PID. Empty active
  epochs still return no rows, including empty `pq_fastscan` indexes that
  expose the deferred payload format but have no assignments to score.
- [ ] **Post-Phase-1 scan path (PQ-FastScan populated indexes).** Persist
  grouped-PQ model metadata for SPIRE assignment payloads, bind the persisted
  metadata to the PQ-FastScan scorer at scan time, and promote populated
  `pq_fastscan` SPIRE indexes from build-blocked to scannable. Empty
  `pq_fastscan` SPIRE indexes already return no rows safely because there are
  no assignments to score. Populated PQ-FastScan support is intentionally
  deferred and does not block Phase 1 landing.
- [x] **Scan/build option plumbing.** Register SPIRE-owned reloptions and
  session GUCs for the single-level foundation before AM callbacks consume
  them. The AM routine now exposes `amoptions` for `nlists`, `nprobe`,
  `rerank_width`, `training_sample_rows`, `seed`, `pq_group_size`,
  `storage_format`, and `quantizer`; session overrides exist for
  `ec_spire.nprobe` and `ec_spire.rerank_width`. These settings now resolve to
  a helper-level single-level scan plan carrying effective `nprobe`, assignment
  payload format, rerank width, and pre-rerank candidate limit, and the scan
  helper now consumes that plan before live AM callback wiring. Live build now
  consumes `nlists`, `training_sample_rows`, `seed`, and assignment
  `storage_format` for populated index publication; live scan option
  consumption remains part of the scan path task.
- [x] **Admin/diagnostics.** Expose centroid counts, assignment cardinality,
  leaf partition object counts, posting-list row counts, placement map state,
  quantizer profile, and build parameters. The foundation now has an internal
  snapshot diagnostics helper that reports epoch/consistency mode, object and
  placement counts, local-store count, placement-state counts, object-kind
  counts, routing-child count, assignment counts, and available object bytes
  for available local placements. Relation-backed active snapshot diagnostics
  now read persisted manifests and partition objects through the relation
  object store for focused PG18 cardinality coverage. SQL function
  `ec_spire_index_active_snapshot_diagnostics(index_oid)` now exposes the
  active root/control cursors, consistency mode, object/placement/state counts,
  assignment counts, routing-child count, and object byte buckets for the
  active SPIRE epoch. SQL function
  `ec_spire_index_allocator_snapshot(index_oid, warn_within)` now exposes
  active root/control allocation cursors, remaining PID/local-vec-id
  allocations as text, and near-exhaustion flags for a caller-provided warning
  threshold. SQL function `ec_spire_index_options_snapshot(index_oid)` now
  exposes relation `nlists`, `nprobe`, `rerank_width`,
  `training_sample_rows`, `seed`, `pq_group_size`, `storage_format`, resolved
  assignment payload format, session scan overrides, active leaf count, and
  effective `nprobe`/`rerank_width` values with source labels. It also reports
  assignment-payload scannability, status, and recommendation text so
  `pq_fastscan` indexes surface the grouped-PQ model metadata deferral before
  scan-time scorer binding is implemented, and `docs/SPIRE_DIAGNOSTICS.md`
  now records the stable `assignment_payload_status` labels for operator
  tooling while the code uses named constants for those labels. SQL function
  `ec_spire_index_health_snapshot(index_oid)` now reports a conservative
  active-epoch health status, recommendation text, delta compaction
  recommendation flag, placement-state counts, and assignment counts.
  SQL function `ec_spire_index_placement_snapshot(index_oid)` now reports
  one row per active `(node_id, local_store_id, store_relid)` with placement
  counts, placement-state counts, object-kind counts, assignment counts,
  routing-child counts, and object-byte buckets. SQL function
  `ec_spire_index_scan_placement_snapshot(index_oid, query)` now reports one
  row per scan-touched `(node_id, local_store_id)` with resolved scan-option
  labels, scanned PID counts, leaf/delta PID counts, candidate-row counts, and
  delete-delta row counts for the supplied query. SQL function
  `ec_spire_index_root_routing_snapshot(index_oid)` now reports active root
  routing rows with centroid ordinal, child PID, child object kind, child
  assignment count, child placement state, and child store identity. SQL
  function `ec_spire_index_relation_storage_snapshot(index_oid)` now reports
  relation object tuple counts/bytes, active-referenced tuple counts/bytes, and
  cleanup-candidate tuple counts/bytes so old-epoch physical debt is visible
  before tuple reclamation is implemented. SQL function
  `ec_spire_index_scan_sanity_snapshot(index_oid)` now reports resolved scan
  option labels, exact-leaf-coverage status, full-frontier-rerank status, and
  conservative recall-sanity/latency-risk labels. This is a deterministic
  precondition diagnostic, not measured recall or latency evidence. SQL
  function `ec_spire_index_epoch_snapshot(index_oid)` now reports detected
  persisted epoch manifest rows, active-root-manifest status, and cleanup
  eligibility/blocker labels, including superseded manifest rows after an epoch
  state transition, so retention state is visible before old-epoch reclamation
  is implemented. SQL function `ec_spire_index_leaf_snapshot(index_oid)` now
  reports active per-leaf base assignment counts, delta object counts,
  delta insert/delete counts, effective assignment counts, split/merge
  assignment thresholds, read-only maintenance recommendation labels, and
  object-byte totals. SQL function
  `ec_spire_index_insert_debt_snapshot(index_oid)` now reports active leaf
  count, leaf count with deltas, active delta object/insert assignment counts,
  max delta objects per leaf, and whether insert batching is supported or
  recommended. SQL function `ec_spire_index_hierarchy_snapshot(index_oid)` now
  reports the active hierarchy shape as one summary row: root PID/level,
  observed max level/depth, routing/root/internal/leaf/delta object counts,
  centroid dimensions, root child count, distinct leaf parent count, and
  explicit `recursive_routing_supported = false` /
  `per_level_nprobe_supported = false` flags for the current single-level
  foundation. SQL function `ec_spire_index_object_snapshot(index_oid)` now
  reports one row per active manifest PID with object kind, object version,
  published-epoch back-reference, level, parent PID, child/assignment counts,
  placement state, store identity, object bytes, and a readable flag. SQL
  function `ec_spire_index_delta_snapshot(index_oid)` now reports one row per
  active readable delta object with parent leaf PID, object version,
  published-epoch back-reference, store placement, assignment count, and
  insert/delete assignment counts. Operator-facing diagnostic guidance now
  lives in `docs/SPIRE_DIAGNOSTICS.md`. Packet
  `review/30530-spire-phase1-recall-latency-gate/` now carries the Phase 1
  measured recall/latency summary; those rows remain intentionally packet-local
  rather than part of the Phase 1 admin diagnostic surface.
- [x] **Validation.** Add focused PG18 behavior tests for build, scan, empty
  index, insert-after-build, delete/vacuum cleanup, and leaf-assignment
  cardinality. Empty-build, populated-build publication, and populated
  active-epoch ordered scan now have PG18 coverage; empty `pq_fastscan` SPIRE
  indexes now have PG18 coverage proving scans return no rows without invoking
  the deferred scorer path; the populated-build test
  now exercises relation-backed active snapshot cardinality diagnostics and
  live `ecvector` heap rerank, and a separate populated `tqvector` test covers
  the decoded heap-rerank branch. Insert-after-build delta publication now has
  focused PG18 coverage, and empty-index insert bootstrap now has focused PG18
  coverage for first-epoch publication plus a second delta insert. Vacuum
  delete-delta publication and routed scan suppression now have focused PG18
  coverage; post-build insert dimension mismatches now have focused PG18
  error-path coverage through the `ec_spire aminsert failed` wrapper, NULL
  indexed values now have focused PG18 post-build insert error-path coverage,
  populated `pq_fastscan` SPIRE builds now have focused PG18 error-path
  coverage for the grouped-PQ model metadata deferral,
  and five-row post-build inserts now have focused PG18 coverage for one epoch
  per row progression plus per-row query visibility. The SQL active-snapshot
  diagnostics surface now has focused PG18 coverage for empty and
  insert-populated active epochs, allocator SQL diagnostics now have focused
  PG18 coverage for empty and insert-bootstrapped indexes, and the SQL options
  snapshot surface has focused PG18 coverage for reloptions, session
  overrides, active leaf count, and effective scan option resolution. Vacuum
  cleanup compaction of active delta objects into replacement V2 base leaves
  now has focused PG18 coverage, and vacuum compaction now guards malformed
  leaf object header PID mismatches before rewriting an affected base leaf. The
  scan descriptor root-control cache now replaces its observed state on every
  rescan so scan-side cursor fields cannot go stale, including explicit
  coverage for the empty-cache seed observation, and the SQL health snapshot
  surface has focused PG18 coverage for clean and delta-pending active
  epochs. The SQL placement snapshot surface has focused PG18 coverage for
  empty and populated local single-store indexes, plus unit-level aggregate/per-store coverage for
  delta object and delta-assignment byte/count accounting. The SQL scan
  placement snapshot surface has focused PG18 coverage for query-specific
  routed leaf PID and candidate-row counts plus post-build insert-delta
  leaf/delta PID and candidate-row splits.
  The SQL root routing snapshot surface has focused PG18 coverage for empty
  and populated local single-store indexes, plus unit-level malformed active
  manifest coverage for missing-root and multiple-root diagnostics; the SQL
  scan sanity snapshot surface has focused PG18 coverage for empty,
  approximate bounded-leaf, and exact-leaf/full-frontier-rerank configurations;
  the SQL epoch snapshot surface has focused PG18 coverage for empty,
  populated, and post-insert active-epoch publication states, including
  previous-epoch retired manifest copies and superseded manifest labels. The
  SQL leaf snapshot surface has focused PG18 coverage for empty, populated,
  and post-insert delta states, and leaf snapshot aggregation now preserves
  delta counters regardless of manifest iteration order.
  It now also covers read-only merge recommendations for empty leaves and no
  split recommendation for tiny populated leaves. Insert-debt SQL diagnostics
  now have focused PG18 coverage for repeated same-leaf post-build inserts,
  and PG18 external-session coverage now exercises concurrent same-leaf
  post-build inserts through serialized delta epoch publication plus
  index-routed visibility for both inserted rows. PG18 external-session
  coverage now also exercises a heterogeneous insert, SQL VACUUM, and routed
  scan workload released through the same advisory barrier, asserting the final
  epoch counters and live/deleted-row visibility remain coherent for either
  writer order.
  Hierarchy SQL diagnostics now have focused PG18 coverage for empty and
  populated local single-store indexes. Object SQL diagnostics now have
  focused PG18 coverage for empty, populated, and post-insert delta active
  epochs. Placement SQL diagnostics now have focused PG18 coverage for
  post-insert delta object and delta-byte accounting. Delta SQL diagnostics
  now have focused PG18 coverage for empty,
  populated no-delta, post-insert delta, and pre-cleanup delete-delta active
  epochs. Real SQL VACUUM end-to-end coverage now exercises insert-delta
  compaction and deleted-row routed scan suppression; physical page reclamation
  and old-epoch cleanup remain open. Packet-local recall/latency evidence now
  lives in `review/30530-spire-phase1-recall-latency-gate/` rather than in this
  behavior-validation checklist item.
- [x] **Review packet.** Land the single-level foundation with packet-local
  logs and a small recall/latency sanity row. Review packet
  `review/30361-spire-phase1-landing/request.md` records the Phase 1 landing
  boundary, cites the scan-sanity SQL row shape, and includes packet-local
  PG18-feature unit logs for the scan sanity labels, root-control refresh, and
  partial-publish residue behavior.

## Phase 2 — Update Mechanics

- [x] **Cluster split-and-merge plan.** Translate the LIRE/SPFresh-style update
  mechanics into SPIRE's Postgres storage model. Recorded in
  `plan/design/spire-update-mechanics.md`: split/merge allocate replacement
  PIDs when partition coverage changes, rebalance may reuse PID with a new
  object version only when the parent-routing centroid remains byte-equal,
  active deltas are folded into replacement V2 leaves before publication, and
  root/control advances through the existing epoch publish contract. Review
  follow-up now also records whole-root rewrite cost for the single-level
  foundation, old-PID queryability during retention, scheduler choices, and
  PID allocator cursor serialization under the publish lock.
- [x] **Replacement leaf planning helper.** Phase 2 implementation has a pure
  `ec_spire::update` helper for replacement-leaf PID planning and row folding.
  Split and merge allocate replacement leaf PIDs from the observed root/control
  PID allocator cursor, while rebalance reuses the existing PID only when the
  parent-routing centroid remains byte-equal. The row-folding helper reads the
  active epoch snapshot, folds active insert/delete deltas into replacement
  base-leaf rows, clears delta-insert flags on surviving rows, and fails closed
  if an affected PID is not an active leaf. Later Phase 2 slices wired this
  into routing-object rewrite and live scheduler execution.
- [x] **Replacement routing rewrite helper.** Phase 2 now has a pure helper for
  rewriting a parent routing object after replacement leaf planning. It removes
  affected child PIDs, inserts the replacement child PIDs and centroids at the
  first affected position, preserves unaffected child order, reassigns
  sequential centroid ordinals, carries root/internal parent identity, and
  rejects replacement PIDs that collide with unaffected children. The live
  split/merge scheduler now consumes this through relation-backed publish
  wiring.
- [x] **Replacement placement-directory helper.** Phase 2 now has a pure helper
  for planning the new active placement directory for a replacement epoch. It
  carries unaffected active placements with the new epoch, drops the replaced
  parent routing object, drops affected old leaf placements, drops active delta
  placements attached to affected leaves, and inserts the rewritten parent
  routing placement plus replacement leaf placements. This keeps old PIDs
  queryable only through retained prior placement directories while the new
  active directory references the replacement objects.
- [x] **Replacement publish-draft helper.** Phase 2 now has a pure draft helper
  that turns a planned replacement placement directory plus durable placement
  write evidence into a published epoch manifest, object manifest, validated
  epoch snapshot, root/control state, and encoded publish bundle inputs. This
  reuses the existing publish coordinator evidence checks before live
  split/merge relation publishing is wired. Replacement leaf object inputs now
  also validate that leaf-input PIDs match replacement routing children exactly
  and contain only normalized base-leaf rows without delta flags or duplicate
  `vec_id`s.
- [x] **Local replacement object write helper.** Phase 2 now has a local
  object-store helper that writes the rewritten parent routing object and
  replacement V2 leaf objects for a planned replacement epoch. It validates
  replacement leaf inputs, writes routing and leaf objects with published-epoch
  backrefs through the local store, and returns placements ordered by
  replacement routing children. Relation-backed object writes and scheduler
  execution now share the same validation contract.
- [x] **Relation replacement object write helper.** The replacement object
  writer now uses a shared validation and placement-ordering path for local and
  relation-backed object stores. The relation wrapper writes the rewritten
  parent routing object and replacement V2 leaves through
  `SpireRelationObjectStore`, so relation publish wiring can consume the same
  replacement-object placement bundle as the local helper. Scheduler execution
  and root/control publication now consume this relation path.
- [x] **Replacement publish assembly helper.** Phase 2 now has a helper that
  turns replacement object placements plus placement-write evidence into the
  final replacement epoch draft. It plans the new active placement directory,
  drops the affected old leaves and their deltas, validates the object
  manifest/root-control publish shape, and preserves root/control allocator
  cursors supplied by the caller. Live scheduler execution and root/control
  relation publication now reuse this assembly path.
- [x] **Relation replacement publish helper.** Replacement epoch publication now
  has a relation wrapper that accepts already-written replacement object
  placements, writes the new placement-directory rows to the index relation,
  builds the validated replacement epoch draft, retires the previous epoch
  manifest through the existing publish coordinator, writes the new manifest
  bundle, and advances root/control. Live split/merge scheduler execution now
  invokes this relation publisher under the SPIRE publish lock.
- [x] **Replacement scheduler-choice helper.** Phase 2 now has a pure selector
  over the existing leaf snapshot diagnostics. It validates that candidate rows
  come from one active epoch, rejects duplicate or ambiguous split+merge rows,
  prefers the largest split candidate over merge work, and otherwise selects
  the sparsest same-parent merge pair. Live execution still needs to re-load
  and re-check the chosen PIDs under the publish lock before writing
  replacement objects.
- [x] **Scheduled replacement PID planning helper.** Scheduler decisions now
  feed directly into the existing replacement PID allocator helper. Split
  decisions allocate at least two fresh replacement leaf PIDs, merge decisions
  allocate exactly one fresh replacement leaf PID, malformed decisions fail
  before advancing the allocator cursor, and the helper returns the next
  root/control PID cursor for publish.
- [x] **Replacement scheduler recheck helper.** The advisory scheduler now has
  a pure publish-lock recheck helper. It recomputes the selected replacement
  decision from freshly loaded leaf snapshot rows and fails closed if the
  decision disappeared or changed before object writes, preserving the design
  requirement that live execution revalidate selected PIDs under the publish
  lock. The recheck now explicitly documents that selector ranking and
  tie-breaks move in lockstep with this consistency contract.
- [x] **Merge replacement leaf-input helper.** Merge scheduler execution now
  has a pure helper that combines folded rows from the selected affected leaves
  into the single replacement leaf input required by the replacement object
  writer. It validates merge decision shape, requires one fresh replacement
  PID with an advanced PID cursor, rejects missing/extra/duplicate base PID row
  groups, preserves affected leaf order, and reuses the replacement leaf-object
  input validator.
- [x] **Split replacement leaf-input helper.** Split scheduler execution now
  has a pure helper that validates caller-routed replacement leaf inputs against
  a split decision and PID plan. It requires fresh replacement PIDs, exact input
  coverage for every planned replacement PID, PID cursor advancement, orders
  inputs by the PID plan, and reuses the replacement leaf-object input validator
  to reject duplicate
  `vec_id`s or non-normalized rows. Source-vector loading remains a live
  scheduler responsibility.
- [x] **Split replacement materialization helper.** Scheduler execution now has
  a pure helper that accepts selected-leaf source vectors, trains replacement
  centroids with spherical k-means, routes normalized base rows into
  replacement leaf inputs in PID-plan order, and rejects stale base PIDs, delta
  rows, invalid dimensions, zero vectors, or malformed split plans before live
  heap-source loading is wired.
- [x] **Split replacement source-row hydration helper.** Scheduler execution now
  has a pure bridge from folded selected-leaf assignment rows plus fetched heap
  source vectors into the materialization input shape. It preserves assignment
  row order while requiring exact heap-TID coverage and rejecting stale row
  groups, duplicate fetched sources, duplicate assignment TIDs, missing vectors,
  and unused vectors before the live heap fetcher is wired.
- [x] **Split replacement materialization-from-rows helper.** Scheduler
  execution now has a pure composition helper that hydrates selected split leaf
  rows from fetched source vectors and then trains/routes replacement leaf
  materialization through the existing split materializer. Live relation work
  can now focus on exact heap-source fetching and selected-plan invocation.
- [x] **Split replacement source-vector fetch helper.** Scheduler execution now
  has a relation-ready bridge that reuses the SPIRE heap-rerank indexed-vector
  loader to fetch `ecvector` or `tqvector` source vectors for folded split
  replacement rows. The collector returns the exact fetched source records the
  source-row hydration helper consumes, leaving selected-plan invocation as the
  remaining split publish wiring.
- [x] **Selected split source execution-input helper.** Scheduler execution now
  has a selected-plan relation builder that loads the decision-bound parent and
  folded selected leaf rows from the active snapshot, hydrates fetched source
  vectors, trains/routes split replacement materialization, and feeds the
  existing relation split execution-input validator.
- [x] **Selected split heap-source execution-input helper.** Scheduler
  execution now has the relation wrapper that collects folded selected split
  rows, fetches indexed heap source vectors for those rows, and delegates to
  the checked selected-plan source execution-input builder. The remaining live
  publish slice can focus on lock-time orchestration and epoch publication.
- [x] **Split heap-dead-row materialization contract.** Reviewer feedback on
  packet 30448 is handled: heap rows that no longer fetch under the heap
  snapshot are omitted from the split materialization assignment set before
  exact source coverage is validated, so a single dead assignment no longer
  blocks progress for the live rows in the selected split leaf.
- [x] **Scheduled routing replacement child helper.** Scheduler execution now
  has a pure helper that pairs fresh scheduled replacement PIDs with
  scheduler-provided replacement centroids in PID-plan order. It validates
  decision shape, PID count, centroid count, fresh/unique replacement PIDs, and
  PID cursor advancement plus finite non-empty centroid vectors before handing
  exact parent-dimension validation to the existing routing rewrite helper.
  Later selected split/merge builders now supply recomputed centroids and
  relation execution inputs.
- [x] **Scheduled merge replacement centroid helper.** Scheduler execution now
  has a pure helper that recomputes the single merge replacement centroid from
  the affected parent-routing child centroids and active leaf snapshot
  assignment counts. It validates merge decision shape, active epoch, parent
  PID, affected leaf row coverage, duplicate affected rows, child centroid
  dimensions, affected-row merge recommendations, and zero-count sparse merges
  before the centroid is bound to the fresh replacement PID. Split centroid
  training/routing remains open.
- [x] **Scheduled replacement parent loader.** Scheduler execution now has a
  checked seam for loading the decision-bound parent routing object from the
  active snapshot before centroid binding or routing rewrite. It validates
  decision shape, active epoch, available parent placement, routing-object
  kind, parent PID, and affected-leaf child coverage.
- [x] **Scheduled merge replacement routing parts helper.** Scheduler
  execution now composes the merge centroid helper, routing-child builder, and
  parent rewrite wrapper into one pure merge routing preparation seam. It
  returns the rewritten parent plus replacement children after validating the
  PID plan and replacement parent object version.
- [x] **Relation scheduled merge execution-parts helper.** Scheduler execution
  now has a pure relation-input parts builder for merge decisions. It combines
  the merge routing parts with folded replacement leaf rows, validates leaf
  input coverage against the replacement child, and carries publish/retention
  timestamps plus replacement object versions into the existing publish-plan
  input builder shape.
- [x] **Relation scheduled merge execution-input helper.** Scheduler execution
  now composes the merge execution-parts builder with the checked publish-plan
  input builder, producing a fully validated relation scheduled replacement
  execution input for merge decisions before relation object writes.
- [x] **Relation selected scheduled merge execution-input helper.** Scheduler
  execution now has a relation merge builder that consumes the selected
  publish-lock plan directly, keeping the chosen decision, PID plan, and
  publish plan bundled until execution-input construction.
- [x] **Local scheduled merge execution-input helper.** Scheduler execution now
  has the same merge routing/leaf composition for local dry-run execution,
  preserving caller-provided placement-write evidence while sharing relation
  merge validation and publish-plan drift checks.
- [x] **Local selected scheduled merge execution-input helper.** Scheduler
  execution now has a local dry-run merge builder that consumes the selected
  publish-lock plan directly while preserving placement-write evidence.
- [x] **Relation scheduled split execution-parts helper.** Scheduler execution
  now has split-side pure composition for caller-trained replacement centroids
  and routed replacement leaf inputs. It builds replacement routing parts,
  orders split leaf inputs by the PID plan, and returns relation execution
  parts while leaving actual split centroid training as the remaining live
  scheduler responsibility.
- [x] **Relation scheduled split execution-input helper.** Scheduler execution
  now composes split execution parts with the checked publish-plan input
  builder, producing a fully validated relation scheduled replacement
  execution input for split decisions once trained centroids and routed leaf
  inputs are available.
- [x] **Relation selected scheduled split execution-input helper.** Scheduler
  execution now has a relation split builder that consumes the selected
  publish-lock plan directly, keeping the chosen split decision, PID plan, and
  publish plan bundled until execution-input construction.
- [x] **Local scheduled split execution-input helper.** Scheduler execution now
  mirrors relation split composition for local dry-run execution, preserving
  placement-write evidence while reusing split routing, leaf-input ordering,
  and publish-plan drift validation.
- [x] **Local selected scheduled split execution-input helper.** Scheduler
  execution now has a local dry-run split builder that consumes the selected
  publish-lock plan directly while preserving placement-write evidence.
- [x] **Scheduled routing rewrite helper.** Scheduler execution now has a pure
  wrapper that binds a checked split/merge decision to the parent routing
  object rewrite. It validates the decision shape, rejects loading a parent
  whose PID does not match the scheduler decision, rejects replacement-child
  count mismatches and invalid replacement parent object versions, and delegates
  affected-child and centroid-dimension validation to the existing routing
  rewrite helper.
- [x] **Scheduled replacement publish-draft helper.** Scheduler execution now
  has a pure wrapper that binds a checked split/merge decision to replacement
  object placements and placement-write evidence before building the replacement
  epoch draft. It rejects active snapshot/decision epoch mismatches and
  non-successor publish epochs, consistency-mode drift, and replacement
  leaf-placement count mismatches before delegating placement directory,
  manifest, and root/control validation to the existing publish-draft helper.
  Relation selected publish now consumes this checked draft path.
- [x] **Scheduled replacement object-write helper.** Scheduler execution now
  has local and relation object-write wrappers that bind a checked split/merge
  decision to the rewritten parent routing object, replacement children, and
  leaf inputs before writing replacement objects. They reject non-successor
  object epochs, parent-PID mismatches, and replacement-child count mismatches
  before reusing the existing routing/leaf object writer validation. The live
  scheduler now uses the relation variant through selected publish.
- [x] **Scheduled replacement PID-plan output validator.** Scheduler execution
  now has a pure guard that checks written replacement object placements and
  final `next_pid` against the fresh PID plan selected under the publish lock.
  It rejects reused PID plans, wrong parent placement PID, leaf-placement PID
  order mismatches, and cursor mismatches before publish-draft assembly.
- [x] **Local scheduled replacement execution draft helper.** Scheduler
  execution now has a local-store dry-run helper that writes decision-bound
  replacement objects, validates the written placement output against the PID
  plan, and builds the scheduled replacement epoch draft. It catches
  replacement-child order drift before relation callback integration.
- [x] **Local selected scheduled replacement draft helper.** Scheduler
  execution now has a local dry-run draft builder that consumes the selected
  publish-lock plan directly, keeping decision, PID plan, and publish plan
  bundled through object writes and draft assembly.
- [x] **Local selected scheduled split draft helper.** Scheduler execution now
  has a local dry-run split helper that composes selected-plan split
  execution-input construction and scheduled replacement draft assembly.
- [x] **Local selected scheduled merge draft helper.** Scheduler execution now
  has a local dry-run merge helper that composes selected-plan merge
  execution-input construction and scheduled replacement draft assembly.
- [x] **Local selected scheduled replacement draft preflight.** Scheduler
  execution now has a pure local dry-run preflight that validates the selected
  execution input and active snapshot before replacement object writes.
- [x] **Local scheduled replacement publish-plan input helper.** Scheduler
  execution now has a pure builder that carries the checked publish-lock plan
  into the local scheduled replacement execution input while preserving
  caller-provided placement-write evidence. It shares PID cursor,
  replacement-child order, publish-plan successor epoch, decision parent/count,
  rewritten parent contents, leaf object-version, publish timestamp, and
  leaf-input validation with the relation input builder. The local dry-run
  draft helper also takes the checked publish plan and revalidates the
  execution input and active snapshot consistency mode against it before
  writing local objects.
- [x] **Relation scheduled replacement publish helper.** Scheduler execution now
  has a relation-side wrapper that writes decision-bound replacement objects,
  validates written placements against the scheduled PID plan, writes the
  replacement placement directory, builds the scheduled replacement epoch draft,
  and publishes root/control through the existing replacement epoch publisher.
  The manual scheduler now invokes this under the shared publish lock.
- [x] **Relation selected scheduled replacement publish helper.** Scheduler
  execution now has a selected-plan preflight and relation publish wrapper that
  keep decision, PID plan, and publish plan bundled through relation publish
  validation before delegating to the existing relation object-write path.
- [x] **Scheduled replacement publish-lock plan helper.** Scheduler execution
  now has a pure helper that binds root/control active epoch and allocator
  cursors, the active epoch manifest, the checked scheduler decision, and the
  fresh PID plan into the immediate replacement publish plan. It rejects stale
  decisions, non-published active manifests, reused PID plans, PID count or
  duplicate-PID drift, replacement PIDs behind the root/control cursor, and
  final PID cursor regressions before relation writes begin.
- [x] **Scheduled replacement publish-lock allocation helper.** Scheduler
  execution now has one pure wrapper that allocates scheduled replacement PIDs
  and derives the checked publish plan as an atomic lock-step output. It plans
  with a scratch PID allocator and only advances the caller's cursor after the
  publish plan validates against root/control and the active manifest.
- [x] **Rechecked scheduled publish-lock allocation helper.** Scheduler
  execution now has a pure wrapper that rechecks the selected split/merge
  decision against the lock-time leaf snapshot before allocating PIDs and
  deriving the publish plan. If the decision is no longer selected, the caller's
  PID cursor is not advanced.
- [x] **Selected scheduled publish-lock plan helper.** Scheduler execution now
  has a pure selector wrapper that chooses the lock-time split/merge decision
  and returns it with the checked publish-lock plan, or returns `None` without
  advancing the PID cursor when no leaf replacement is currently recommended.
- [x] **Relation scheduled replacement publish-plan input helper.** Scheduler
  execution now has a pure builder that carries the checked publish-lock plan
  into the relation scheduled replacement execution input. It preserves the
  planned epoch, active consistency mode, and local vector cursor while
  rejecting PID cursor drift, reused PID plans, publish-plan successor epoch
  drift, decision parent/count drift, replacement parents missing replacement
  children or still containing affected leaves, and replacement-child PID order
  or leaf object-version/publish timestamp mismatches before relation writes
  begin. The relation publish wrapper also takes the checked publish plan and
  revalidates the execution input and active snapshot consistency mode against
  it before writing relation objects.
- [x] **Selected scheduled execution-input validators.** Scheduler execution
  now has relation and local validators that consume the selected publish-lock
  plan directly, keeping decision, PID plan, and publish plan bundled during
  final execution-input drift checks.
- [x] **Selected scheduled execution snapshot validator.** Scheduler execution
  now has a pure snapshot validator that consumes the selected publish-lock plan
  directly for active epoch and consistency-mode drift checks.
- [x] **Selected scheduled replacement snapshot loaders.** Scheduler execution
  now has selected-plan wrappers for loading the decision-bound parent routing
  object and collecting folded affected-leaf rows from the active snapshot,
  keeping snapshot validation bundled with the publish-lock plan before
  merge/split material preparation.
- [x] **Relation selected scheduled merge snapshot input helper.** Scheduler
  execution now has a relation merge input builder that loads the selected
  parent routing object and folded affected-leaf rows from the active snapshot
  before composing selected-plan relation execution input.
- [x] **Relation selected scheduled split snapshot input helper.** Scheduler
  execution now has a relation split input builder that loads the selected
  parent routing object from the active snapshot before composing selected-plan
  relation execution input with caller-trained centroids and routed leaf rows.
- [x] **Local selected scheduled merge snapshot input helper.** Scheduler
  execution now has a local merge input builder that loads the selected parent
  routing object and folded affected-leaf rows from the active snapshot before
  composing selected-plan local execution input with placement-write evidence.
- [x] **Local selected scheduled split snapshot input helper.** Scheduler
  execution now has a local split input builder that loads the selected parent
  routing object from the active snapshot before composing selected-plan local
  execution input with caller-trained centroids, routed leaf rows, and
  placement-write evidence.
- [x] **Local selected scheduled merge snapshot draft helper.** Scheduler
  execution now has a local dry-run merge helper that loads the selected parent
  routing object and folded affected-leaf rows from the active snapshot before
  composing selected-plan merge draft assembly.
- [x] **Local selected scheduled split snapshot draft helper.** Scheduler
  execution now has a local dry-run split helper that loads the selected parent
  routing object from the active snapshot before composing selected-plan split
  draft assembly with caller-trained centroids and routed leaf inputs.
- [x] **Maintenance plan snapshot.** Scheduler execution now exposes a
  read-only SQL planning surface,
  `ec_spire_index_maintenance_plan_snapshot(index_oid)`, that loads one active
  epoch snapshot, reuses the leaf snapshot collector, chooses the current
  split/merge candidate, derives the checked publish-lock plan with a scratch
  PID allocator, and reports planned action, affected/replacement PIDs,
  successor epoch, and allocator cursors without writing relation objects.
- [x] **Locked maintenance plan snapshot.** Scheduler execution now also
  exposes `ec_spire_index_locked_maintenance_plan_snapshot(index_oid)`, a
  no-write preflight that takes the shared SPIRE publish lock before loading
  the active epoch snapshot and deriving the same checked maintenance plan the
  live manual scheduler entrypoint will re-use.
- [x] **Maintenance run result shape.** Live scheduler orchestration now has a
  shared result row shape plus no-op/projected/published helpers for SQL
  entrypoints, preserving planned action, affected/replacement PIDs, publish
  epoch, allocator cursors, and whether a replacement epoch was actually
  published.
- [x] **Locked maintenance run plan.** Scheduler execution now exposes
  `ec_spire_index_locked_maintenance_run_plan(index_oid)`, a no-write run-plan
  SQL surface that holds the publish lock, loads the active snapshot, chooses
  the same selected replacement candidate as the live scheduler, and reports
  the run-result row with `published = false`.
- [x] **Locked maintenance run-plan SQL smoke.** The no-write run-plan SQL
  surface now has focused PG18 coverage proving a populated merge candidate is
  reported as planned/projected, while active epoch, allocator cursor, and leaf
  count remain unchanged after the locked call returns.
- [x] **Locked run-plan to publish consistency smoke.** The locked run-plan SQL
  smoke now immediately invokes the live maintenance entrypoint after the
  no-write assertion, proving the actual publish reuses the projected action,
  affected PIDs, replacement PIDs, publish epoch, and allocator cursor when the
  active snapshot has not changed.
- [x] **Scheduled replacement object-version plan.** Live scheduler
  orchestration now derives successor object versions for replacement parent
  routing and replacement leaves from active snapshot metadata, rejecting zero,
  overflow, duplicate, or missing affected-leaf versions before relation writes.
- [x] **Maintenance run entrypoint.** Scheduler execution now exposes
  `ec_spire_index_maintenance_run(index_oid)`, which takes the publish lock,
  reloads and rechecks the active candidate, builds merge or heap-source split
  execution input, publishes the scheduled replacement epoch, and returns the
  maintenance run result row with `published = true`. The SQL function is
  marked `VOLATILE` so PostgreSQL treats the manual scheduler as a mutating
  maintenance entrypoint.
- [x] **Maintenance run empty SQL smoke.** The manual scheduler entrypoint now
  has focused PG18 coverage for the empty-index no-action row, proving the SQL
  binding returns `maintenance_status = 'no_action'`, `planned_action = 'none'`,
  `planned_reason = 'empty_index'`, `published = false`, and active epoch 0.
- [x] **Maintenance populated no-candidate SQL smoke.** The manual scheduler
  entrypoint now has focused PG18 coverage for a populated healthy two-leaf
  fixture, proving it reports `no_action` / `no_candidate`, leaves active epoch
  1 and the leaf count unchanged, and does not publish a replacement epoch.
- [x] **Maintenance merge publish smoke.** The manual scheduler entrypoint now
  has focused PG18 coverage for a populated merge publish, including empty
  affected leaves, proving the run publishes epoch 2, reports merge/published
  status, and reduces a three-leaf fixture to two active leaves.
- [x] **Maintenance merge rerun no-op smoke.** The merge publish smoke now also
  immediately runs the manual scheduler a second time, proving the post-merge
  active epoch reports `no_action` / `no_candidate`, does not publish another
  epoch, and keeps the merged two-leaf shape stable.
- [x] **Maintenance split publish smoke.** The manual scheduler entrypoint now
  has focused PG18 coverage for a populated split publish over a skewed
  heap-source fixture, proving the run publishes epoch 2, reports split/published
  status, and expands a ten-leaf fixture to eleven active leaves.
- [x] **Maintenance publish scan visibility smoke.** The merge and split
  publish smokes now also force indexed ordered scans after the replacement
  epoch publishes, proving the manual scheduler's replacement objects remain
  visible to user queries through the SPIRE scan path.
- [x] **Insert path.** Assign new vectors to one partition in the single-level
  path, update assignment rows, and make inserted rows visible to scans.
  Populated strict local indexes now route post-build inserts to one leaf PID,
  write a row-encoded `DELTA_INSERT` object, publish a new active epoch, and
  include routed delta inserts in live scans. The first insert into an empty
  active epoch now publishes epoch 1 with a one-child root routing object and a
  V2 base leaf using the inserted vector as the bootstrap centroid; later
  inserts use the delta epoch path, including focused coverage for multi-row
  inserts that publish multiple deltas on one base leaf. Vacuum cleanup can now
  compact active delta epochs into replacement V2 base leaves. Replacement
  epoch publication now writes a retired manifest copy for the previous active
  epoch before advancing root/control. SQL insert-debt diagnostics now expose
  repeated same-leaf delta fanout and mark batching recommended while
  `insert_batching_supported = false`. SQL delta diagnostics now expose
  active delta objects, parent leaf PIDs, and insert/delete assignment counts;
  insert batching remains open as a performance/scalability follow-up rather
  than a Phase 1 correctness blocker.
- [x] **Delete/vacuum path.** Remove dead assignment rows and posting-list
  entries without breaking scan invariants. The first strict local path now
  runs `ambulkdelete` callbacks over visible base and delta-insert assignments,
  groups callback-dead heap locators by base leaf PID, writes row-encoded
  delete-delta objects, publishes a replacement active epoch, and makes routed
  scans suppress covered `vec_id`s. `amvacuumcleanup` now compacts active delta
  objects into replacement V2 base leaves and removes delta objects from the
  active placement directory, with focused coverage for no-delta cleanup,
  insert-only deltas, and mixed insert/delete deltas on one leaf. Replacement
  vacuum publishes now write a retired manifest copy for the previous active
  epoch before advancing root/control, and the retired manifest helper now has
  focused unit coverage proving callers cannot retire an already-retired
  manifest before relation I/O. Real SQL VACUUM end-to-end coverage now
  exercises insert-delta compaction and deleted-row routed scan suppression
  through PostgreSQL's normal callback path. Physical page reclamation and
  old-epoch cleanup remain open as separate reclamation follow-ups.
- [x] **Split trigger.** Define the partition growth/drift threshold that
  schedules a split. SQL leaf diagnostics now expose per-leaf base, delta, and
  effective assignment counts. The first read-only trigger marks a leaf as a
  split candidate when its effective assignment count is at least
  `max(SPIRE_LEAF_SPLIT_MIN_ASSIGNMENTS,
  SPIRE_LEAF_SPLIT_AVERAGE_MULTIPLIER *
  ceil(total_effective_assignments / active_leaf_count))`; the manual
  scheduler now uses that candidate to publish selected split replacements.
- [x] **Merge trigger.** Define the sparse/low-quality partition threshold that
  schedules a merge. The first read-only trigger marks a leaf as a merge
  candidate when its effective assignment count is at or below
  `floor(ceil(total_effective_assignments / active_leaf_count) / 4)`; the
  manual scheduler now uses those candidates to publish selected merge
  replacements.
- [x] **Concurrency validation.** Concurrent same-leaf post-build inserts have
  a focused PG18 external-session test that verifies root-control
  epoch/allocator serialization, active leaf/delta-assignment accounting, and
  scan visibility. Mixed insert/delete/VACUUM/scan overlap now also has a
  focused PG18 external-session test that releases insert, VACUUM, and scan
  workers from the same advisory-lock barrier, then verifies live-row
  visibility, deleted-row invisibility, and bounded active delta debt.
  Longer-running soak-style stress remains a later hardening/measurement item,
  not a Phase 2 local scheduler landing blocker.

## Phase 3 — SPIRE Recursion

- [x] **Recursive hierarchy design checkpoint.** Phase 3 recursion now has a
  durable design note in `plan/design/spire-recursive-hierarchy.md`. It defines
  level numbering, root/internal/leaf PID invariants, routing child references,
  per-level build and `nprobe` metadata, bottom-up recursive build over leaf
  centroids, level-local scan routing, and the deferred boundaries for
  replication, graph routing, placement, background scheduling, reclamation,
  recursive update propagation, and product-scale measurements.
- [x] **Hierarchy metadata.** Store levels, parent/child partition IDs,
  centroid dimensions, per-level `nprobe`, and build parameters. The
  single-level foundation now persists root/leaf levels, parent/child PIDs, and
  root centroid dimensions, and exposes them through
  `ec_spire_index_hierarchy_snapshot(index_oid)`. The hierarchy diagnostic now
  also runs a pure recursive shape validator over active root/internal/leaf/delta
  metadata, preserving the single-level status while reporting malformed
  recursive parent/child level shapes as invalid before level-aware scan
  routing descends through them. Now that opt-in recursive `ambuild` publishes
  internal routing objects and recursive scan routing is live, the hierarchy
  diagnostic reports `recursive_routing_supported = true` for valid hierarchies
  with internal routing objects while leaving single-level indexes unsupported.
  Durable per-level `nprobe` configuration remains deferred; the current live
  routing policy applies configured `nprobe` at level 1 and probes one child
  above that.
  `ec_spire_index_options_snapshot(index_oid)` now reports `recursive_fanout`
  plus whether the relation is using the recursive build path, effective
  nprobe-per-level and policy arrays, and options / scan-sanity active leaf
  counts now traverse recursive hierarchies instead of treating
  root-to-internal edges as leaves.
  `ec_spire_index_level_parameter_snapshot(index_oid)` now exposes one row per
  active routing level with routing object/child counts, target fanout,
  effective `nprobe`, the current per-level `nprobe` policy, training sample
  rows, training iterations, centroid dimensions, distance semantics, and
  assignment payload format. Recursive hierarchy snapshots now report
  `per_level_nprobe_supported = true` for valid recursive hierarchies that have
  this per-level diagnostic metadata. Deferred hierarchy-metadata follow-ups
  are durable per-level `nprobe` storage/configuration, a durable per-level
  parameter table rather than diagnostic reconstruction, and explicit
  user-facing per-level fanout configuration beyond the current diagnostic
  `target_fanout` exposure.
- [x] **Recursive build coordinator.** Run single-level IVF on input vectors,
  take resulting centroids as the next-level input, and repeat to target depth.
  Phase 3 now has a pure in-memory recursive routing hierarchy draft helper:
  it accepts child PID/centroid records, preserves the single-level root shape
  when the child set is under target fanout, repeatedly trains spherical
  k-means over child centroids when another routing level is needed, allocates
  internal/root routing PIDs from the normal PID allocator, and materializes
  level-aware root/internal routing objects without relation I/O. Build now
  also has a local recursive routing epoch materializer that writes routing
  objects, combines them with already-written leaf placements, validates the
  manifest/directory snapshot, rejects missing leaf placement coverage, and
  verifies each leaf object's stored parent PID against its level-1 routing
  parent. The materializer now uses a shared object-store writer seam with a
  relation-backed entry point for writing recursive routing objects. The same
  seam now also accepts recursive leaf inputs as assignment rows, validates
  duplicate, missing, and unexpected leaf PID coverage plus parent alignment
  before writing, writes V2 leaf objects through the local or relation object
  store, and then materializes the recursive routing snapshot from those fresh
  placements. The
  build side now has a coordinator input assembler that takes the first-level
  centroid plan, allocates leaf PIDs, groups primary assignment rows by
  centroid, builds the recursive routing hierarchy over those leaf centroids,
  attaches each leaf input to its routed parent PID, and returns the recursive
  epoch object input plus allocator cursors for relation publishing.
  Recursive epoch drafts now expose manifest/root-control publish bundle helpers
  that consume the local vector allocator cursor explicitly, so relation
  publishing can use the same publish coordinator path as single-level
  builds without hiding cursor ownership inside the recursive epoch draft.
  Relation publishing now has a recursive epoch bridge that writes placement
  directory entries, rebuilds the object manifest with durable placement-entry
  TIDs, writes the manifest bundle, and installs root/control state through the
  same publish coordinator used by the existing build path. The build module now
  also has a relation recursive build composer that trains the
  first-level centroid plan from `SpireBuildState`, assembles recursive epoch
  input, writes recursive leaf/routing objects through the relation store,
  checks allocator cursor agreement, and invokes the recursive relation publish
  bridge. SPIRE now has an explicit `recursive_fanout` reloption: the default
  `0` preserves single-level build behavior, while values `>= 2` opt into
  recursive routing fanout. Live populated `ambuild` now switches to recursive
  relation build when `recursive_fanout >= 2`, preserving default single-level
  builds while publishing root/internal/leaf hierarchy metadata and routing
  recursive scans through the existing recursive candidate path.
- [x] **Centroid materialization.** Persist each level's centroids so rebuild,
  diagnostics, and query routing can inspect them. The pure recursive routing
  hierarchy draft now emits materialized centroid records for every routing
  parent/child edge, including parent PID, child PID, child level, centroid
  ordinal, dimensions, centroid vector, and source count. Recursive epoch
  materialization now carries those centroid records through the epoch draft so
  the relation publisher has an explicit persistence payload. Relation-backed
  root/internal routing objects durably store the centroid vectors for each
  parent-to-child edge, and
  `ec_spire_index_routing_centroid_snapshot(index_oid)` now exposes those
  persisted centroid vectors, levels, ordinals, child kinds, placement state,
  and parent links through SQL diagnostics.
- [x] **Level-local scan primitive.** Given an input query and a parent
  partition, return child partitions to probe. Scan now has a pure
  `route_routing_object_to_child_pids` primitive that accepts root or internal
  routing objects, applies the existing bounded route heap and deterministic
  score/ordinal/PID ordering, and leaves the existing root-to-leaf wrapper's
  root-kind guard intact for the current single-level scan path. A pure
  recursive routing coordinator now composes that primitive over already-loaded
  routing objects, validates internal child kind/parent/level shape while
  descending, and returns selected leaf PIDs without relation I/O. Scan now also
  has a snapshot preload helper that reads the unique active root plus active
  internal routing objects through the existing `SpireObjectReader` boundary,
  giving live recursive scan wiring a checked root/internal loading seam. The
  routed leaf-row collector now uses that recursive preload/routing path and
  validates each selected leaf against its immediate parent PID, while
  preserving the existing hierarchy root PID in returned scan-row groups. The
  quantized candidate collector now also consumes recursive leaf routes, so V2
  column scoring validates recursive leaf parentage before returning ranked
  candidates; the diagnostics-only helper remains single-level. Scan now has a
  recursive leaf-count helper that traverses active root/internal routing
  objects and distinguishes actual leaf-level children from root child count.
  Recursive route descent now uses the Phase 3 conservative per-level `nprobe`
  policy: configured relation/session `nprobe` applies at level 1, while higher
  routing levels probe one child until durable per-level control lands. Pure
  scan coverage now includes a three-routing-level hierarchy to verify upper
  levels select one internal child per level before applying leaf-level
  `nprobe`.
- [x] **Review packet.** Demonstrate a small multi-level hierarchy where the
  same dataset can be queried as flat single-level IVF and recursive SPIRE. The
  pure scan helper tests now include a four-leaf synthetic hierarchy where a
  flat root and a two-level recursive root/internal shape route the same query
  to the same best leaf. The quantized candidate tests now also build matching
  flat and recursive local object-store snapshots over the same four V2 leaf
  objects and verify the same top candidate is returned. A follow-up proof now
  materializes a recursive routing epoch through the build helper and scans the
  resulting snapshot through the quantized recursive candidate path.
  Relation-backed SQL smoke now covers an opt-in `recursive_fanout = 2`
  populated build: hierarchy diagnostics report internal routing objects and
  depth 2, root-routing diagnostics report root-to-internal children, and an
  ordered scan returns the expected nearest row. The final SQL comparison now
  builds flat and recursive relation-backed SPIRE indexes over the same four
  rows, confirms their hierarchy/root diagnostics differ as expected, and
  verifies both ordered scans return the same nearest row across multiple query
  vectors and top-k set checks.
- Phase 1 recall/latency gate evidence in packet `30530` was measured on the
  single-store surface; packet `30533` covers same-device and `/mnt/e`
  two-store recall parity.

### Phase 3 Closeout Follow-ups

These items came out of the Phase 3 closeout review and are carried forward
explicitly so the boundary between Phase 3 and Phase 4 stays durable:

- [ ] Durable per-level `nprobe` metadata/configuration. Phase 3 exposes the
  effective policy through diagnostics, but still uses configured
  relation/session `nprobe` at level 1 and a conservative one-child probe above
  level 1.
- [ ] Durable per-level parameter storage. Phase 3 exposes level parameters
  through diagnostics reconstructed from active routing objects and reloptions.
- [ ] Explicit user-facing per-level fanout configuration. Phase 3 exposes
  effective target fanout diagnostics, while relation configuration remains the
  single `recursive_fanout` reloption.
- [x] Three-routing-level recursive descent coverage.
- [x] Degraded-placement recursive descent coverage.
- [x] `effective_nprobe_per_level` and `nprobe_policy_per_level` on
  `ec_spire_index_options_snapshot(index_oid)`.
- [x] Pre-Phase-4 guard that refuses split/merge maintenance on recursive
  hierarchies until recursive update propagation lands.
- [x] Parse-time rejection of `recursive_fanout = 1`.
- [x] Conservative recursive nprobe policy naming plus TODO beside the
  hardcoded one-child upper-level policy.
- [x] Dense centroid ordinal assertion for materialized recursive centroid
  records.
- [x] Validation-layering comment naming the in-memory draft, post-write
  placement, and snapshot-time hierarchy barriers.

## Phase 4 — Local Multi-NVMe Placement

- [x] **Local multi-store placement design checkpoint.** Phase 4 now has a
  local placement design in
  `plan/design/spire-local-multistore-placement.md`. The note defines the
  bounded store count/configuration surface, root/control active store set,
  store relation naming/discovery, tablespace mapping, single-store
  compatibility, placement-entry validation, hash placement policy, lock
  ordering, strict/degraded failure semantics, store-grouped fetch boundary,
  and authoritative placement diagnostics. The implementation and measurement
  checkpoints below now close the local multi-store placement slice.
- [x] **Partition-store relation layout.** Multi-store populated builds now
  create the planned bounded auxiliary store relations, initialize their
  SPIRE object-page metadata block, record internal catalog dependencies on
  the root/control index relation, and publish the created store relids in the
  active `SpireLocalStoreConfig`. The single-store path preserves store 0 as
  the root/control index relation. The relation plan still preserves repeated
  tablespace OIDs for same-device baseline runs.
- [x] **Local store configuration metadata codec.** `meta.rs` now has
  `SpireLocalStoreDescriptor` and `SpireLocalStoreConfig` primitives that
  preserve the embedded single-store default, encode/decode a versioned active
  store generation, validate placement entries against the active store set,
  and explicitly allow repeated tablespace OIDs so same-device baseline runs
  are distinguishable from future true multi-NVMe measurements.
- [x] **Hash placement planning primitive.** `meta.rs` now has a fixed
  SplitMix64-style `spire_pid_hash(pid)` and
  `SpireLocalStoreConfig::store_for_pid` helper with stable-value coverage.
  This defines the cross-platform placement rule before object writes move to
  auxiliary stores.
- [x] **Local store count reloption surface.** `ec_spire` now parses a bounded
  `local_store_count` reloption, exposes it through
  `ec_spire_index_options_snapshot`, and allows populated builds to publish a
  logical multi-store baseline; later Phase 4 checkpoints route those stores
  into auxiliary relations.
- [x] **Local store tablespace reloption surface.** `ec_spire` now parses and
  normalizes `local_store_tablespaces`, requires the name count to match
  `local_store_count`, permits repeated names for same-device baseline runs,
  and exposes the normalized string through `ec_spire_index_options_snapshot`.
- [x] **Local store tablespace OID planning.** The build path now resolves the
  normalized tablespace names to OIDs, preserves repeated names as repeated
  OIDs for same-device baselines, and defaults omitted names to the index
  relation tablespace for the multi-store relation DDL helper.
- [x] **Local store relation-name planning.** The build path now derives a
  deterministic auxiliary store relation plan named
  `ec_spire_store_<index_oid>_<store_id>` from the resolved tablespace plan,
  preserving repeated tablespace OIDs for same-device baseline runs and feeding
  the auxiliary relation creation helper.
- [x] **Local store descriptor publish planning.** The relation planning layer
  can now combine a resolved relation/tablespace plan with created store relids
  into a validated `SpireLocalStoreConfig`, preserving repeated tablespace OIDs
  and rejecting missing, duplicate, or unexpected created store relids before
  the catalog DDL helper publishes an active store generation.
- [x] **Active local store config persistence.** The publish coordinator now
  writes the active `SpireLocalStoreConfig` as a manifest-bundle tuple and
  records its TID in root/control. Active manifest loading decodes that config
  and validates every placement against it; insert, vacuum, and relation
  replacement publishes carry the existing config forward instead of
  re-deriving and losing tablespace metadata. Root/control snapshots now treat
  the active config tuple as live.
- [x] **Object store local-store-id surface.** The SPIRE object-store wrappers
  now carry their `local_store_id` when creating and validating placement
  entries, so the next writer slice can select a store descriptor instead of
  relying on hardcoded store `0`.
- [x] **Hash-routed build writer surface.** The build draft path can now write
  through a local object-store set that chooses the target store by
  `SpireLocalStoreConfig::store_for_pid(pid)`, with coverage proving root and
  leaf placements carry the hashed store IDs and relation OIDs.
- [x] **Two-store write+scan-fetch fixture.** The quantized routed scan tests
  now build a hash-routed two-store partitioned draft, read through the
  multi-store object-reader set, and prove scan candidates are fetched from
  leaves placed in both local stores. This closes the in-memory end-to-end
  correctness gap before relation-backed auxiliary store DDL lands.
- [x] **Hash-routed object writes.** Relation-backed populated builds now open
  a writable relation object-store set and place root, internal, and leaf
  partition objects by `hash(pid) % local_store_count`. Multi-store builds now
  write those objects into physically distinct auxiliary store relations, while
  single-store builds continue to use the root/control index relation.
- [x] **Mutation-path local store routing.** Relation-backed live insert,
  delete-delta publication, delta compaction, and live-assignment counting now
  open relation object-store sets from the active local-store config or
  placement directory. Insert and delete deltas are written into the base
  leaf's local store, preserving store-local leaf/delta grouping after the
  initial build. PG18 coverage now proves a post-build insert into a two-store
  index publishes one delta without adding a root-relation fallback placement
  and that ordered scan still returns the inserted row.
- [x] **Parallel local fetch.** Relation-backed scan prefetch now collects all
  selected leaf/delta placements, batches them through
  `SpireObjectReader::prefetch_objects`, groups relation-backed placements by
  `(local_store_id, store_relid)`, and uses PG18 `ReadStream` over each store
  relation's object blocks before scoring. This is async read-ahead within the
  PostgreSQL backend, not backend-thread parallelism.
- [x] **Multi-store SQL VACUUM coverage.** PG18 coverage now builds a
  relation-backed two-store index, performs a post-build insert and delete,
  runs PostgreSQL SQL `VACUUM`, and then asserts active placement diagnostics
  still span both local store ids and both store relation OIDs. The same test
  disables seqscan for ordered scan and verifies the deleted row is not
  returned while the inserted row remains fetchable after cleanup.
- [x] **Scan leaf-route store grouping primitive.** The quantized routed scan
  path now groups selected leaf routes by `(node_id, local_store_id)` before
  fetching leaf/delta object bytes and scoring candidates. Execution remains
  synchronous and uses the current object-reader abstraction until auxiliary
  store relation opening and measured parallel fetch land.
- [x] **Scan leaf/delta read grouping primitive.** The scan grouping layer can
  now combine selected leaf routes with matching delta object routes and group
  both by their own `(node_id, local_store_id)`, filtering deltas whose parent
  leaf was not selected. This keeps the future store-local fetch plan explicit
  before delta header discovery and auxiliary store readers are wired into live
  execution.
- [x] **Store-grouped relation prefetch.** The live quantized routed scan path
  now discovers selected delta routes from object headers, groups leaf and
  delta object reads by local store, and calls the object-reader prefetch hook
  for each selected placement before decoding/scoring that store group.
  Relation-backed stores issue PostgreSQL `PrefetchBuffer` requests for the
  target object tuple block, while in-memory stores keep the default no-op
  behavior. This is an I/O overlap boundary, not measured multi-NVMe evidence.
- [x] **Relation-backed scan store opener.** The relation scan path now builds
  a relation object-store set from the active placement directory, opens
  non-root `store_relid` values in ascending `local_store_id`, and dispatches
  object reads by `(local_store_id, store_relid)` instead of assuming every
  placement lives in the root/control index relation. Later Phase 4 checkpoints
  now create and publish those auxiliary store relations.
- [x] **Real two-relation write+scan-fetch fixture.** PG18 coverage now uses
  two real `ec_spire` index relations as root/control and auxiliary local
  store relations, writes routing and leaf objects across both relation files,
  publishes root metadata with mixed `store_relid` placements, and scans
  through placement-directed relation reads. This closes the "write one object
  to a second store relation and fetch it" design proof without claiming that
  user-facing auxiliary-store DDL is complete.
- [x] **Placement diagnostics.** Expose per-store object count, bytes,
  candidate rows, and scanned PID counts. The first SQL placement snapshot now
  reports active per-store placement counts, store relids, placement-state
  counts, object-kind counts, assignment counts, routing-child counts, and
  object bytes. Active/options/scan-sanity/relation-storage diagnostics now
  read placement-routed logical store sets instead of assuming
  `local_store_id = 0`, and the same-relation two-store baseline has focused
  PG18 coverage across those surfaces. Relation storage-debt diagnostics now
  aggregate object tuple/block counts and cleanup-candidate debt across the
  root/control relation plus auxiliary local store relations. The
  query-specific SQL scan placement snapshot reports scan-touched leaf/delta
  PID counts and candidate rows per local store.
- [x] **Local placement benchmark.** Packet
  `review/30533-spire-local-placement-benchmark/` measures one-store,
  same-device two-store, and `/mnt/e` two-store behavior on the local real 10k
  fixture. The same-device two-store lane uses repeated `pg_default`
  tablespace selection, and the extra-drive lane places store 1 in
  `ecaz_spire_e` at `/mnt/e/ecaz_pg_tblspc/spire_e`. Treat this as local
  placement/regression evidence; product claims still require future
  production/cloud hardware.
- [x] **Larger multi-store regression fixture.** PG18 coverage now includes a
  256-row, 384-dimension, 32-list, four-local-store relation-backed build. The
  fixture asserts placements span all four local stores, routing object bytes
  exceed a single page, relation storage diagnostics see multiple relation
  blocks, and ordered scan still returns a full top-10 result.
- [x] **Auxiliary-store autovacuum behavior guard.** PG18 coverage now opens
  the created auxiliary heap relations and asserts PostgreSQL relcache parsed
  `rd_options.autovacuum.enabled = false`, proving the catalog reloption is
  visible at the boundary autovacuum uses rather than only present as raw
  `pg_class.reloptions` text.
- [x] **Explicit multi-store REINDEX rejection.** Multi-store REINDEX now
  fails with a clear unsupported-lifecycle error while single-store REINDEX
  remains allowed and covered. A future full lifecycle must create and publish
  a fresh auxiliary-store relation set, then retire stale auxiliary store
  relations; internal dependencies alone are not enough to make existing
  auxiliary stores participate correctly in PostgreSQL REINDEX.

## Phase 5 — Boundary Replication

- [x] **Boundary predicate.** Define the threshold/rule for assigning a vector
  to multiple nearby partitions. Phase 5 now has a design checkpoint in
  `plan/design/spire-boundary-replication.md`: boundary replication is
  default-off through a bounded `boundary_replica_count` reloption, uses the
  existing top-N leaf route ordering as the first predicate, derives scan
  `VecIdDedupeEnabled` mode from active replica-capable metadata, and preserves
  Phase 4 hash-by-PID local placement. The first implementation slice has
  landed the parsed reloption, SQL options diagnostics, CLI profile key, and a
  pure route-map helper that resolves primary plus bounded secondary leaf PIDs
  without writing replica rows yet.
- [x] **Assignment fanout.** Extend the assignment writer from one row per
  vector to multiple `(vec_id, pid)` rows. The populated single-level
  relation-backed build path now writes one primary row plus bounded
  `BOUNDARY_REPLICA` rows with the same `vec_id` when
  `boundary_replica_count > 0`; post-build inserts now publish one insert
  delta per selected target leaf with a shared `vec_id`; recursive builds now
  route each source vector through the same top-N boundary predicate before
  writing leaf rows; split replacement materialization now fans out normalized
  source rows across replacement leaves; merge replacement remains primary-only
  because it publishes one replacement leaf.
- [x] **Duplicate control.** Ensure scans deduplicate replicated vector IDs
  before final top-k. Scan candidate collection now treats primary and
  boundary-replica rows as scored-visible and uses the existing
  `VecIdDedupeEnabled` mode for replica-capable scan plans; the default
  primary-only path still resolves to `NoReplicaDedupeDisabled`.
- [x] **Recall study.** Packet `review/30548-spire-boundary-recall-study/`
  measures real-10k recall/storage with boundary replication off/on. With
  `boundary_replica_count=1`, base assignment rows double from 10,000 to
  20,000 and SPIRE index bytes rise from 8.2 MiB to 16.0 MiB. Recall@10 improves
  from 0.9950 to 0.9975 at `nprobe=4` and from 0.9985 to 0.9990 at `nprobe=8`,
  while mean query time roughly doubles at the same `nprobe`. The result keeps
  boundary replication functioning but not compelling for the current real-10k
  default operating point.
- [x] **Storage accounting.** Leaf snapshot diagnostics now report base
  primary rows, base boundary-replica rows, delta boundary-replica insert rows,
  and effective boundary-replica rows so physical assignment growth from
  replication is visible separately from logical row count.

## Phase 6 — Top-Level Graph

- [x] **Graph choice.** Phase 6 now has a design checkpoint in
  `plan/design/spire-top-level-graph.md`. The first top-level graph is a
  single-layer Vamana/DiskANN-style graph over top-level SPIRE routing
  centroids, reusing the pure `ec_diskann` graph core rather than nesting an
  `ec_diskann` AM or introducing a selectable graph abstraction. HNSW and
  build-time graph algorithm selection remain deferred until there is a second
  SPIRE graph implementation.
- [x] **Build integration.** Opt-in recursive builds now publish a durable
  `TopGraph` partition object over the root routing centroids when
  `top_graph_enabled = 1`; the build path rejects top-graph publication unless
  `recursive_fanout >= 2`.
- [x] **Routing integration.** Opt-in scans now load the active top-graph object,
  route from graph-selected root children through recursive SPIRE levels, and
  then reuse the existing quantized scoring, `vec_id` dedupe, and exact-rerank
  pipeline. The default remains flat because `top_graph_enabled = 0`.
- [x] **Diagnostics.** SQL function `ec_spire_index_top_graph_snapshot(index_oid)`
  now exposes active top-graph presence, size, degree/build parameters,
  configured/effective scan fanout, object bytes, and fail-closed status rows
  for missing or duplicated visible graph objects.

## Phase 7 — Multi-Machine Placement

- [x] **Remote node model.** Phase 7 now has a design checkpoint in
  `plan/design/spire-remote-node-model.md`: `node_id = 0` remains the local
  coordinator node, nonzero node IDs are coordinator-scoped remote SPIRE
  storage nodes, remote placements stay in the existing
  `pid -> node_id -> local_store_id` map, and strict/degraded behavior is
  defined around explicit node health, epoch-serving evidence, and
  stale/unavailable diagnostics before libpq execution lands.
- [x] **Remote search API.** `ec_spire_remote_search` is now available as the
  first storage-node SQL endpoint: it accepts query vector, selected leaf PIDs,
  requested active epoch, top-k budget, and strict/degraded consistency mode,
  then returns compact quantized candidate rows with epoch/node/object identity,
  vec-id bytes, opaque row locator bytes, and score. Coordinator libpq fanout and
  retained-epoch serving remain separate Phase 7 work.
- [ ] **Coordinator transport.** Use libpq pipeline mode first for
  coordinator-to-node fanout; do not invent a custom network protocol until the
  SQL/protocol shape fails measurement. Coordinator-side fanout planning now
  groups selected leaf PIDs into local work, per-remote-node target requests,
  and degraded skipped-placement diagnostics before libpq execution lands. The
  plan is SQL-visible through `ec_spire_remote_search_fanout_plan(...)`.
  Target-level request grouping is SQL-visible through
  `ec_spire_remote_search_target_plan(...)`, which emits one row per local,
  remote, or degraded-skipped target group with the selected PID array and
  transport status. Target-readiness planning is SQL-visible through
  `ec_spire_remote_search_target_readiness(...)`, which joins target fanout to
  node descriptor readiness and reports remote targets as blocked by missing
  remote-node descriptors before libpq transport can run. Request-level
  planning is SQL-visible through `ec_spire_remote_search_request_plan(...)`,
  which binds target groups to the storage-node endpoint contract: query
  dimension, top-k budget, consistency mode, endpoint function, and transport
  status. Request-level readiness is SQL-visible through
  `ec_spire_remote_search_request_readiness(...)`, which binds query/top-k and
  endpoint metadata to target/node readiness so missing remote descriptors are
  visible at the same granularity a future libpq request executor will consume.
  Descriptor-aware request readiness is summarized through
  `ec_spire_remote_search_readiness_summary(...)`, which reports one gating row
  with ready/blocked/skipped request counts, blocked PID counts, missing
  descriptor counts, transport counts, and the effective readiness status.
  Request readiness is also
  SQL-visible through `ec_spire_remote_search_request_summary(...)`, which
  aggregates request counts, local/remote/skipped PID counts, executable PID
  count, query dimension, top-k budget, consistency mode, and the effective
  transport/degraded status into one coordinator gating row.
  `ec_spire_remote_search_coordinator_local(...)` now exercises the planned
  coordinator path for local-only fanout by planning selected leaves, executing
  the local target batch, validating the batch, and applying the coordinator
  merge helper. It fails closed before remote-target execution until libpq
  transport lands. `ec_spire_remote_search_coordinator_local_summary(...)`
  exposes the same path's fanout counts, skipped-placement count, merge input
  count, duplicate vec-id count, returned candidate count, and transport status
  for local-ready and remote-target plans. `ec_spire_remote_node_snapshot(...)`
  now exposes node-level diagnostic rows derived from active placement metadata:
  local node readiness is explicit, and nonzero node IDs are reported as
  remote placements that require durable remote-node descriptors before libpq
  fanout execution can be enabled.
  `ec_spire_remote_node_descriptor_contract()` now exposes the durable
  descriptor fields required before real libpq fanout can run; conninfo remains
  an indirect secret reference rather than a raw connection string.
  `ec_spire_remote_node_descriptor_readiness(...)` and
  `ec_spire_remote_node_descriptor_readiness_summary(...)` now project that
  contract onto remote nodes so required missing descriptor fields are visible
  as the precise pre-libpq blocker.
  `ec_spire_remote_node_capability_plan(...)`
  now exposes the pre-libpq capability-check contract per node: required epoch
  window, candidate format, extension version, conninfo source, identity status,
  and readiness status. `ec_spire_remote_node_capability_summary(...)`
  aggregates that contract into one coordinator gate with ready/blocked node
  counts, missing descriptor counts, required candidate format, required
  extension version, and a recommendation.
  `ec_spire_remote_search_execution_plan(...)` and
  `ec_spire_remote_search_execution_summary(...)` now expose the final
  pre-libpq executor contract: local-direct vs. libpq-pipeline transport,
  endpoint function, remote index/conninfo metadata source, candidate format,
  blocked/degraded counts, and effective status.
  `ec_spire_remote_search_libpq_request_plan(...)` and
  `ec_spire_remote_search_libpq_request_summary(...)` now expose the remote
  libpq request envelope without opening connections: SQL template, bind
  parameter count, expected result column count, remote index source, conninfo
  source, candidate format, PID counts, and blocked/readiness status.
  `ec_spire_remote_search_libpq_result_contract()`,
  `ec_spire_remote_search_receive_plan(...)`, and
  `ec_spire_remote_search_merge_input_summary(...)` now expose the next
  receive/merge boundary: result-column schema, per-node candidate batch
  validation expectations, opaque row-locator policy, merge helper, dedupe key,
  tie-breaker, batch counts, and blocked/readiness status.
  `ec_spire_remote_search_row_locator_contract()` and
  `ec_spire_remote_search_finalization_summary(...)` now expose the final
  post-merge boundary: row locators remain origin-node opaque bytes, remote heap
  resolution stays deferred to origin-node lookup, and finalization reports
  whether remote heap fetch is blocked or ready.
  `ec_spire_remote_search_coordinator_gate_summary(...)` now ties execution,
  merge, and final heap-fetch readiness into one coordinator integration gate
  with the next unresolved blocker.
  `ec_spire_remote_search_heap_resolution_contract()` now makes the local vs.
  origin-node heap lookup boundary explicit.
  `ec_spire_remote_search_local_heap_resolution_plan(...)` now decodes
  coordinator-local opaque row locators into heap block/offset work items while
  keeping remote-origin resolution blocked behind the origin-node contract.
  `ec_spire_remote_search_heap_resolution_summary(...)` now aggregates local
  decoded locator counts, remote heap work, and the effective resolution
  blocker for the coordinator.
- [ ] **Distributed epoch manifest.** Publish root/hierarchy/placement metadata
  only after all nodes can serve the requested epoch or report an explicit
  stale-node state. `ec_spire_remote_epoch_publish_readiness(...)` now exposes
  the pre-publish remote-node descriptor gate for active placement metadata:
  remote node counts, remote placement-state counts, blocked/missing descriptor
  counts, readiness status, and recommendation.
  `ec_spire_remote_epoch_publish_plan(...)` now exposes the same gate per
  remote node, including placement-state counts, required served/retained epoch
  windows, observed node epoch windows, and the precise publish blocker.
- [ ] **Graceful degradation policy.** Define strict fail-closed and degraded
  recall modes for unavailable or stale nodes/stores, with degraded mode
  reporting skipped placements explicitly. The coordinator-local summary now
  reports `degraded_ready` when degraded-mode planning skips selected
  placements, and exposes the skipped-placement count alongside merge counters.
  `ec_spire_remote_degradation_policy_contract()` now documents the strict vs.
  degraded placement-state actions that coordinator fanout and distributed
  epoch publication share.
- [ ] **Merge semantics.** Remote candidate merge now has a production helper
  that globally ranks compact candidate rows, dedupes by stable `vec_id`, keeps
  primary placements ahead of boundary replicas on score ties, validates
  candidate envelopes, and applies the final top-k cap after dedupe. The
  coordinator receive boundary now validates candidate batches against the
  requested epoch, expected node, selected PIDs, object version, visible
  assignment flags, vec-id, locator, and score before those batches can enter
  the merge path. `ec_spire_remote_search_merge_order_contract()` now exposes
  the exact comparator order used by that helper. Coordinator integration and
  local heap row resolution after remote candidate selection remain open.
- [x] **Replica deferral.** Record replicated partition objects as future work
  for read throughput and availability; v1 assumes one primary placement per
  PID. Recorded in the Phase 0 storage note as a future
  boundary-replica/remote availability phase; Phase 1 keeps one primary
  placement per PID.

## Phase 8 — Product-Scale Measurement Gate

- [ ] **Background maintenance scheduler.** Add automatic scheduling around the
  existing manual `ec_spire_index_maintenance_run(index_oid)` machinery, such
  as a background worker, VACUUM-time hook, or operator-controlled periodic
  job. Any automated scheduler must keep the Phase 2 lock-time reload/recheck
  contract and reuse the same publish path rather than inventing a second
  split/merge implementation.
- [ ] **Old-epoch physical reclamation.** Physically reclaim or reuse retained
  old epoch object/manifest tuples only after active-query and retention rules
  prove they are no longer needed. Phase 2 preserves retired epochs for
  correctness and exposes cleanup-candidate debt, but tuple/page reclamation is
  a later space-management phase.
- [ ] **Local correctness matrix.** Keep local PG18 tests narrow and focused on
  correctness, WAL safety, and scan behavior.
- [ ] **SPIRE planner cost model.** Replace the
  `cost::gated_planner_cost_estimate(block_count)` stub in
  `src/am/ec_spire/cost.rs` with a SPIRE-aware cost function factoring in
  `nlists`, effective `nprobe`, `local_store_count`, recursion depth, and the
  routing-vs-leaf object distribution from the active snapshot. Implement
  `ec_spire_amgettreeheight` to return the actual recursion depth instead of
  the current hardcoded `0`. Pattern reference:
  `src/am/ec_ivf/cost.rs::compute_amcostestimate` and
  `estimate_ivf_cost`. Required before benchmark-harness and scale-packet
  measurements so `EXPLAIN` and cost-based plan selection across `ec_spire`,
  `ec_ivf`, and `ec_hnsw` are credible. Until this lands, planner choice
  involving `ec_spire` is driven only by block count and any cross-AM
  benchmark comparison through the planner is misleading.
- [ ] **Benchmark harness.** Extend `ecaz` to prepare/load/query SPIRE corpora
  and write packet-local artifacts. Depends on the SPIRE planner cost model
  above for any measurement that traverses the SQL planner.
- [ ] **Scale packet.** Run controlled AWS/RDS-class measurements before making
  product billion-scale claims. Depends on the SPIRE planner cost model
  above.
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
