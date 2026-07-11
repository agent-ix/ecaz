# Task 179 packet 003 — transactional generation storage

**Review requested.** Please review implementation checkpoint
`531bbb22f4c009c7f49ced7340815b2649bcff88` on
`task-179-ec-distann-physical-shards`.

This checkpoint implements the first physical-generation storage boundary and
incorporates every finding from Claude's packet-001 seq-03 and packet-002
seq-01 reviews. The finding-by-finding coder responses are stored in those
packets; this request does not close either review topic.

## Implemented scope

- Adds the seven UUID-scoped Task 179 catalogs, revokes catalog and internal
  endpoint privileges from `PUBLIC`, and cleans catalog rows through a drop
  event trigger.
- Adds secured `control_identity`, exact/idempotent `begin_epoch_handoff`,
  unpublished-generation listing, and idempotent pre-decision abort surfaces.
- Creates one permanent WAL-logged row-tier heap, graph-store heap, and unique
  B-tree directory for each local `(logical_index_uuid, build_id)` generation.
  The row tier preserves physical attnums/dropped slots, stores generated
  columns as ordinary captured values, and uses normal PostgreSQL TOAST.
- Keys every catalog lookup by local index OID plus the persisted logical UUID,
  keeps relation OIDs local, assigns the control owner/schema/tablespace, and
  records exact internal dependencies on the logical control index.
- Makes begin replay return prior progress only for byte-exact immutable inputs;
  conflicting build-ID reuse errors without mutation. Relation DDL and catalog
  insertion roll back as one PostgreSQL transaction.
- Makes abort drop only unpublished physical relations, refuse a publish
  decision, and remain idempotent. DROP cascades dependents and event-cleans
  rows. Every REINDEX is an explicit destructive identity/state boundary;
  control REINDEX removes old generation state and writes a fresh UUID.
- Gates legacy lifecycle and local-data endpoints on control roots, treats
  control VACUUM as a no-op, and rejects concurrent control builds for both
  populated and empty sources.
- Freezes format-v1 validity bounds and seeded codec derivation semantics,
  pins code/score goldens, validates RFC-v4 build IDs through one predicate,
  canonicalizes dropped schema attributes and `+0.0`, hardens snapshot ranges,
  aliases the physical version constant, and cross-checks codec plaintext.
- Resolves packet-001's retirement-fence ambiguity in the normative spec/task:
  exactly one fence per logical UUID, committed-decision rejection, defined
  authoritative coordinator, and explicit contention/release semantics.

## Review focus

1. Transactionality and lock/dependency correctness of create, replay, abort,
   DROP, and REINDEX.
2. Dynamic SQL safety, schema fidelity, owner/tablespace behavior, and local-OID
   confinement in `generation_store.rs`.
3. Catalog identity isolation and cleanup ordering in
   `generation_catalog.rs`/`bootstrap.sql`.
4. Security-definer search paths and `PUBLIC` privilege revocation.
5. Claude finding parity in the two coder-response files.

## Validation

Packet-local artifacts are indexed by `artifacts/manifest.md`:

- DistANN unit + PG18: **147 passed, 0 failed, 1 ignored fixture emitter**.
- Golden/layout/upgrade: **65 + 13 + 2 passed, 0 failed**.
- Strict PG18 all-target clippy: exit 0.
- Quire: **244/244**, zero EARS findings.
- Exact traceability: zero missing/unexpected DistANN criterion mappings and
  zero missing stable error categories.

## Explicit non-claims / remaining Task 179 work

This is not a Task 179 closeout. Node registration/unregistration, staged batch
ingest and seal/Ready receipts, Task 163 D8 streamed handoff integration,
publication/recovery/retention, generation-aware expansion/materialization,
true three-PostgreSQL-instance topology/read validation, and 10k/50k/100k
`ecaz bench suite` A/B evidence remain open. No benchmark result is claimed by
this packet.
