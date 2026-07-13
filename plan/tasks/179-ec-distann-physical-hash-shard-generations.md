# Task 179: ec_distann Physical Hash-Shard Generations, Handoff, and Publish

Status: done (2026-07-13). Packet 059's outside review accepted Task 163 D8,
Task 179 AC-13's physical 10k/50k/100k matrix, and the aggregate architecture.
Packet 060 closes the conditional recovery-state remediation, aggregate PG18
validation, and task-metadata reconciliation. Packet 061 records the
reviewer's post-closeout safety/resource hardening and follow-on dispositions.
Task 172's broader performance program remains open; Task 167's physical DML
adaptation is now unblocked.

The completed task depends on the accepted Task 163 ADR-085 D8 /
FR-077-CON-4 streamed-stitch evidence and the revised FR-075..FR-083 /
NFR-014/NFR-016..020 contracts. It preserves the replicated control evidence
from the Task 165 implementation line without promoting it as physical proof.

Task numbers 173–178 are already reserved by the parallel Task 173 BatANN spec
and B0–B4 implementation plan. Task 179 is therefore the first available task
number for this corrective physical-storage lane.

Owner: coder. One coder, one branch. Keep each checkpoint in
its owning task packet; the prerequisite D8 spill change is reviewed under
Task 163, not hidden inside this task.

Priority: P0, delivered. The physical fixture and accepted Task 179 matrix no
longer block Task 172 or BatANN multinode work. BatANN reconciliation remains
owned by Tasks 173–178 rather than this task's closeout.

## Why

The current multi-instance lane builds the complete global graph independently
on every PostgreSQL instance, then filters serving ownership. Its destructive
"disjoint" drill deletes non-owned source rows and leaves non-owned graph
records tombstoned. That proves transport, fanout, merge, and fail-closed
behavior against a replicated control; it does not implement FR-078 physical
placement, FR-082 generation publication, or NFR-018 storage accounting.

The current implementation also has one metadata-page generation, a 16-byte
FNV-derived fingerprint, GUC-derived roster identity, live base-heap TIDs, and
caller-selected payload send functions. Those are useful prototypes but are
not compatible with the reviewed physical contract.

## Goal

Land a real distributed-control ec_distann index whose one coherent stitched
graph is stored as disjoint physical owner generations. Each vec_id has one
owner-local graph record and one immutable frozen source row, publication is
commit-only and recoverable, scans register one manifest/fingerprint locally,
and the three-instance fixture proves exact/disjoint topology before Task 172
measures anything.

The implementation-ready checkpoint unblocked Task 172. Task 179's required
10k/50k/100k A/B recall, latency, and storage evidence subsequently landed in
Task 172 packets 002–003 and was accepted by the packet 059 outside review.

## Normative authority

The specification owns behavior and wire/storage identities:

- FR-075: `distributed_control` initialization and scan boundary.
- FR-076: graph record and handoff entry/batch formats.
- FR-077: one-entry-per-vec_id stitched stream and D8 memory bound.
- FR-078: node registry, generation descriptor, physical placement, frozen row
  tier, handoff endpoints, transaction/replay rules, and topology endpoint.
- FR-079: expansion and attnum-based row materialization.
- FR-082: manifest v2, 34-byte fingerprint, decision/publish/recovery order,
  coordinator scan retention/retire fencing, and active-pointer semantics.
- FR-083: physical mutation model; full DML adaptation remains Task 167.
- NFR-014/NFR-016/NFR-017..020 and TC-040/TC-042/TC-044/TC-050.

This task selects implementation mechanisms and checkpoint order. If code needs
a different observable contract, update and review the spec first.

## Current-to-target boundary

| Surface | Current implementation | Task 179 target |
| --- | --- | --- |
| Root index | One directly scanned local graph | Metadata-only logical control index when `distributed_control=true`; legacy single-node mode preserved |
| Physical graph | Full replica per node | One hidden owner generation per participant; exact/disjoint global coverage |
| Source row | Live base-heap TID | Immutable AM-owned row-tier heap copied from the build snapshot |
| Directory | In-index sorted vec_id→record-TID chain | Generation-local unique vec_id directory over the physical graph store |
| Epoch | One v4 metadata-page state | Multiple Building/Ready/Published/Retired generations plus Aborted/Reclaimed state |
| Fingerprint | 16-byte FNV-style identity | FR-082 v2: `u16_le(2) || SHA-256(manifest)` (34 bytes) |
| Roster | Session GUC with raw conninfo | Ordered node-descriptor catalog with secret references; immutable manifest snapshot |
| Row payload | Live owner heap + caller column/function names | Frozen row tier + attnums; owner resolves local send functions and coordinator resolves receive functions |
| Publication | One local metadata-page write | Durable decision transaction, participant-first publication, coordinator active-pointer-last |
| Fixture | Replicated build plus delete/tombstone pruning | Coordinator source + schema-compatible owner shells + true streamed owner handoff; no pruning |

## Implementation architecture

### 1. Preserve the legacy lane and add an explicit control format

- Add `distributed_control` to `EcDistannOptions`; default `false` must remain
  byte-for-byte compatible with the current single-node build and scan path.
- Bump/freeze the ec_distann control metadata format under NFR-016. Persist a
  never-reused logical-index UUID and the distributed-control flag so local
  catalog rows cannot alias a later relation after PostgreSQL OID reuse.
- In `ambuild.rs`, a distributed-control build initializes only control
  metadata. It writes no node record, directory, codebook, source vector, or
  legacy active epoch. Direct `amgettuple` on an unpublished control index
  errors; multinode reads are CustomScan-only.
- Require `source_identity='include'` with exactly one non-NULL UUID/bytea16
  identity attribute for every physical build. Reject the current local
  heap-TID identity mode before snapshot capture; it remains valid only for the
  single-node legacy lane.
- Preserve an explicit `replicated-serving-control` fixture mode. It must never
  be selected implicitly and must keep its current non-gate label.

Primary files: `options.rs`, `page.rs`, `ambuild.rs`, `routine.rs`,
`custom_scan.rs`, `sql/bootstrap.sql`, plus TC-050 fixtures.

### 2. Freeze codecs, schemas, manifests, and wire formats first

Create narrow modules instead of extending `remote_endpoint.rs` or
`epoch_manifest.rs` into new monoliths:

- `handoff_wire.rs`: FR-076 entry/batch encode, decode, preflight, and SHA-256
  domains.
- `row_schema.rs`: canonical descriptor/fingerprint, catalog send/receive
  resolution, attnum validation, and supported-type preflight.
- `generation_descriptor.rs`: versioned descriptor and complete codec artifact;
  add serialize/restore support to `DistannCodecBinding`, including trained
  GroupedPQ codebooks (owners never retrain). The descriptor also carries the
  ordered `(node_id, logical_index_uuid, endpoint_identity)` roster and
  placement-hash version so an owner derives its ordinal without session GUCs.
- `manifest_v2.rs`: Ready receipt, canonical manifest, manifest digest, and
  34-byte fingerprint. Do not mutate the current v1/FNV helper into a
  conditionally interpreted byte array.

Before endpoint work, add golden fixtures, independent decoders, byte-swap and
unknown-version rejection, layout assertions, and upgrade-matrix rows for every
new persisted/wire object. The physical lane is rebuild-only; no implicit v4
to distributed-generation migration.

### 3. Use transactional PostgreSQL relations for generation storage

Do not append handoff batches directly to the current custom index pages: those
writes are not MVCC rows and cannot satisfy whole-batch rollback after an error
or backend crash. Use ordinary WAL-logged heap/B-tree relations for the first
correct implementation.

For each local `(logical_index_uuid, build_id)` generation:

- Create one hidden row-tier heap relation from the participant's
  schema-compatible source-shell physical attnum/type/typmod/collation layout.
  Make captured columns nullable and copy no defaults, CHECK/NOT-NULL/identity
  constraints, or generated expressions. Insert decoded source datums directly
  so generated values are stored, not recomputed, and normal PostgreSQL TOAST
  handles wide values. The relation is immutable after Ready.
- Create one hidden graph-store heap with fixed columns equivalent to
  `(vec_id bigint, graph_record bytea, row_tid tid)`. Insert the row-tier tuple
  first, encode its local CTID into `DistannNodeTuple`, then insert the graph
  row.
- Create one unique B-tree on graph-store `vec_id`; this is the generation-local
  sorted directory and duplicate guard. Keep `row_tid` alongside the encoded
  record so topology inspection can detect locator disagreement and orphans.
- Record internal PostgreSQL dependencies from every hidden relation/index to
  the logical control index. Raw local relation OIDs live only in the local
  generation catalog; they never enter a descriptor, receipt, manifest,
  fingerprint, log, or remote payload.
- Count graph heap, graph TOAST, directory B-tree, row-tier heap, and row-tier
  TOAST separately for topology/storage results; report the logical control
  index in its own `control_index_bytes` column so NFR-018 can include it.

Adapt the proven `ec_spire/storage/relation_plan.rs`
`heap_create_with_catalog`/`DEPENDENCY_INTERNAL` helpers where their ownership
model matches. Do not share SPIRE epoch state or partition-object semantics.

Add extension-owned catalogs (exact SQL names may be adjusted together in one
reviewed schema checkpoint):

- `ec_distann_participant_identity`
- `ec_distann_registry_state`
- `ec_distann_node_descriptor` (desired roster only)
- `ec_distann_generation`
- `ec_distann_generation_batch`
- `ec_distann_build_registration`
- `ec_distann_build_participant_binding` (private immutable transport snapshot)
- `ec_distann_build_candidate` (immutable T2 manifest/spec/snapshot/receipt state)
- `ec_distann_publish_decision`
- `ec_distann_retire_decision`
- `ec_distann_active_epoch`
- `ec_distann_generation_reclaim` (participant idempotency tombstone)
- `ec_distann_cancelled_generation_reclaim` (never-active cancellation tombstone)

Every row is keyed by logical-index UUID in addition to the local index OID.
Catalog and endpoint privileges are revoked from `PUBLIC`. DROP/REINDEX tests
must prove hidden relations are dependency-cleaned and stale catalog rows are
never addressable after OID reuse.

Packet 006 extends these catalog contracts before lifecycle RPCs: registration
stores source OID and enforces one gate-active build; candidate rows byte-bind
registration/spec/descriptor/snapshot/receipt/manifest identities; generation
rows persist descriptor-v2 coordinator identity, Published manifest/fingerprint,
and successor-retirement marker; publish decisions carry predecessor identity
and Pending/Activated/Applied or audited terminal Cancelled progress; retire decisions carry canonical bytes,
target build/epoch/private roster, and Pending/Applied progress; reclaim
tombstones retain exact status/replay fields after relation deletion. One
predecessor-disposition row per immutable binding records Pending, exact
Retired acknowledgement, or explicit audited Abandoned terminal state; the
publish decision reaches Applied only when every row is terminal. Abandonment
is operator-only, never implies remote reclaim, and remains immutable through
ordinary cleanup.
Because authoritative coordinator identity was missing from the already-drafted
descriptor v1, Packet 006 also owns the deliberate descriptor-v2 encoder,
decoder, digest domain, offsets/layout assertions, independent golden fixture,
compatibility-matrix row, handoff migration, and explicit v1 rebuild-only
rejection before later lifecycle code consumes the format.

Persist stage/seal restart state in `ec_distann_generation`: the last unsigned
vec-id as eight canonical bytes, one explicitly versioned serializable SHA-256
owner-stream state, and the exact 303-byte Ready receipt after seal. Pin the
direct SHA implementation/version used for serialized state and prove resumed
hashing equals one-shot owner-stream hashing at every chunk boundary. The batch
journal stores digest, encoded length, counts, and receipts, not complete batch
payloads. Seal independently reconstructs all public/physical digests from the
relations rather than trusting cumulative catalog values.

Implement participant identity configuration as an insert-only durable local
binding. Node registration resolves the secret, queries the remote v5 control
identity plus canonical compatibility digest/configured endpoint/canonical
index locator, compares it to the coordinator control, and stores only returned
identity plus the secret reference in the desired registry. Register,
unregister, and begin-build serialize through one registry-state row; begin
copies private transport bindings by build id so later desired-roster edits do
not break active or retained epochs. Add the participant-local unpublished
generation listing for orphan reconciliation; do not infer identity or recovery
state from relation names/OIDs.

Suggested modules: `generation_catalog.rs`, `generation_store.rs`, and
`topology.rs`.

### 4. Close Task 163 D8, then produce a canonical handoff stream

This prerequisite lands and is reviewed in the next Task 163 packet:

- Write each sorted shard output to PostgreSQL-managed `BufFile` temporary
  storage.
- Replace retained `Vec<ShardGraph>` inputs with bounded cursors and k-way
  merge one vec_id group at a time.
- Report/verify peak retained bytes for cursors + one union/prune group; the
  existing `shard_output_retained_node_ids` evidence remains the before case.

Task 179 then changes `ambuild.rs` from "build and immediately stage one local
DataPageChain" to "build and yield canonical graph entries":

- Retain only vectors/identity needed by Vamana and codec training. Spool full
  source-row payloads to PostgreSQL-managed temporary storage rather than
  retaining a second epoch-sized payload in memory.
- Under the source session lock, drive `table_index_build_scan` with one
  registered MVCC snapshot and concurrent-build visibility. Resolve each
  callback index-entry TID through the table AM index-fetch API (including HOT
  root→visible-member traversal), compare its indexed vector/source identity
  with the callback datums, and serialize every non-dropped attribute through
  locally resolved `typsend`. Reject a defensive callback-dead invocation
  before datum access. Add focused coverage for NULL, generated, toasted,
  dropped, unsupported, HOT-updated, and recently-dead-excluded tuple cases.
- During that capture pass, reject any row whose complete eventual handoff
  entry (row payload plus fixed graph/code payload) can exceed 8 MiB, before
  Vamana construction or any participant `begin`.
- Join the one-group stitched output to its spooled source payload and emit one
  `distann_epoch_handoff_entry` per vec_id in global vec_id order.
- Route entries into one bounded buffer per owner. Each buffer is at most 8 MiB;
  one batch per owner may be unacknowledged. Flush/retry without ever collecting
  a complete owner or epoch in memory.

### 5. Implement owner handoff as validated MVCC transactions

Add a small `handoff.rs` service behind the exact FR-078 SQL wrappers.

- `begin`: authenticate, validate endpoint/logical-index identity and the full
  generation descriptor before relation creation; return prior progress for an
  exact replay.
- `stage`: verify declared lengths/digest, fully decode and validate every entry
  before the first insert, lock one generation row, insert row-tier + graph +
  directory rows, journal the batch digest/receipt, and update cumulative
  counts/digest in one ordinary PostgreSQL transaction.
- `seal`: rescan physical relations, verify expected counts/digests, local
  ownership, record/row locator agreement, unique coverage, and physical bytes;
  then transition Building→Ready and emit the canonical receipt.
- `abort`: idempotently drop only an unpublished generation and its batch rows.
  It must refuse a generation named by a durable publish decision.
- All participant handoff calls use READ COMMITTED. Before Packet 006 adds
  remote publication dispatch, add a runtime isolation check that rejects a
  participant lifecycle call outside READ COMMITTED before RPC or mutation.
  Publish-decision insertion
  takes the same control-index `ShareRowExclusiveLock` before catalog rows, so
  it cannot race abort between the decision check and guarded generation drop.
- An identical acknowledged replay returns the journaled receipt. A conflict,
  sequence error, malformed entry, wrong owner, duplicate, schema/codec error,
  or oversize input performs zero relation/catalog mutation.

Participant restart coverage must resend exactly the first unacknowledged
batch while the coordinator still owns the source lock and build workspace.
Coordinator loss before a durable decision follows this task's v1 choice:
abort the unpublished remote generation; do not attempt to reconstruct the old
MVCC snapshot.

### 6. Split Ready, decision, and recovery across real commit boundaries

The operator/CLI sequence uses one coordinator connection: four committed
transactions for the first publication with no predecessor, and five or more
when predecessor retirement marking is required:

1. `ec_distann_begin_epoch_build` acquires the source and coordinator-control
   session locks,
   snapshots the registry/reloptions/schema identity, and commits the durable
   build registration/gate before any remote handoff call.
2. `ec_distann_build_epoch` captures/builds/hands off and returns a Ready
   manifest candidate in a new transaction using that registration.
3. `ec_distann_decide_epoch_publish` re-reads owner topology/receipts and commits
   the immutable commit-only decision. It must not contact publish endpoints.
4. In T4a, `ec_distann_recover_epoch_publish` idempotently publishes successor
   participants, waits for matching acknowledgements, conditionally swaps the
   coordinator active pointer from the recorded predecessor, records Activated
   (or Applied when no predecessor exists), clears the build gate, and releases
   both session locks only after commit.
5. In T4b, a later recovery invocation uses immutable predecessor bindings to
   mark every reachable old-roster participant Retired—including removed owners. A
   permanently unavailable binding can become terminal only through the
   immutable operator-authorized abandon-binding audit, inserted atomically
   with the Pending→Abandoned CAS and replayed from stored bytes/time. Recovery marks the
   decision Applied only when every binding is exact Retired or audited
   Abandoned. It performs no source recapture and needs no live-build
   source/control lock.

The shared order for begin, abort, and pre-activation publish recovery is source
session `ShareLock` → coordinator control session `ShareRowExclusiveLock` → registry row →
registration/decision rows. A lock acquired in an aborted top-level or
subtransaction is released by callbacks; committed ownership is build-specific.
The registration digest covers the complete private binding list, not only the
public roster.

PostgreSQL also drops previously committed default-lock-manager session
relation locks when that backend later aborts a top-level transaction. Treat
that event as loss of ephemeral ownership, not loss of the durable gate: clear
the backend-local mirror, keep the registration, and require exact source then
control reacquisition before recovery resumes. The durable DML/utility gate is
what closes this interval, so begin-build must not be promoted independently of
that enforcement.

T2 persists an immutable `ec_distann_build_candidate` containing canonical
build spec, generation descriptor, source snapshot, complete receipt set, and
manifest bytes/digests before returning Ready. T3 never reconstructs its input
from client memory. T3 and every T4 recovery transaction recompute the complete
candidate digest chain over the stored canonical bytes before consuming it.

The successor publish decision stores its all-or-none predecessor build/epoch/
fingerprint/manifest tuple, canonical activation marker, and
Pending→Activated→Applied phase. An operator-only Pending→Cancelled CAS verifies
that exact predecessor is still active, records caller/reason/time, clears the
build gate without deleting the durable fingerprint registration, and leaves
any partially Published successor storage non-routable until explicit
cancelled-publish recovery replays the private bindings. Each participant
atomically records the canonical cancellation audit tombstone before deleting
Ready or Published-but-never-active storage; partial remote cleanup is
idempotently re-driven. Participant generations persist the published
manifest/fingerprint and exact successor marker. Retire apply leaves an
immutable `ec_distann_generation_reclaim` tombstone carrying canonical decision
bytes and status fields.

Extend the existing SPIRE DML/utility-hook pattern only far enough to enforce
the durable build gate after coordinator-session loss: source DML and
schema-changing DDL fail closed until explicit pre-decision abort or
post-decision recovery. Reads of the prior Published epoch continue.

Replace the current one-page `epoch_manifest.rs` state machine with catalog
generation state; keep the legacy local implementation behind
`distributed_control=false`. Exercise crashes before decision, after decision,
after each successor acknowledgement, before pointer swap, after pointer swap,
and after each predecessor retirement mark including a removed owner.

### 7. Register scans locally and make retirement the expensive path

- Add a coordinator-local in-flight registry keyed by
  `(logical_index_uuid, fingerprint, scan_token UUID)` and an RAII guard reached
  by normal completion, `EndCustomScan`, rescan, epoch-mismatch restart, remote
  error, statement timeout, and cancellation.
- Use exactly one retirement fence per logical-index UUID; no per-fingerprint
  fence exists. Atomically select/register the active fingerprint under that
  per-index fence before expansion, and reject registration when the selected
  fingerprint already has a committed retire decision. Do not add participant
  pin RPCs, participant catalog writes, WAL flushes, or synchronous commits to
  the query path.
- Implement normal retire by exclusively fencing new registrations, requiring
  the target fingerprint's local count to reach zero, re-checking that the
  target is non-active, and holding the fence through the durable retire-decision
  commit. Release the fence before applying idempotent reclaim to participants;
  later registrations are rejected by the decision. Add recovery for a crash
  after a subset applies. On an active-count rejection, release the fence and
  create no decision; a concurrently arriving scan waits for the local critical
  section rather than failing on fence contention.
- Require a covering successor decision to be Applied before normal or forced
  retirement of its predecessor fingerprint; Activated cannot enter reclaim.
- Carry the exact abandoned ordinal/audit-digest set into the canonical retire
  decision. Retire recovery skips those forfeited bindings, applies reclaim to
  every non-abandoned binding, and preserves the abandonment audits rather than
  claiming the unreachable participant reclaimed.
- Participants never reclaim autonomously, so their restart cannot race a live
  scan. Force-retire remains a non-active-epoch operator override with the full
  FR-082 audit record.
- The scan-token registry and sole per-index fence use PostgreSQL add-in shared
  memory and require `ecaz` in `shared_preload_libraries` for distributed-control
  serving. Participant reclaim leaves an immutable tombstone. Successor publish
  recovery marks each reachable predecessor-roster participant Retired after
  the active pointer swap, including owners removed from the successor roster;
  an explicitly abandoned unreachable binding remains an unroutable Published
  orphan pending out-of-band cleanup. Define
  liveness by exact ProcNumber/PID plus an extension-maintained per-ProcNumber
  generation. Fence-map entries carry operation references covering waiters and
  holders; dependency cleanup may recycle a dropped UUID's fence id only after
  its tokens and operation references reach zero, so DROP/CREATE churn does not
  exhaust capacity or alias a live locktag.

Suggested module: `generation_retention.rs`; keep this separate from beam
counters.

### 8. Make expansion and materialization generation-aware

Introduce a read adapter rather than scattering legacy-vs-physical branches:

- `LegacyLocalStore`: current index-page directory + live heap, only when
  `distributed_control=false` and roster size is one.
- `PhysicalGenerationStore`: manifest fingerprint → local generation catalog →
  unique directory lookup → graph-store record → immutable row-tier CTID.

Update `expand.rs`, `reader.rs`, `remote_endpoint.rs`, `remote_transport.rs`,
`routine.rs`, and `custom_scan.rs` through that adapter.

- `ec_distann_expand_nodes` loads codec/schema metadata from the selected
  generation, not the logical control root or session GUC.
- Replace the current `ec_distann_materialize_rows`/caller-selected
  column/function path with the FR-079 attnum + schema-fingerprint endpoint.
  The owner resolves `typsend`; the coordinator validates the same schema and
  resolves `typreceive` before constructing the virtual tuple.
- Apply coordinator quals only after complete tuple reconstruction. Preserve
  request order and zero-partial-batch behavior.
- Remove physical-lane fallbacks that materialize a remote hit through the
  coordinator's local heap or directory. A missing generation/record/row is a
  classified structural error.
- Load the head sample and codec artifact for the exact registered manifest so
  retained old and new fingerprints can be read concurrently.
- On first use per exact control-UUID/build/fingerprint identity, validate and
  build the immutable head graph, then retain only descriptor/head state in a
  two-entry backend-local LRU. Never cache conninfo, relation handles, active
  pointer state, or scan tokens. Keep a Userset off switch for suite-driven A/B.
- Bound every remote connection and RPC with nonzero Userset connect and
  statement/call budgets. Apply the remote `statement_timeout` when a pooled
  session is opened, retain client-side deadlines for lifecycle and scan calls,
  and check PostgreSQL interrupts before and after each awaited RPC.

### 9. Replace the fixture's pruning drill with a physical build

Refactor `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs`:

- Load the source corpus only on the coordinator. Create schema-compatible
  owner shells and metadata-only distributed-control indexes on participants.
- Register the ordered roster through secret references/local-dev resolver,
  then drive build→Ready, commit decision, and recover/publish through the
  public operator sequence above.
- Delete the build-then-delete/tombstone path from the physical lane. Preserve
  it only under the explicit `replicated-serving-control` name.
- Run build-id-selected generation topology before the publish decision and
  `ec_distann_epoch_topology` after activation on every participant; emit
  structured rows for state, record/row counts and digests, non-owner residue,
  orphans, and graph/row/directory bytes.
- Make the suite runner invalidate all downstream rows if topology is absent,
  incomplete, not Published, or disagrees with expected owner streams.
- Exercise coordinator-outside-roster, coordinator-in-roster, one-owner
  degenerate, and three-owner cases. Task 172's required gate fixture is three
  owners; a coordinator outside the roster is a required correctness case.

The topology step/config belongs in `ecaz bench suite`, not a packet-local
script. TC-040/TC-042 logs and normalized rows land under Task 179; TC-044's
10k/50k/100k measurements land under Task 172.

### 10. Fail closed on physical DML until Task 167 is adapted

Task 179 does not silently reinterpret the current local delta/fold path.
INSERT/UPDATE/DELETE against a Published distributed-control index must report
the explicit unsupported physical-DML status until Task 167 routes complete
row/record replacements to owners. Single-node legacy DML remains unchanged.

Task 167 then owns the FR-083 append/redirect/back-edge work against these
generation relations and row-schema descriptors.

## Checkpoints and review packets

The implementation landed as narrow code commits followed by separate request
commits. The original nine-packet forecast expanded into these reviewed groups:

1. Task 163 packets 003–005 — D8 `BufFile` spill, bounded cursors, scale memory,
   and recall neutrality (the prerequisite, outside accepted in packet 059).
2. Task 179 packets 001–006 — specification, frozen formats, transactional
   generation storage, registry, streamed handoff, and lifecycle schemas.
3. Packets 007–018 — durable build gate, coordinator endpoints, committed
   decision/recovery boundaries, crash windows, and lock-order remediation.
4. Packets 019–030 — scan-token RAII, physical reads, retirement/reclaim,
   multi-owner handoff/publication, remote serving, and bounded head state.
5. Packets 031–038 — real three-instance fixture, suite support, lifecycle
   recovery, epoch caching, bounded transport, and head-cap evidence.
6. Packets 039–046 — endpoint security, cancellation safety, parallel fanout,
   honest warmup, fanout A/B, and system-column rejection.
7. Packets 047–054 — legacy-seed/direct-reader A/Bs, prompt cancellation,
   physical publish fault windows, and `DROP EXTENSION` cleanup.
8. Packets 055–058 — utility/build-gate correctness, raw suite results, DML
   overhead A/B, and the transactionally invalidated inactive-gate fast path.
9. Packet 059 — sole aggregate outside closeout; AC-1 and AC-13 accepted and
   aggregate accepted with three mechanical conditions.
10. Packet 060 — rows-affected/state-pair remediation, aggregate PG18 evidence,
    and final housekeeping; this packet records the done transition.
11. Packet 061 — post-closeout interrupt, transaction-fence, and CustomScan
    cleanup hardening plus explicit disposition of the reviewer's follow-ons.

Each measurement packet needs its suite config, manifest, results JSONL, and
only cited logs. Do not commit corpus TSVs, PostgreSQL operational logs, tunnel
state, polling exhaust, or packet-local sweep scripts.

## Verification plan

Run the narrowest PG18 checks appropriate to each risky checkpoint:

- pure Rust codec/digest/schema/placement tests;
- NFR-016 fixture, upgrade-matrix, endian, and layout checks;
- focused PG18 pg_tests for relation/catalog/WAL/rollback behavior;
- three-instance TC-040 handoff/materialization tests;
- three-instance TC-042 lifecycle/fault/retention tests;
- `ecaz bench suite` physical topology preflight and 10k implementation smoke;
- Task 172 A/B at 10k/50k/100k for physical vs single-node and explicitly
  labeled replicated control: recall + latency + cluster storage.

Do not rerun unrelated PG17 coverage unless a touched compatibility surface
requires it or the user asks.

## Acceptance criteria

1. Task 163 D8/FR-077-CON-4 has outside-reviewer closure with bounded spill
   evidence before Task 179 claims streamed handoff.
2. TC-050 covers every new descriptor/receipt/manifest/fingerprint/control
   format with golden, independent-decode, endian/version, and layout evidence.
3. `distributed_control=true` creates no graph replica and cannot return a
   pre-publish empty/legacy-local result; `false` preserves legacy behavior.
4. Every valid handoff batch commits row tier, graph record, unique directory,
   batch journal, and cumulative receipt atomically; every invalid or aborted
   batch changes none of them.
5. Trained-codec owners restore the coordinator artifact and produce identical
   prepared-query/code scoring without retraining.
6. Ready/publish/recovery follows a real committed decision boundary and every
   crash drill ends with the old active epoch or the fully acknowledged new
   epoch, never mixed/partial state.
7. Coordinator-local scan registrations cover normal/error/cancel/restart paths,
   add no participant query-path work, and retire decisions prevent reclaim;
   force-retire is explicit, non-active-only, and fully audited.
8. Expansion and materialization read only the registered physical generation;
   owner-local row CTIDs never cross the wire, caller-selected functions are
   absent, and frozen projections/quals match the source snapshot.
9. The topology endpoint and suite preflight prove exact global coverage, empty
   pairwise ownership intersections, correct hash owners, one row per record,
   zero live/tombstoned non-owner residue, zero orphans, and coordinator
   in/out-roster behavior.
10. The physical three-instance fixture loads no full corpus/index replica onto
    owners and contains no delete/tombstone pruning step.
11. Existing replicated-control fault/identity evidence stays labeled as a
    control; no old row is promoted to a physical gate result.
12. Distributed-control DML fails closed until Task 167's physical adaptation;
    legacy single-node DML remains regression-clean.
13. Task 172 produced and packet 059's outside reviewer accepted 10k/50k/100k
    A/B recall, latency, and storage evidence before Task 179 was marked
    complete.

## Non-goals

- BatANN stack/direct coordination (Tasks 174–178).
- Physical incremental insert/update/delete implementation (Task 167 reopen).
- Cloud deployment or WAN tuning.
- Locality-aware graph partitioning; placement remains FR-078 hash ownership.
- Premature custom-page optimization of generation graph/directory storage.
  First prove transactional correctness and measure ordinary heap/B-tree cost.

## Cross-branch reconciliation

Task 173's current BatANN draft was authored against the prototype 16-byte
fingerprint and treats the existing replicated fixture as its multinode base.
Before BatANN Task 175 starts, rebase that batch onto this spec revision and:

- change relay-state fingerprint width/validation to FR-082's 34-byte v2 form;
- make physical Task 179 an explicit prerequisite for B1/B3/B4 multinode
  evidence;
- keep BatANN B0's local state-seam refactor independent where possible; and
- rerun its spec review/matrix validation for FR-078/FR-082 interface drift.

## References

- Specs: FR-075..FR-083; NFR-014, NFR-016..NFR-020; TC-040/042/044/050.
- ADR-085 D1/D6/D8/D10/D11.
- Task 163 D8 closeout: `reviews/task-163/005-d8-scale-memory/`.
- Task 172 Task-179-specific acceptance: packets 002–003 under
  `reviews/task-172/`.
- Aggregate outside decision:
  `reviews/task-179/059-closeout/feedback/2026-07-13-01-reviewer.md`.
- Current implementation seams: `src/am/ec_distann/{ambuild,shard_build,page,
  tuple,reader,epoch,epoch_manifest,roster,expand,remote_endpoint,
  remote_transport,routine,custom_scan}.rs` and
  `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs`.
