# SPIRE Coordinator INSERT Descriptor Refresh

## Scope

This packet closes the follow-up left by packets `30834` and `30835`: the
coordinator INSERT helper now refreshes the active remote-node descriptor from
the remote transaction's post-INSERT index identity before staging local
placement.

Changes:

- `coordinator_insert_prepare_remote_sql(...)` now probes the remote index
  active epoch and endpoint fingerprint inside the same remote transaction,
  after the tuple INSERT and before `PREPARE TRANSACTION`.
- `ec_spire_prepare_coordinator_insert_tuple_payload(...)` advances
  `ec_spire_remote_node_descriptor.descriptor_generation` and writes the
  post-INSERT `remote_index_identity`, `last_served_epoch`,
  `min_retained_epoch`, and `extension_version` in the coordinator
  transaction before placement staging.
- The descriptor refresh is intentionally local-transactional: if placement
  staging aborts, the descriptor update rolls back and the existing xact
  callback rolls back the remote prepared transaction.
- The multicluster read-after-insert smoke no longer manually calls
  `ec_spire_register_remote_node_descriptor(...)` after insert. It asserts the
  helper-updated descriptor row instead.
- ADR-069 and the Phase 11 tracker now mark automatic descriptor refresh done.

## Validation

- `cargo test coordinator_insert --lib`
  - result: pass, 7 tests.
  - coverage includes the public helper and trigger-front-door tests asserting
    descriptor generation advancement and nonempty refreshed identity.
- `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh --skip-install --artifact-dir review/30836-spire-insert-descriptor-refresh/artifacts --run-id 20260511T183000Z`
  - result: pass.
  - key lines:
    - `remote_epoch_after_insert=2`
    - `remote_identity_hex_after_insert=1566947d2ec7c239`
    - `descriptor_row=93,2,2,1566947d2ec7c239`
    - `insert_result=2,remote_insert_prepared_pending_local_commit,await_local_commit,true,true`
    - `plan=Limit -> Custom Scan (EcSpireDistributedScan) on ec_spire_insert_read_coord_sql`
    - `read_row=303,remote inserted via coordinator`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the descriptor refresh belongs in the public coordinator INSERT helper
  rather than in the remote endpoint itself.
- Confirm the insert metadata probe is correctly weaker than the read-path
  endpoint validator: it decodes the endpoint fingerprint and extension version
  without requiring read-ready status, so non-rabitq write targets are not
  rejected solely by descriptor refresh.
- Confirm the local descriptor update is ordered correctly before placement
  staging and fails closed if descriptor generation cannot advance.

## Artifacts

- `review/30836-spire-insert-descriptor-refresh/artifacts/manifest.md`
- `review/30836-spire-insert-descriptor-refresh/artifacts/cargo-test-coordinator-insert-lib.log`
- `review/30836-spire-insert-descriptor-refresh/artifacts/multicluster-insert-read-after-customscan.log`
- `review/30836-spire-insert-descriptor-refresh/artifacts/remote-postgres.log`
- `review/30836-spire-insert-descriptor-refresh/artifacts/coord-postgres.log`
- `review/30836-spire-insert-descriptor-refresh/artifacts/cargo-fmt-check.log`
- `review/30836-spire-insert-descriptor-refresh/artifacts/git-diff-check.log`
