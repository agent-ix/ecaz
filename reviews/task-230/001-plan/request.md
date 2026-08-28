---
task: 230
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-28
seq: 02
---

# Task 230 hot/cold vertical row tier — concrete implementation plan

This revision requests outside review against exact current main
`23fb9b7ba1f0803be5dfc700d9865f80fbf60862`. It supersedes the planning-only
seq-01 request, which restated the task but did not freeze a format contract.
Task 229 is review-closed STOP and its sidecar is absent from both Task 230
arms. Source grounding is `artifacts/current-main-architecture.md`.

No source, SQL, test, fixture, or benchmark result is under review. Per Task
230 entry condition 3, persisted-format implementation remains gated on an
outside verdict for this packet.

## 1. Opt-in layout and authoritative attribute partition

Add two build-time reloptions:

- `row_tier_layout = 'row_heap' | 'hot_cold'`, defaulting to `row_heap`; and
- `hot_payload_attnums`, a canonical strictly increasing list of at most 16
  positive physical attnums, legal only with `distributed_control=true` and
  `row_tier_layout='hot_cold'`.

The persisted generation descriptor, never a later reloption read, is the
authority. The descriptor freezes the complete source row schema, indexed
vector attnum, source-identity attnum, and hot scalar attnums. The indexed
vector and source identity are mandatory hot attributes; the vector is
implicit and must not appear in `hot_payload_attnums`, while the source
identity must appear explicitly so the requested layout is visible in DDL and
suite configuration.

Hot scalars use the same deliberately closed PG18 fixed-width type contract as
Task 229: `bool`, `int2`, `int4`, `int8`, `float4`, `float8`, `uuid`, `date`,
`time`, `timestamp`, and `timestamptz`. Generated, dropped, collatable,
variable-width, domain, array, and user-defined attributes are rejected as hot
scalars. Sixteen 16-byte values plus a two-byte NULL bitmap gives a hard
258-byte scalar-value bound. The full-precision indexed vector is separately
bounded by the descriptor dimension and is not counted in that cover bound.

Every non-dropped source attnum is assigned exactly once:

- hot: indexed vector, source identity, and selected hot scalars;
- cold: every other source attribute.

The descriptor validates a disjoint union equal to the complete set of live
source attnums. There is no full-row fallback copy, second exact vector, or
Task 229 payload sidecar in a hot/cold generation. Dropped attnums remain in
the frozen logical schema but have no stored value in either tier.

## 2. Physical relations and versioned locator

The existing catalog `row_tier_relid` denotes the primary/hot relation for a
hot/cold generation; a new nullable `cold_tier_relid` denotes its cold
counterpart. Legacy row-heap generations keep `cold_tier_relid IS NULL`.
Catalog checks require exactly one of these shapes:

1. row heap: row tier present, cold tier absent, Task 229 sidecar pair optional;
2. hot/cold: primary hot and cold tiers present, sidecar pair absent.

The two new heaps use compact physical columns rather than original-attnum
holes. Source fields are named by generated attnum-based identifiers and
ordered by ascending source attnum. The descriptor maps each logical attnum to
its tier and physical ordinal, so user column names are never interpreted as
internal identifiers.

The hot heap contains:

`vec_id bigint | tombstone boolean | exact vector | selected scalar values`

The cold heap contains:

`vec_id bigint | every remaining source value`

Both `vec_id` fields are internal identity echoes, not duplicate source
attributes. The hot tuple's MVCC header is the visibility authority; its
`tombstone` is updated with the graph tombstone in one transaction. The cold
echo makes a wrong-but-visible locator fail closed during reconstruction.

Graph record V1 continues to mean `row_tid -> complete legacy row`. Hot/cold
generations use graph record V2 with a versioned
`DistannRowLocatorV2 { hot_tid, cold_tid }`; its fixed header grows by one
six-byte TID. The graph heap's existing `row_tid` column remains a hot-TID echo
for lookup/integrity checks, while the canonical graph-record bytes bind both
locators. The generation descriptor binds graph-record version and row-layout
descriptor, and validation rejects V1/hot-cold or V2/row-heap combinations.
Expansion responses that already carry an owner locator carry the complete V2
pair, still as opaque owner-local bytes. Only the retained owner named by the
epoch roster may interpret either TID. No source-table or cross-node CTID is
inferred.

Creation, ownership, persistence, tablespace, internal dependency, abort,
REINDEX cleanup, retirement, and reclaim cover hot and cold relations in the
same generation transaction. Older row-heap generations retain their current
read and reclaim paths unchanged.

## 3. Canonical format and lifecycle identity

Add `DistannRowTierLayoutDescriptorV1`, domain-separated and bound into a new
generation-descriptor version. Legacy no-cover V2 and Task 229 cover V3 bytes
remain byte-identical; hot/cold uses V4 and is mutually exclusive with the
payload-cover descriptor. The layout descriptor records:

- layout/entry version and full row-schema fingerprint;
- vector and identity attnums;
- ordered hot scalar descriptors with type/binary-I/O identity and width;
- the complete canonical hot/cold attnum partition; and
- physical hot/cold ordinal mappings and maximum scalar bytes.

Handoff wire V1 remains unchanged: it already carries every source attribute
as canonical binary values. The participant partitions each entry only after
validating the descriptor, inserts cold first, records its TID in hot, then
records the hot TID in graph. A failure at any point aborts all three writes.

Ready receipt V3 records one logical row count plus separate initial hot and
cold content digests and heap bytes. Graph and directory fields remain
explicit. The hot digest folds, in ascending vec_id order, vec_id, tombstone,
exact vector bytes, NULL shape, and hot scalar bytes. The cold digest folds
vec_id, NULL shape, and cold attribute bytes; the graph digest binds both
locators. Receipt validation requires equal graph/hot/cold counts and matching
vec_id/locator echoes.

Epoch manifest V4 records the layout-descriptor digest and roster-ordered
global hot and cold initial-content digests. The existing logical global row
digest remains the source/handoff completeness identity; it is not relabeled
as either physical tier. Fingerprint decoding accepts legacy V2, Task 229 V3,
and hot/cold V4. Build candidate, publication/recovery, topology, inspection,
and storage reports verify all descriptor/receipt/manifest links before a
generation becomes readable.

## 4. Read and reconstruction contract

Expansion, exact scoring, head/traversal-replica construction, and exact rerank
resolve only the V2 hot locator and deform only the compact hot tuple. They
validate vec_id, tombstone parity, exact-vector dimension/finiteness, and the
expected hot relation identity. No cold relation is opened by an exact-vector
read. Feature-gated counters separately report hot tuple/block reads, exact
vector detoast/send work, and any cold relation opens/reads; the acceptance
invariant is zero cold work for expansion/rerank-only calls.

Task 222's typed `PayloadAttributeMask` stays authoritative. The graph lookup
resolves the V2 locator pair before payload access; materialization partitions
an exact requested attnum set by the frozen layout:

- hot-only requests batch-fetch hot rows and never open cold;
- cold-only requests batch-fetch cold and never open hot on the success path;
- mixed requests batch-fetch both and merge by requested vec_id/ordinality;
- `AllColumns(reason)` requests both tiers; and
- exact-vector projection reads its sole hot value and does not duplicate it
  from cold.

Both batches preserve request order and echo vec_id plus requested/returned
TIDs. Reconstruction validates each echo, matching MVCC visibility behavior,
and one returned value for every requested live logical attnum. It feeds the
existing binary receive metadata in original attnum order, including NULLs and
dropped-column holes, so the virtual scan tuple has the exact source row
descriptor. A missing required tier follows the existing one-refresh retry
contract. On a miss only, the owner probes the counterpart at the V2 locator
under the same snapshot to distinguish an atomically invisible row pair from a
half-missing corrupt pair. A visible counterpart with a missing/mismatched
required tuple is structural corruption, never a partial row or silent skip. A
row absent in both tiers under the same remote request snapshot retains the
existing remote-skipped semantics.

Legacy descriptor V2/V3 generations dispatch to the current row-heap (and, for
V3, sidecar-aware) reader. Task 230 never constructs a V3 sidecar candidate.

## 5. DML and lifecycle mutations

Task 167 insert and replacement prepare one full logical row, partition it by
the retained descriptor, insert cold then hot then graph, and publish the graph
version only after every tuple exists in the same owner transaction. A
same-identity replacement appends new cold/hot versions and a new graph
version; predecessor tuples remain for snapshot-pinned readers until generation
reclaim.

Delete updates the current graph tombstone and its referenced hot tombstone in
one transaction. It never rewrites or fetches cold payload. Retry/intent,
forwarded mutation, fault, and recovery paths validate that both mutations
target the same build, vec_id, record version, and hot locator. Post-Ready DML
does not rewrite immutable initial digests or the epoch fingerprint, matching
the existing graph/row-tier lifecycle rule.

Ready, publication, restart, retained-predecessor reads, rollback, cancellation,
retirement, forced/ordinary reclaim, owner outage, index drop, extension drop,
and REINDEX must handle the relation pair as one fail-closed unit. An absent
half, unexpected sidecar, descriptor/catalog mismatch, digest mismatch, or
locator echo mismatch makes the generation unreadable and unpublishable.

## 6. Storage expectation and measurement gate

The standard staged corpus is 1,536 dimensions and the physical fixture schema
is `(id bigint, source_id uuid, source real[], embedding ecvector[, payload_note
text])`. The preregistered Task 230 candidate uses hot scalars `1,2`: hot owns
`id`, `source_id`, and `embedding`; cold owns `source` and optional
`payload_note`. No source value is duplicated.

Before page/TOAST effects, the candidate adds one heap-tuple header per logical
row plus two internal vec_id echoes, one extra six-byte graph locator, one hot
tombstone byte, alignment, and relation-page overhead. A conservative
fixed-overhead planning bound is 96 bytes per row before cold TOAST changes.
Against two 1,536-float source/vector values this predicts low-single-digit
amplification, but this is explicitly a hypothesis, not a result. Packet 004
reports hot heap/TOAST, cold heap/TOAST, graph, directory, and total generation
bytes separately and uses measured values for the decision.

The primary A/B is fresh row-heap versus fresh hot/cold at 10k/50k/100k, two
counterbalanced pairs per scale, driven only by a checked-in `ecaz bench suite`
config. Both arms use the same final extension SHA, corpus, query set, graph/
quantizer/search settings, correctness payload, and matched fixture position.
Task 229 sidecar and Task 231 blocks are disabled.

The suite records recall, predictions, warm/cold latency and tails, exact-vector
reads, hot/cold tuple and block reads, detoast/send bytes, materialization
stages, build time, insert/replacement/delete cost, per-tier storage, total
storage, and NFR-021 conformance. Promotion requires exact semantic parity,
zero cold reads on exact-vector-only paths, no recall regression, and a
repeatable end-to-end retrieval benefit that is not purchased by an
unacceptable storage/build/DML regression. Packet 004 preregisters exact
numeric thresholds before any full-scale result is read; absent a qualifying
candidate, Task 230 closes STOP and Tasks 231/232 still proceed.

## 7. Checkpoints requested after plan acceptance

### Packet 002 — format and read path

1. Reloptions and layout descriptor, canonical versions, corrupt/legacy frozen
   fixtures, and mutual-exclusion validation.
2. Catalog/bootstrap changes and transactional hot/cold relation creation,
   handoff partition/insertion, Ready digests/sizes, manifest/fingerprint, and
   old-generation replay.
3. V2 locator decode plus hot-only exact reads, typed hot/cold/mixed/all-column
   materialization, exact row reconstruction, visibility retry, and counters.
4. Focused Rust tests plus PG18 id-only, hot scalar, exact vector, cold scalar,
   mixed, `SELECT *`, NULL, external TOAST, qual-only, deepening, rescan,
   restart, and owner-failure coverage.

### Packet 003 — lifecycle and DML

1. Insert/replacement/delete atomicity and retry/intent/fault behavior.
2. Rebuild rollback, retained predecessor, publication recovery, retirement,
   reclaim, drop, and REINDEX coverage.
3. PG18 mutation/restart/retirement matrix with packet-local logs.

### Packet 004 — full-scale decision

Check in and audit the standard suite config, preregister thresholds, run the
counterbalanced 10k/50k/100k matrix, and request an outside PROMOTE or STOP
verdict. No Task 230 code or task status is closed before that verdict.

## Review request

Please review the attribute-partition/no-duplication invariant, V1/V2 locator
compatibility, compact relation shape, descriptor/receipt/manifest versioning,
visibility and reconstruction semantics, DML ordering, lifecycle coverage, and
the 96-byte-per-row planning bound. The specific authorization requested is to
begin packet 002 persisted-format implementation; findings should land under
this packet's `feedback/` directory.
