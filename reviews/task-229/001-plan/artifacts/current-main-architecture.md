# Current-main architecture grounding

Inspected exact main `3419c9c758bea7d9940b27d9afbcf9e627e84879` before
refreshing packet 001.

## Existing reusable contracts

- `src/am/ec_distann/payload_projection.rs` owns the typed Task 222 boundary:
  `PayloadAttributeMask::Exact(attnums)` versus
  `AllColumns(PayloadFallbackReason)`. Its walk includes target entries and
  quals, recognizes whole-row Vars, and deliberately classifies the zero-value
  visibility case as all-column fallback.
- `src/am/ec_distann/custom_scan.rs` stores that typed mask, builds the matching
  binary receive metadata, and currently passes only the resulting attnum list
  into physical materialization. The typed distinction must therefore be
  threaded explicitly for Task 229; inferring eligibility from the expanded
  attnum list would reopen whole-row/ambiguous selection.
- `src/am/ec_distann/row_schema.rs` already freezes every physical attnum's
  name, type namespace/name, typmod, collation identity, dropped/generated
  state, and binary send/receive functions and provides a canonical schema
  fingerprint.
- `src/am/ec_distann/generation_descriptor.rs` currently emits descriptor V2
  containing the complete row schema. Descriptor digest is already carried by
  build spec, Ready receipt, manifest, and participant validation.
- `src/am/ec_distann/manifest_v2.rs` supplies canonical Ready receipt,
  manifest, digest, and the 34-byte `version + manifest digest` fingerprint.
  Current receipt storage is fixed-width, so dual receipt versions require the
  catalog Rust type and SQL check to become version-aware rather than assuming
  303 bytes forever.

## Current physical relations and lifecycle

- `src/am/ec_distann/generation_store.rs` creates exactly a row-tier heap, a
  graph-store heap, and the graph's partial unique B-tree. All share owner,
  persistence, tablespace, deterministic build-id naming, internal control
  dependencies, transactional abort, reclaim, and REINDEX cleanup.
- `src/am/ec_distann/generation_catalog.rs` catalogs those three non-null OIDs.
  `sql/bootstrap.sql` enforces one unique catalog identity per OID and exact
  fixed-width Ready/fingerprint checks.
- `src/am/ec_distann/handoff.rs` prepares every frozen row from canonical
  binary handoff values, inserts row-tier and graph records in one transaction,
  recomputes storage digests/counts at Ready, and records graph/row/directory
  sizes. The canonical handoff bytes allow a sidecar entry to be selected
  without a second type round trip.
- `src/am/ec_distann/participant_lifecycle.rs` publishes the manifest-derived
  fingerprint, retains predecessor generations for pinned readers, and owns
  retirement/reclaim state transitions.

## Current read and DML paths

- `src/am/ec_distann/generation_read.rs::materialize_payloads` validates the
  retained row-schema fingerprint, resolves graph records to row TIDs, builds
  one ordered projection SQL query over those TIDs, and returns packed nulls,
  offsets, and values. `materialize_remote_payload_pairs` groups requests by
  owner and preserves request order.
- `src/am/ec_distann/remote_transport.rs` already sends physical materialize
  requests containing epoch fingerprint, vec_ids, projection attnums, and
  expected schema fingerprint; its wire response shape already matches the
  proposed sidecar decoder output.
- `src/am/ec_distann/custom_scan.rs::run_physical_generation_search` currently
  emits local-owner hits as frozen row-tier TIDs and materializes only remote
  hits as virtual payload rows. A complete owner-local sidecar candidate must
  route eligible local hits through the same projected virtual-row shape.
- `src/am/ec_distann/physical_dml.rs` appends the full row tier before graph
  publication, retires/replaces the current graph version transactionally, and
  uses the same prepared row slot for local and owner-payload inserts. Deletes
  mutate only the current graph record's tombstone flag. This supplies a single
  transaction boundary for sidecar insert/upsert and preserves the existing
  tombstone-retention rule.

## Standard measurement schema

`crates/ecaz-cli/src/commands/dev/distann_multicluster.rs` builds the standard
physical corpus as `(id bigint, source_id uuid, source real[], embedding
ecvector[, payload_note text])`; id-only release queries require attnum 1. A
candidate cover of attnum 1 therefore isolates a compact fixed-width scalar.
The correctness fixture's external `payload_note` remains a natural uncovered
TOAST fallback case.

## Commands

Read-only `rg` and `sed` inspection covered the files named above plus
`options.rs`, `build_coordinator/t2.rs`, `generation_catalog.rs`,
`generation_store.rs`, `handoff.rs`, `physical_dml.rs`, `generation_read.rs`,
`remote_endpoint.rs`, `remote_transport.rs`, `manifest_v2.rs`,
`participant_lifecycle.rs`, `sql/bootstrap.sql`, and the accepted Task 222
decision/config. No build, PostgreSQL, test, fixture, or benchmark command ran.
