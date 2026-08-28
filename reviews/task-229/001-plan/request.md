---
task: 229
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 04
---

# Task 229 covering payload sidecar — revised concrete plan

This revision requests the short seq-02 rereview against exact current main
`3419c9c758bea7d9940b27d9afbcf9e627e84879`. It supersedes request seq-02 and
seq-03. Seq-02 confirmed every seq-01 P1/P2 finding closed and left only two
local-read blockers; both are addressed below. Dispositions are
`artifacts/seq01-disposition.md` and `artifacts/seq02-disposition.md`.

No source, SQL, test, fixture, or benchmark result is under review. Source
grounding remains `artifacts/current-main-architecture.md`.

## 1. Opt-in cover contract

Add the build-time string reloption `covering_payload_attnums`. Absence means no
sidecar and preserves all current no-cover format bytes. The value is a
canonical comma-separated list of positive physical attnums, strictly
increasing, unique, and bounded to 16. The persisted generation descriptor,
not a later mutable reloption value, is authoritative for reads and DML.

Generation construction resolves every attnum against the frozen
`DistannRowSchemaDescriptor` and rejects:

- absent, dropped, or generated attributes;
- the indexed vector attribute;
- missing binary send/receive identity; and
- any type outside this closed PG18 fixed-width `pg_catalog` set: `bool`,
  `int2`, `int4`, `int8`, `float4`, `float8`, `uuid`, `date`, `time`,
  `timestamp`, and `timestamptz`.

The allowlist is intentionally PG18 binary-I/O-stable and excludes variable-
width, array, domain, and user-defined values. Sixteen attributes times the
widest allowed 16-byte scalar is 256 value bytes; with a two-byte null bitmap,
the canonical entry payload is bounded to 258 bytes, below the TOAST threshold.
Queries requiring an unsupported or uncovered attribute use the row tier.

`DistannPayloadCoverDescriptorV1` records:

- entry format version 1 and the maximum attribute count;
- exact sorted attnums and their fixed binary widths;
- complete row-schema fingerprint;
- for each covered attribute, frozen attnum, type namespace/name, typmod,
  collation identity, and binary send/receive identity; and
- a domain-separated digest of those canonical descriptor bytes.

## 2. One row-version-exact representation

Create one owner-local ordinary PostgreSQL heap per covered generation:

`_ecdz_cover_<index_oid>_<build_uuid>(row_tid tid NOT NULL, vec_id bigint NOT
NULL, payload bytea NOT NULL)`

and one unique, non-covering B-tree on `row_tid`, named by the existing
deterministic OID/build-id helper convention. Both names must stay within
`NAMEDATALEN`; no hand-written suffix rule is allowed. The heap uses
`fillfactor=100`, and `payload` is `STORAGE PLAIN`.

This task deliberately chooses a non-covering unique B-tree rather than
`INCLUDE (payload)`. An INCLUDE index duplicates every payload and only becomes
index-only when heap visibility-map state permits it; recent Task 167 DML would
still require heap visibility checks. The chosen compact heap gives stable MVCC
semantics and minimum storage without depending on VACUUM state. This is the
only representation built or measured in Task 229.

Initial handoff creates exactly one sidecar row per owned vec_id. Later
same-identity replacements append one sidecar row per new row-tier version,
keyed by that version's frozen `row_tid`; superseded entries follow the row
tier's append-only retention rule and disappear only at generation reclaim.
`vec_id` is retained as a non-key identity echo and digest-order field.

Remote reads resolve one bounded batch with ordered
`unnest(row_tids, vec_ids) WITH ORDINALITY` against the single unique index.
For local physical hits, eligible `Frozen(row_tid)` outputs become
`FrozenPayloadPending { vec_id, row_tid }` at ranking time, when both identities
are available. `materialize_pending_physical_window` is generalized to gather
all local and remote pending rows in the demanded lazy-10 window; eager mode
uses one batch over the proven set. It executes at most one local SPI
`unnest(row_tids, vec_ids) WITH ORDINALITY` lookup per snapshot attempt and
window, never SPI per row. The pending variant makes both returned `row_tid`
and `vec_id` echoes checkable before conversion to a virtual local payload row.
There is no query per row, second sidecar variant, coordinator copy, or O(N)
state.

### Compact canonical entry

The relation's cover descriptor supplies version, column count, order, and
fixed widths. `payload` therefore contains only:

`null_bitmap | concatenated_non_null_fixed_width_binary_values`.

NULL is one bit (`1 = NULL`) and consumes no value bytes. Decode walks the
known widths and bitmap and requires the exact derived length; trailing,
truncated, or impossible payloads fail. The returned SQL row must echo the
requested `row_tid` and `vec_id`. No per-entry cover digest, column count,
offset array, or 32-byte checksum is duplicated into every row.

Corruption detection is layered: strict key/vec_id/length decoding, the frozen
cover descriptor, relation/page checksums, and the initial whole-sidecar digest
bound into receipt/manifest. Packet 002 will corrupt keys, vec_id echoes,
length/null shapes, descriptor identity, relation identity, and initial digest.
It will not claim detection of an arbitrary same-length value bit flip beyond
PostgreSQL page checksums, which is the same boundary as the row tier.

The owner initial-content digest folds `(vec_id, row_tid, payload)` in ascending
`(vec_id, row_tid)` order under a sidecar-specific domain. Handoff uses the
already-canonical row values directly. DML encodes the same positions from the
prepared row slot with the frozen send identities.

## 3. Typed, complete, visibility-equivalent selection

Task 222's `PayloadAttributeMask` remains the only executor authority. Thread
an explicit typed eligibility value, not merely the expanded attnum list,
through local and remote physical materialization:

- `Exact(attnums)` is eligible only when every requested attnum—including every
  qual-required attnum—is covered and the cover descriptor exactly matches the
  retained generation row schema;
- `AllColumns(reason)` is categorically ineligible even if its expanded live-
  attnum list happens to be a cover subset;
- an uncovered attnum, uncovered qual-required attnum, unsupported type,
  absent/legacy cover, or cover/schema disagreement selects the complete
  existing row-tier path; and
- selection is per request and all-or-nothing. No row is reconstructed from a
  mix of sidecar and row tier.

The owner repeats the descriptor and subset validation. Structural failures
are common to both owner classes:

- ERROR when the descriptor declares a cover but either sidecar relation is
  absent; cover/schema mismatch after catalog resolution; returned key or
  vec_id echo mismatch; malformed length/null shape; a visible row-tier tuple
  with no matching visible sidecar tuple; or any decode failure.

Remote visibility retains the existing remote-only behavior. A sidecar miss
probes that exact row-tier TID under the remote request snapshot: row tier also
not visible returns the existing `tuple_payload_missing -> RemoteSkipped`
marker; row tier visible is corruption. No local row becomes skippable through
this rule.

Local `FrozenPayloadPending` visibility matches the current `Frozen` control
attempt-for-attempt. The first batched sidecar lookup runs under
`estate.es_snapshot`. For initial misses, the same exact row-tier TIDs are
probed under `es_snapshot`: a visible row tier with no sidecar is corruption.
Rows invisible in both stores retry together in one batch under the exact
snapshot installed by `ActiveSnapshotGuard::latest()`—the active-SPI form of
the same `GetLatestSnapshot` retry the current direct fetch performs with
`RegisteredSnapshotGuard::latest()`. A second sidecar miss probes the exact row-
tier TID under that latest guard. Visible row tier is corruption; both still
invisible raise the existing error verbatim:
`EC_GENERATION_MISSING: published row-tier tuple ({},{}) disappeared`. They do
not skip. Successful pending rows become virtual local payload rows; the legacy
`CustomScanOutputRow::Local(tid)` path addresses the user's heap and is
untouched. A frozen row-tier TID was never a coordinator user-table ctid, so
converting only physical `Frozen` rows adds no ctid/EPQ/`FOR UPDATE` capability;
`custom_scan_recheck` retains its existing unconditional virtual-row contract.

## 4. Backward-compatible identity and lifecycle

### Canonical versions

- No-cover descriptors continue to encode as V2. Covered descriptors encode as
  V3. Decode accepts both and re-encodes a decoded legacy V2 byte-for-byte.
- No-cover Ready receipts remain V1/303 bytes. Covered receipts use V2 and add
  sidecar row count, explicitly named `initial_content_digest`, heap bytes, and
  index bytes. Receipt storage/framing becomes bounded variable-length and
  accepts both versions.
- No-cover epoch manifests remain V2. Covered manifests use V3 and add the
  cover-descriptor digest plus roster-ordered global initial-content digest.
  Fingerprints accept both `02 00 + digest` and `03 00 + digest`; either version
  may name the other as parent.
- Existing digest domain strings remain unchanged. Version is inside canonical
  bytes; no V2 descriptor, V1 receipt, V2 manifest, or their digest is reprinted.
- Packet 002 carries frozen legacy byte fixtures proving decode, byte-identical
  re-encode, and digest identity for descriptor V2, receipt V1, manifest V2,
  fingerprint V2, and Ready-receipt-set framing.

The project is in research-stage bootstrap posture: catalog SQL changes require
re-bootstrap rather than an extension upgrade script. Every control and
candidate matrix arm will use the post-change bootstrap and the same extension
binary; no pre-change database or SHA is a control.

### Catalog and relation ownership

`ec_distann_generation` gains nullable paired
`payload_sidecar_relid`/`payload_sidecar_directory_relid` with paired-state and
unique-non-null checks. Existing rows decode both as absent and use the row
tier. The implementation checklist explicitly covers every fixed-three-OID
consumer:

- generation cache invalidation;
- replay/existence validation;
- abort and reclaim drop ordering;
- control REINDEX cleanup;
- generation relation enumeration; and
- bootstrap uniqueness constraints.

It also covers every fixed 303-byte receipt consumer: generation catalog row
and transition parameter, handoff Ready construction, lifecycle-wire
Ready-receipt-set framing, AM/module/lib exports, SQL receipt and candidate-set
checks, and physical lifecycle fixtures.

Sidecar heap/index creation shares the row/graph generation transaction,
owner, permanence, explicit tablespace, deterministic name, and internal
control-index dependency. Ready locks and scans all five physical relations,
recomputes sidecar count/digest, requires initial sidecar count equal to owned
record count, and records heap/index sizes. Publication, restart, retained
predecessor reads, rollback, owner outage, abort, retirement reclaim, cancelled
reclaim, index/extension drop, and REINDEX all use the cataloged pair.

### Task 167 DML

Insert appends the row-tier tuple, then its TID-keyed sidecar tuple, then the
graph record in the same owner transaction before graph publication.
Same-identity replacement appends a new row-tier tuple and matching new sidecar
tuple, then switches the graph's current version; it never updates the old
sidecar row. Any later graph/backlink/fault failure aborts both payload writes.
Remote inserts reuse the existing Task 167 transaction/intent boundary and add
no round trip or independent commit.

Delete continues to flip only the graph tombstone. Historical sidecar versions
are retained exactly because their row-tier versions are retained; no new
sidecar-specific tombstone rule exists. A later valid replacement appends a new
TID-keyed entry.

Receipt/manifest/fingerprint bind only the immutable initial build content, as
existing graph/row-tier digests do. Post-Ready DML does not rewrite the
fingerprint. Packet 003 explicitly proves that post-Ready insert/replacement/
delete does not invalidate publication, restart, or retained-predecessor reads.

## 5. Review checkpoints

### Packet 002 — format and lifecycle

1. Parse the reloption and resolve/validate the exact fixed-width cover.
2. Implement the compact V1 cover/entry codec, exact length/null rules,
   TID+vec_id echo checks, 258-byte bound, corruption tests, `STORAGE PLAIN`,
   fillfactor 100, and the documented non-covering B-tree choice.
3. Implement descriptor V2/V3, receipt V1/V2, manifest V2/V3, fingerprint and
   lifecycle-wire receipt-set dual decode under unchanged domains; update all
   fixed-width Rust/SQL consumers; prove legacy byte/digest fixtures.
4. Add nullable sidecar OIDs and update all six fixed-three-relation surfaces;
   create/build/lock/digest/size/replay/cache-invalidate/abort/reclaim/restart/
   REINDEX/drop behavior and topology output.
5. Extend benchmark-only physical response telemetry—without changing the
   production semantic payload fields—with owner sidecar selected/fallback
   reason, lookup time, requested/returned/missing rows, payload bytes, and
   row-tier visibility probes. Record local batched SPI initial/retry work and
   remote owner lookup work as separate stages, including local batch count and
   rows per batch, so per-row SPI cannot hide in the primary endpoint. The
   coordinator aggregates remote response fields into stage/work counters; the
   fixture also records per-node topology sizes.

### Packet 003 — correctness and DML

1. Thread Task 222's typed exact eligibility and owner-side revalidation.
2. Exercise id-only, covered multi-scalar, covered NULL, and covered qual-only
   paths on local, remote, and mixed owners with byte-identical control output.
3. Exercise uncovered scalar/qual, `SELECT *`, whole-row, unsupported variable-
   width and external TOAST, legacy no-cover, and cover/schema disagreement
   fallback without partial reconstruction.
4. Exercise deepening, rescan, local Frozen conversion, remote payload reuse,
   current visibility skip, missing-visible corruption error, malformed bytes,
   restart, retained predecessor, reclaim, and owner outage.
5. Exercise insert, same-identity replacement with distinct old/new TIDs,
   delete/tombstone, fault rollback, routed DML, and post-Ready DML publication/
   restart/retained-read stability.
6. EXPLAIN and counters distinguish exact selection, fallback reason, sidecar
   rows/lookups/bytes, row-tier reads/probes, and local/remote owner work.

Focused callback/lifecycle validation uses PG18 pgrx tests; static codec work
uses focused Rust tests. PG17 is not planned absent a PG17-specific defect.

### Runner prerequisite before packet 004

The current suite cannot declare an ec_distann cover. Before any matrix, land
and review a separate runner commit adding:

- `covering_payload_attnums` to `distann-local-multinode` and its fixture DDL;
- a feature-gated Userset `benchmark_covering_sidecar` variant field, default
  on, where off forces the row-tier read against the same covered generation;
- isolated-pair validation that every other runtime axis matches;
- counterbalanced fresh-build position metadata; and
- sidecar topology, build, DML, and owner telemetry collection.

No packet-local shell runner is permitted.

## 6. Full-scale preregistration and decision rule

Packet 004 uses a bespoke checked-in `ecaz bench suite` config because current
canonical lane configs are single-node while Task 229 requires a three-owner
format A/B. Its manifest records that reason, exact release profile/feature
set, `allow_debug_extension=false`, one-index-per-table isolation for every
build arm, build/fixture/position identity for every result, and direct
`results.jsonl` provenance for every cited number.

At each of 10k, 50k, and 100k use the standard real corpus, three owners,
production lazy-10, Task 222 projection, identical RaBitQ/search/head/build
settings, post-change bootstrap, and one release binary SHA.

### Primary read-path A/B

Build covered generations with `covering_payload_attnums='1'` (`id bigint`).
Within each covered generation compare, through the feature-gated Userset arm:

- control: `benchmark_covering_sidecar=false`, forced row-tier read;
- candidate: `benchmark_covering_sidecar=true`, sidecar read.

Run this isolated pair on both covered builds produced by the build/storage
counterbalance below. Variant order is row-tier/sidecar on the first covered
build and sidecar/row-tier on the second. Both arms use the same extension
binary and exact generation; recall, result digest, and storage are therefore
same-generation controls for the read mechanism.

The primary endpoint is 100k warm mean end-to-end latency. **PROMOTE requires
both independent 100k covered-generation pairs to improve by at least 5.0% and
at least 0.50 ms.** Opposite signs, either pair below either magnitude, or an
envelope crossing zero is STOP; there is no favorable averaging tie-break.

Additional gating rules:

- recall and ordered prediction digest must be identical in every read pair;
- every 10k/50k pair must avoid warm-mean regression greater than 2.0%; a 100k
  win does not override a smaller-scale breach;
- p95 and p99 may not regress more than 5.0% at any scale/pair;
- candidate sidecar heap+index bytes must be at most 5.0% of the no-cover
  generation's total per-node physical bytes at every scale and all NFR-021
  amplification/placement constraints must still pass;
- covered build time may not regress more than 10.0% in either matched
  counterbalanced pair;
- at 100k, covered single-row insert throughput may not fall more than 10.0%,
  replacement/delete p95 may not regress more than 15.0%, and the sidecar may
  add no remote round trip; and
- any semantic, lifecycle, provenance, or measurement-integrity failure is not
  a performance result and must be corrected before the single authorized
  decision run is interpreted.

If 10k/100k directions disagree but the smaller scale stays inside its 2.0%
non-regression band, the 100k primary rule governs. A smaller-scale breach,
replicate sign disagreement, or any cost ceiling breach is STOP. Passing every
gate is PROMOTE to a separate production-default disposition; it never skips
Tasks 230--232.

### Build, storage, and DML A/B

At each scale run two fresh-build pairs in order
`no-cover -> cover` and `cover -> no-cover`. No arm reuses a fixture while its
mate is fresh, and no build-time arms share a corpus table or index. Compare
matched positions and report both deltas plus the envelope. The no-cover and
covered builds must match source count/digest, graph digest, row-tier digest,
search settings, and release provenance. They are separate-generation format
comparisons and are never labelled same-generation.

### Reported non-gating endpoints

Report p50/max, owner open/validation/lookup/row-tier/sidecar stages, local and
remote split, heap/sidecar reads, bytes by attribute, wire bytes, per-node
graph/row/sidecar/index/control storage, build phases, detailed DML work,
NFR-021 conformance, and NFR-022 admissibility. These explain the decision but
do not override the explicit gates above.

## Rereview request

Please confirm only that seq-02 B1/B2 are closed without reopening its accepted
format, descriptor, catalog, lifecycle, DML, threshold, or evidence rulings,
and authorize packet 002 implementation only if this revised plan is DONE.
