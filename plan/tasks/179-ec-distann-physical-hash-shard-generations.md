# Task 179: ec_distann Physical Hash-Shard Generations, Handoff, and Publish

Status: proposed (2026-07-10). Depends on: Task 163's open ADR-085 D8 /
FR-077-CON-4 streamed-stitch closeout and the revised FR-075..FR-083 /
NFR-014/NFR-016..020 contracts. Build from the current
`task-165-ec-distann-m3` implementation line after preserving its replicated
control evidence.

Task numbers 173–178 are already reserved by the parallel Task 173 BatANN spec
and B0–B4 implementation plan. Task 179 is therefore the first available task
number for this corrective physical-storage lane.

Owner: coder (to be assigned). One coder, one branch. Keep each checkpoint in
its owning task packet; the prerequisite D8 spill change is reviewed under
Task 163, not hidden inside this task.

Priority: P0. Blocks Task 172's real multinode benchmark gate. BatANN Task 174
(the local beam-state refactor) may proceed independently, but BatANN Tasks
175–178 must not claim multinode or benchmark completion until this task's
physical fixture is implementation-ready.

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
commit-only and recoverable, scans pin one manifest/fingerprint, and the
three-instance fixture proves exact/disjoint topology before Task 172 measures
anything.

The implementation-ready checkpoint unblocks Task 172. Task 179 itself remains
open until Task 172 supplies the required 10k/50k/100k A/B recall, latency, and
storage evidence and Task 179 cites that immutable packet for closeout.

## Normative authority

The specification owns behavior and wire/storage identities:

- FR-075: `distributed_control` initialization and scan boundary.
- FR-076: graph record and handoff entry/batch formats.
- FR-077: one-entry-per-vec_id stitched stream and D8 memory bound.
- FR-078: node registry, generation descriptor, physical placement, frozen row
  tier, handoff endpoints, transaction/replay rules, and topology endpoint.
- FR-079: expansion and attnum-based row materialization.
- FR-082: manifest v2, 34-byte fingerprint, decision/publish/recovery order,
  scan pins, retention, and active-pointer semantics.
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
| Epoch | One v4 metadata-page state | Multiple Building/Ready/Published/Retired generations plus Aborted/reclaimed state |
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

- Create one hidden row-tier heap relation by copying the participant's
  schema-compatible source-shell tuple descriptor. Insert decoded source
  datums directly so generated values are stored, not recomputed, and normal
  PostgreSQL TOAST handles wide values. The relation is immutable after Ready.
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
  TOAST separately for topology/storage results.

Adapt the proven `ec_spire/storage/relation_plan.rs`
`heap_create_with_catalog`/`DEPENDENCY_INTERNAL` helpers where their ownership
model matches. Do not share SPIRE epoch state or partition-object semantics.

Add extension-owned catalogs (exact SQL names may be adjusted together in one
reviewed schema checkpoint):

- `ec_distann_node_descriptor`
- `ec_distann_generation`
- `ec_distann_generation_batch`
- `ec_distann_publish_decision`
- `ec_distann_active_epoch`
- `ec_distann_scan_pin`

Every row is keyed by logical-index UUID in addition to the local index OID.
Catalog and endpoint privileges are revoked from `PUBLIC`. DROP/REINDEX tests
must prove hidden relations are dependency-cleaned and stale catalog rows are
never addressable after OID reuse.

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
- Under the source session lock, fetch the exact callback TID into a tuple slot,
  compare its indexed vector/source identity with the callback datums, and
  serialize every non-dropped attribute through locally resolved `typsend`.
  Add focused coverage for NULL, generated, toasted, dropped, unsupported, and
  recently-dead tuple cases.
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
- `abort`: idempotently drop only an unpublished generation and its batch/pin
  rows. It must refuse a generation named by a durable publish decision.
- An identical acknowledged replay returns the journaled receipt. A conflict,
  sequence error, malformed entry, wrong owner, duplicate, schema/codec error,
  or oversize input performs zero relation/catalog mutation.

Participant restart coverage must resend exactly the first unacknowledged
batch while the coordinator still owns the source lock and build workspace.
Coordinator loss before a durable decision follows this task's v1 choice:
abort the unpublished remote generation; do not attempt to reconstruct the old
MVCC snapshot.

### 6. Split Ready, decision, and recovery across real commit boundaries

The operator/CLI sequence uses one coordinator connection but three committed
transactions:

1. `ec_distann_build_epoch` captures/builds/hands off and returns a Ready
   manifest candidate. It holds a session-level source lock and persists the
   durable build gate before returning.
2. `ec_distann_decide_epoch_publish` re-reads owner topology/receipts and commits
   the immutable commit-only decision. It must not contact publish endpoints.
3. In a later transaction, `ec_distann_recover_epoch_publish` idempotently
   publishes participants, waits for matching acknowledgements, swaps the
   coordinator active pointer last, clears the build gate, and releases the
   session lock.

Extend the existing SPIRE DML/utility-hook pattern only far enough to enforce
the durable build gate after coordinator-session loss: source DML and
schema-changing DDL fail closed until explicit pre-decision abort or
post-decision recovery. Reads of the prior Published epoch continue.

Replace the current one-page `epoch_manifest.rs` state machine with catalog
generation state; keep the legacy local implementation behind
`distributed_control=false`. Exercise crashes before decision, after decision,
after each participant acknowledgement, before pointer swap, and after pointer
swap.

### 7. Pin physical generations for the whole scan attempt

- Add idempotent pin/unpin wrappers keyed by `(fingerprint, scan_token UUID)`.
- At CustomScan attempt start, pin every participant before expansion. On a
  partial pin failure, unpin the acquired subset and issue no expansion.
- Put unpin in a guard reached by normal completion, `EndCustomScan`, rescan,
  epoch-mismatch restart, remote error, statement timeout, and cancellation.
- Persist pins on participants so a participant restart cannot erase a live
  coordinator's retention claim. Duplicate pin/unpin is a no-op; token reuse
  across fingerprints is `EC_EPOCH_PIN_CONFLICT`.
- Normal retire requires zero token rows. Force-retire deletes wedged tokens
  only with the FR-082 audit record.

Suggested module: `generation_pin.rs`; keep this separate from beam counters.

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
- Load the head sample and codec artifact for the exact pinned manifest so
  retained old and new fingerprints can be read concurrently.

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

Use narrow code commits followed by separate request commits:

1. Task 163 next packet — D8 `BufFile` shard-output spill and bounded-cursor
   proof (prerequisite; not a Task 179 packet).
2. `reviews/task-179/001-format-and-control/` — reloption/control metadata,
   wire/descriptor/manifest/fingerprint codecs, TC-050 fixtures.
3. `reviews/task-179/002-generation-storage/` — catalogs, hidden relation
   lifecycle, transactional batch model, DROP/OID-reuse tests.
4. `reviews/task-179/003-streamed-handoff/` — source-row capture, owner streams,
   begin/stage/seal/abort, replay/boundary/error tests.
5. `reviews/task-179/004-publication-and-pins/` — build gate, decision boundary,
   recovery, active pointer, pin/retire fault matrix.
6. `reviews/task-179/005-generation-read-path/` — expansion, frozen-row
   materialization, quals, retained old/new reads, legacy parity.
7. `reviews/task-179/006-physical-three-instance-fixture/` — suite-driven
   physical setup, topology results, TC-040/TC-042 implementation-ready verdict.
8. `reviews/task-179/007-closeout/` — reviewer response plus immutable Task 172
   10k/50k/100k A/B evidence citation; only this packet may mark Task 179 done.

Each measurement packet needs its suite config, manifest, results JSONL, and
only cited logs. Do not commit corpus TSVs, PostgreSQL operational logs, tunnel
state, polling exhaust, or packet-local sweep scripts.

## Verification plan

Run the narrowest PG18 checks appropriate to each risky checkpoint:

- pure Rust codec/digest/schema/placement tests;
- NFR-016 fixture, upgrade-matrix, endian, and layout checks;
- focused PG18 pg_tests for relation/catalog/WAL/rollback behavior;
- three-instance TC-040 handoff/materialization tests;
- three-instance TC-042 lifecycle/fault/pin tests;
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
7. Durable scan pins cover normal/error/cancel/restart paths and prevent normal
   reclaim; force-retire is explicit and fully audited.
8. Expansion and materialization read only the pinned physical generation;
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
13. Task 172 produces and an outside reviewer accepts 10k/50k/100k A/B recall,
    latency, and storage evidence before Task 179 is marked complete.

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
- Task 163 open feedback:
  `reviews/task-163/002-m1-closure-sweep/feedback/2026-07-09-03-reviewer.md`.
- Task 172 corrective review:
  `reviews/task-172/001-real-multinode-benchmark/feedback/2026-07-10-03-reviewer.md`.
- Current implementation seams: `src/am/ec_distann/{ambuild,shard_build,page,
  tuple,reader,epoch,epoch_manifest,roster,expand,remote_endpoint,
  remote_transport,routine,custom_scan}.rs` and
  `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs`.
