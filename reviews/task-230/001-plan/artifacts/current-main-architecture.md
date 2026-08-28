# Task 230 current-main architecture grounding

Inspected exact main `23fb9b7ba1f0803be5dfc700d9865f80fbf60862` before
revising packet 001.

## Existing format and physical relations

- `generation_descriptor.rs` currently supports descriptor V2 (plain row
  heap) and V3 (Task 229 payload cover). Both freeze the complete
  `DistannRowSchemaDescriptor`; V3 is selected only when `payload_cover` is
  present.
- `generation_store.rs` creates a generation-owned full-schema row heap,
  graph heap, and partial unique graph directory. A Task 229 generation may
  additionally own a sidecar heap and directory. All share deterministic
  build-id names, owner, permanence, tablespace, internal control dependency,
  rollback, and reclaim handling.
- The graph heap stores `(vec_id, graph_record, row_tid, record_version,
  is_current)`. Graph record V1 embeds the same row TID in a fixed six-byte
  field. `DistannNodeTuple::decode_physical_v1` rejects any other version.
- `generation_catalog.rs` catalogs non-null row/graph/directory OIDs and the
  optional paired Task 229 sidecar OIDs. The row relation is assumed to have
  the complete frozen schema.

## Current write and lifecycle paths

- Handoff wire V1 already carries every non-dropped source attribute as
  canonical PostgreSQL binary values plus a NULL bitmap. `handoff.rs` resolves
  receive/send functions, reconstructs one full row tuple, inserts it, writes
  its TID into graph, and recomputes graph/row/directory digests at Ready.
- `physical_dml.rs` prepares the complete source row, appends it before graph
  publication, and performs insert/replacement graph changes in the same owner
  transaction. Delete changes the graph tombstone only. Task 229 adds an
  optional append between row and graph writes but does not change row-heap
  authority.
- Ready receipt V1 records graph, row, and directory digests/bytes. V2 adds the
  Task 229 sidecar digest/bytes. Manifest V2 is the row-heap form and V3 is the
  sidecar form; fingerprint bytes begin with the manifest version.
- Abort, cancel, retirement, reclaim, REINDEX, and cache invalidation enumerate
  the cataloged relation identities. These fixed-relation consumers must all
  become layout-aware together.

## Current reads and materialization

- `GenerationExpander` opens row, graph, and directory relations. Exact scoring
  and owner-local source-vector reads fetch the graph record's row TID from the
  full row heap, then call `slot_getattr(source_attnum)`. This deforms the same
  tuple descriptor that contains cold arrays and payload values.
- Traversal-replica construction uses that same exact-vector path. Graph-only
  diagnostics deliberately avoid the row heap.
- `materialize_payloads` validates the retained full row schema, resolves graph
  records to row TIDs, and executes one ordered projection query against the
  full row relation. Task 229 can substitute its sidecar only for an exact
  typed mask fully covered by the retained descriptor.
- `custom_scan.rs` already owns Task 222's typed `PayloadAttributeMask` and
  binary receive metadata for the original source tuple descriptor. Remote
  rows are materialized as virtual tuples; local frozen rows use retained
  row-tier TIDs or Task 229 pending-sidecar rows.
- The retry contract for a graph-visible row whose row tuple is not visible to
  the first snapshot refreshes once with `GetLatestSnapshot`; a second miss is
  fail-closed. Remote materialization retains a distinct skip marker when the
  referenced tuple is absent under the request snapshot.

## Standard measurement surface

- `ecaz dev distann-multicluster` creates
  `(id bigint, source_id uuid NOT NULL, source real[], embedding
  ecvector(1536)[, payload_note text])` for the current staged corpus.
- The Task 229 final matrix established the checked-in suite runner,
  counterbalanced fresh-fixture positions, exact prediction comparison,
  materialization telemetry, DML rows, and per-relation storage capture that
  Task 230 can extend. Task 229 closed STOP and its sidecar must be disabled in
  both Task 230 arms.
- `data/staged-current/ec_real_100k_manifest.json` records dimension 1536. The
  planning storage estimate in `request.md` uses only that schema/dimension and
  an explicit fixed-overhead bound; it cites no measured Task 230 result.

## Inspection scope

Read-only `rg`/`sed` inspection covered the task/program ledger, Task 229
planning and final decision packets, `options.rs`, `row_schema.rs`,
`payload_sidecar.rs`, `generation_descriptor.rs`, `tuple.rs`,
`generation_store.rs`, `generation_catalog.rs`, `handoff_wire.rs`,
`handoff.rs`, `manifest_v2.rs`, `generation_read.rs`, `remote_endpoint.rs`,
`custom_scan.rs`, `physical_dml.rs`, `sql/bootstrap.sql`, the Task 229 suite
config, and the staged corpus manifest. No build, PostgreSQL, test, fixture, or
benchmark command ran.
