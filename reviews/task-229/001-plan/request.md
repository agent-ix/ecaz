---
task: 229
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 02
---

# Task 229 covering payload sidecar — concrete current-main plan

This refresh supersedes the planning-only placeholder written at checkpoint
`627477613`. It requests review against exact current main
`3419c9c758bea7d9940b27d9afbcf9e627e84879`, after Task 239 closed the bounded-
read semantic blocker and the campaign ledger landed through PR #89. No source
change, test result, benchmark result, or performance claim is under review.

The source-grounding record is
`artifacts/current-main-architecture.md`; artifact provenance is in
`artifacts/manifest.md`.

## Proposed contract

### 1. One opt-in format and one lookup representation

- Add the build-time index reloption `covering_payload_attnums`. Absence means
  no sidecar and preserves the current format byte-for-byte. The value is a
  canonical comma-separated list of positive physical attnums, strictly
  increasing, unique, and bounded to 16 attributes. Names are not persisted or
  re-resolved after build. The persisted generation descriptor, not a later
  mutable reloption value, is authoritative for reads and DML.
- At generation construction, resolve every declared attnum against the frozen
  `DistannRowSchemaDescriptor`. Reject dropped/generated columns, the indexed
  vector, absent binary send/receive identity, and types outside a deliberately
  small fixed-width built-in scalar set: `bool`, `int2`, `int4`, `int8`,
  `float4`, `float8`, `uuid`, `date`, `time`, `timestamp`, and `timestamptz` in
  `pg_catalog`. This keeps each entry bounded; variable-width values, arrays,
  user types, domains, and arbitrary payloads remain row-tier-only.
- Persist one owner-local ordinary PostgreSQL heap relation per covered
  generation:
  `_ecdz_cover_<index_oid>_<build_uuid>(vec_id bigint NOT NULL, payload bytea
  NOT NULL)`, plus one unique B-tree on `vec_id`. No second representation,
  coordinator copy, cache, or per-attribute side table is introduced.
- Resolve a batch with one ordered `unnest(vec_ids) WITH ORDINALITY` join to the
  sidecar and its single unique index. Lookup count is bounded by the already-
  bounded materialization window; there is no query per id and no O(N)
  coordinator state.

### 2. Canonical cover and entry bytes

Introduce `DistannPayloadCoverDescriptorV1` containing:

- sidecar entry version and the fixed maximum attribute count;
- exact sorted covered attnums;
- the complete row-schema fingerprint;
- for each covered attribute, its frozen attnum, type namespace/name, typmod,
  collation identity, and binary send/receive identity copied from the row
  schema; and
- a domain-separated digest over those canonical cover bytes.

Each `payload` is independently decodable canonical V1 bytes:

`version | vec_id | cover_descriptor_digest | column_count | null_bitmap |
end_offsets | concatenated_binary_values | entry_digest`.

The null bitmap uses one bit per covered position (`1 = NULL`); NULL consumes no
value bytes and adjacent offsets remain equal. Offsets are bounded `u32`s.
The record repeats `vec_id` and the cover digest even though the heap key owns
them so a wrong-key, wrong-generation, truncated, reordered, or bit-corrupt
entry cannot decode successfully. The content digest for one owner is the
domain-separated digest of complete canonical entries in ascending `vec_id`
order.

Handoff already carries canonical binary values for the entire frozen row.
Sidecar build therefore selects the declared positions directly from
`DistannHandoffEntry` rather than receiving and re-sending the row. DML paths
encode the same positions from the already-prepared row slot with the frozen
send identities.

### 3. Typed, complete, fail-closed selection

Task 222's `PayloadAttributeMask` remains the only planner/executor authority.
Thread an explicit eligibility value alongside the requested attnums through
the local and remote physical materialization APIs:

- `Exact(attnums)` is eligible only if every attnum is present in the cover and
  the cover descriptor exactly matches the retained generation's row schema;
- `AllColumns(reason)` is never eligible, even if its expanded live-attnum list
  happens to be a subset of the cover;
- an uncovered attnum, qual-required attnum, unsupported type, absent cover,
  cover/schema disagreement, or legacy descriptor selects the existing whole
  row-tier path; and
- selection is all-or-nothing for a requested row. No sidecar/row-tier partial
  reconstruction is allowed.

This preserves whole-row, `SELECT *`, unproved-expression, and visibility
fallbacks. A covered qual-only column is eligible because Task 222 includes it
in the exact mask; an uncovered qual forces the complete row-tier path.

The owner repeats the descriptor/subset check. A missing sidecar relation,
missing covered entry for a live graph record, wrong key/digest, malformed
offset/null shape, or corrupt bytes is structural corruption and errors
fail-closed; it does not silently fall back. Ordinary unsupported query shapes
fall back before lookup.

The optimization applies to both owner classes. Remote hits keep the current
wire response shape after an owner-side sidecar lookup. Covered local hits are
also reconstructed as virtual projected tuples from the local owner's sidecar
instead of opening the frozen row tier. Fallback local hits retain the existing
frozen-row TID path.

## Backward-compatible identity and lifecycle

### Format evolution

- A descriptor with no cover continues to encode as generation descriptor V2
  under the existing V2 domain. A descriptor with a cover encodes as V3.
  Decode accepts V2 as `cover = None` and V3 as `cover = Some(...)`; digesting
  decoded V2 bytes reproduces their existing digest.
- Ready receipts remain V1 when no sidecar exists. Sidecar generations use V2,
  adding sidecar row count, initial content digest, heap bytes, and index bytes.
  Receipt storage becomes variable-length and decodes both versions.
- Epoch manifests remain V2 for no-sidecar builds. Sidecar builds use V3 and
  add the cover-descriptor digest and roster-ordered global initial sidecar
  digest. Fingerprint decoding accepts both the existing `02 00 + digest` and
  new `03 00 + digest` forms. A V3 successor may name a V2 parent and vice
  versa.
- Catalog rows gain nullable, paired `payload_sidecar_relid` and
  `payload_sidecar_directory_relid` columns. Existing rows decode as absent and
  use the row tier. SQL receipt/fingerprint checks accept both canonical
  versions and reject every other shape.

The initial immutable sidecar content is bound by descriptor -> owner receipt
-> epoch manifest -> manifest digest/fingerprint, matching how existing build-
time graph and row-tier digests remain the immutable publication identity even
though Task 167 later adds mutable generation-local DML state.

### Physical lifecycle

- Create the sidecar heap/index in the same transaction as the generation row
  and graph relations, with the same owner, persistence, tablespace, deterministic
  naming, and internal dependency on the control index.
- Build entries in the same handoff transaction as row-tier and graph records.
  Ready recomputes count/digest from physical storage, records exact heap/index
  sizes, and refuses count/key/digest disagreement.
- Publication, restart, retained predecessor reads, rollback, and outage use
  cataloged OIDs plus the descriptor/manifest identity. Abort, retirement
  reclaim, cancelled reclaim, control REINDEX, and extension/index drop remove
  both sidecar relations with their generation.
- Insert writes the row tier, sidecar entry, and graph record in one owner
  transaction before graph publication. Same-identity replacement upserts the
  sole sidecar row before switching the graph's current version. Any later
  failure aborts all three writes. Remote inserts use the existing Task 167
  transaction/intent boundary; no independent sidecar commit is added.
- Delete keeps the existing graph tombstone rule. The one sidecar row is
  retained but unreachable while the current graph record is tombstoned, and
  is reclaimed only with the generation. A later valid same-identity
  replacement overwrites it transactionally.

## Implementation checkpoints

Packet 002 (`format-and-lifecycle`) will contain:

1. reloption parsing and cover resolution;
2. canonical cover/entry V1 codecs and corruption/limit tests;
3. descriptor V2/V3, receipt V1/V2, manifest V2/V3, and fingerprint dual
   decoding with legacy byte fixtures;
4. nullable catalog OIDs plus create/build/digest/size/abort/reclaim/restart
   lifecycle; and
5. topology/storage counters for sidecar rows, heap bytes, index bytes, and
   digests.

Packet 003 (`correctness-and-dml`) will contain:

1. typed exact-mask propagation and local/remote owner selection;
2. id-only and covered multi-scalar success;
3. uncovered scalar, uncovered qual, `SELECT *`, whole-row, unsupported
   variable-width/TOAST, and schema-disagreement fallback;
4. covered NULL, mixed local/remote ownership, deepening, rescan, and byte-
   identity checks;
5. insert, same-identity replacement, delete/tombstone, injected rollback,
   restart, retained predecessor, reclaim, and owner outage cases; and
6. EXPLAIN/stage/work counters distinguishing sidecar selection, fallback
   reason, sidecar rows/lookups/bytes, and row-tier reads.

Tests that touch PostgreSQL callbacks/lifecycle will use focused PG18 pgrx
commands. Static codec tests will use focused `cargo test`. No PG17 run is
planned unless a PG17-specific issue appears.

## Preregistered full-scale decision design

Packet 004 will use only a checked-in `ecaz bench suite` config. If the suite's
`distann-local-multinode` step cannot declare the cover, counterbalance build
order, or report the required sidecar metrics, those narrow runner fields will
land and be reviewed before the matrix; no packet-local shell sweeper will be
created.

At each of 10k, 50k, and 100k:

- use the standard three-owner real corpus, production lazy-10, Task 222
  projection, identical RaBitQ/search/head/build settings, and one release SHA;
- compare no-cover control with `covering_payload_attnums='1'` (`id bigint`) so
  the standard id-only query exercises exactly the candidate;
- run two fresh-build pairs in counterbalanced order `control -> candidate`
  and `candidate -> control`; neither arm may be a reused fixture while the
  other is fresh;
- compare matched pair positions and report both pair deltas plus their
  counterbalanced envelope, never a pooled number that hides order/warmth;
- require source count/digest, graph digest, row-tier digest, predictions, and
  recall to match between arms. This is a format-changing separate-generation
  comparison, not a false same-generation claim; NFR-022 provenance will name
  the build and position for every cell; and
- report build time, DML work, mean/p50/p95/p99/max, recall/result digest,
  owner endpoint/lookup/row-tier/sidecar stages, heap and sidecar reads, bytes
  by attribute, wire bytes, per-node graph/row/sidecar/index/control storage,
  amplification, NFR-021 conformance, and NFR-022 admissibility.

The decision is `PROMOTE` or `STOP` only after all scales and both positions
pass semantic/provenance gates. Promotion authorizes a separate default-policy
decision; it does not skip Tasks 230--232.

## Review questions

Please rule specifically on:

1. numeric physical-attnum reloption, 16-attribute cap, and fixed-width
   built-in allowlist;
2. one heap plus one unique B-tree as the sole bounded representation;
3. preserving no-cover V2/V1/V2 bytes while introducing optional
   descriptor/receipt/manifest versions;
4. initial-build digest binding versus later Task 167 mutable DML state;
5. retaining tombstoned sidecar rows until generation reclaim;
6. treating corruption/missing covered rows as an error rather than fallback;
7. applying the sidecar to local as well as remote owner hits; and
8. the two-pair `AB/BA` full-scale envelope as sufficient position/warmth
   control.
