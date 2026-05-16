# Artifact Manifest

Packet: `30836-spire-insert-descriptor-refresh`

Head SHA: `3ea245e24b7a9347c3456cbbcb5c429f4dae24c7`

Timestamp: `2026-05-11 11:16-11:30 America/Los_Angeles`

## Artifacts

### `cargo-test-coordinator-insert-lib.log`

- Command: `script -q -e -c "cargo test coordinator_insert --lib" review/30836-spire-insert-descriptor-refresh/artifacts/cargo-test-coordinator-insert-lib.log`
- Lane / fixture: Rust-side PG18 `pg_test` lane, focused coordinator INSERT tests.
- Storage format / rerank mode: mixed unit coverage; not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster.
- Isolated one-index-per-table or shared-table surface: isolated test tables.
- Result:
  - `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1631 filtered out`
  - Includes `pg_test_ec_spire_prepare_coordinator_insert_tuple_payload_sql ... ok`
  - Includes `pg_test_ec_spire_enable_coordinator_insert_trigger_sql ... ok`

### `multicluster-insert-read-after-customscan.log`

- Command: `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh --skip-install --artifact-dir review/30836-spire-insert-descriptor-refresh/artifacts --run-id 20260511T183000Z`
- Lane / fixture: PG18 multicluster coordinator + remote read-after-insert smoke.
- Storage format / rerank mode: SPIRE `rabitq` indexes, default rerank behavior for the fixture.
- Cluster layout: two local PG18 clusters with separate coordinator and remote data directories.
- Isolated one-index-per-table or shared-table surface: isolated one-index-per-table fixture tables.
- Result:
  - `remote_epoch_after_insert=2`
  - `remote_identity_hex_after_insert=1566947d2ec7c239`
  - `descriptor_row=93,2,2,1566947d2ec7c239`
  - `insert_result=2,remote_insert_prepared_pending_local_commit,await_local_commit,true,true`
  - `remote_row=303,remote inserted via coordinator`
  - `placement_row=2,3,2`
  - `Custom Scan (EcSpireDistributedScan)`
  - `read_row=303,remote inserted via coordinator`
  - `SPIRE multicluster coordinator insert read-after-CustomScan passed`

### `remote-postgres.log`

- Command/source: remote PostgreSQL server log from the multicluster smoke above.
- Lane / fixture: PG18 multicluster remote node.
- Storage format / rerank mode: same as multicluster smoke.
- Cluster layout: remote cluster.
- Isolated one-index-per-table or shared-table surface: isolated fixture tables.
- Result: retained for review/debugging; smoke assertions passed.

### `coord-postgres.log`

- Command/source: coordinator PostgreSQL server log from the multicluster smoke above.
- Lane / fixture: PG18 multicluster coordinator node.
- Storage format / rerank mode: same as multicluster smoke.
- Cluster layout: coordinator cluster.
- Isolated one-index-per-table or shared-table surface: isolated fixture tables.
- Result: retained for review/debugging; smoke assertions passed.

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30836-spire-insert-descriptor-refresh/artifacts/cargo-fmt-check.log`
- Lane / fixture: formatter check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass with the repo's existing stable-rustfmt warnings.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30836-spire-insert-descriptor-refresh/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
