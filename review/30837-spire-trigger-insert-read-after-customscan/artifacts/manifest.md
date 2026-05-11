# Artifact Manifest

Packet: `30837-spire-trigger-insert-read-after-customscan`

Head SHA: `e68dade0c0d33ee11d14004f459d3db09474e0b5`

Timestamp: `2026-05-11 11:23-11:31 America/Los_Angeles`

## Artifacts

### `multicluster-insert-read-after-customscan.log`

- Command: `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh --skip-install --insert-mode trigger --artifact-dir review/30837-spire-trigger-insert-read-after-customscan/artifacts --run-id 20260511T185000Z-trigger --remote-port 39244 --coord-port 39245`
- Lane / fixture: PG18 multicluster coordinator + remote trigger INSERT read-after-CustomScan smoke.
- Storage format / rerank mode: SPIRE `rabitq` indexes, default rerank behavior for the fixture.
- Cluster layout: two local PG18 clusters with separate coordinator and remote data directories.
- Isolated one-index-per-table or shared-table surface: isolated one-index-per-table fixture tables.
- Result:
  - `insert_mode=trigger`
  - `remote_epoch_after_insert=2`
  - `remote_identity_hex_after_insert=1566947d2ec7c239`
  - `descriptor_row=93,2,2,1566947d2ec7c239`
  - `insert_result=trigger_insert_committed`
  - `coordinator_row_count=0`
  - `remote_row=303,remote inserted via coordinator`
  - `placement_row=2,3,2`
  - `Custom Scan (EcSpireDistributedScan)`
  - `read_row=303,remote inserted via coordinator`
  - `SPIRE multicluster coordinator insert read-after-CustomScan passed`

### `remote-postgres.log`

- Command/source: remote PostgreSQL server log from trigger-mode smoke.
- Lane / fixture: PG18 multicluster remote node.
- Storage format / rerank mode: same as trigger-mode smoke.
- Cluster layout: remote cluster.
- Isolated one-index-per-table or shared-table surface: isolated fixture tables.
- Result: retained for review/debugging; smoke assertions passed.

### `coord-postgres.log`

- Command/source: coordinator PostgreSQL server log from trigger-mode smoke.
- Lane / fixture: PG18 multicluster coordinator node.
- Storage format / rerank mode: same as trigger-mode smoke.
- Cluster layout: coordinator cluster.
- Isolated one-index-per-table or shared-table surface: isolated fixture tables.
- Result: retained for review/debugging; smoke assertions passed.

### `helper-mode/multicluster-insert-read-after-customscan.log`

- Command: `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh --skip-install --insert-mode helper --artifact-dir review/30837-spire-trigger-insert-read-after-customscan/artifacts/helper-mode --run-id 20260511T185500Z-helper --remote-port 39246 --coord-port 39247`
- Lane / fixture: PG18 multicluster direct-helper read-after-CustomScan smoke.
- Storage format / rerank mode: SPIRE `rabitq` indexes, default rerank behavior for the fixture.
- Cluster layout: two local PG18 clusters with separate coordinator and remote data directories.
- Isolated one-index-per-table or shared-table surface: isolated one-index-per-table fixture tables.
- Result:
  - `insert_mode=helper`
  - `descriptor_row=93,2,2,1566947d2ec7c239`
  - `insert_result=2,remote_insert_prepared_pending_local_commit,await_local_commit,true,true`
  - `coordinator_row_count=not_applicable`
  - `Custom Scan (EcSpireDistributedScan)`
  - `read_row=303,remote inserted via coordinator`
  - `SPIRE multicluster coordinator insert read-after-CustomScan passed`

### `helper-mode/remote-postgres.log`

- Command/source: remote PostgreSQL server log from helper-mode smoke.
- Lane / fixture: PG18 multicluster remote node.
- Storage format / rerank mode: same as helper-mode smoke.
- Cluster layout: remote cluster.
- Isolated one-index-per-table or shared-table surface: isolated fixture tables.
- Result: retained for review/debugging; smoke assertions passed.

### `helper-mode/coord-postgres.log`

- Command/source: coordinator PostgreSQL server log from helper-mode smoke.
- Lane / fixture: PG18 multicluster coordinator node.
- Storage format / rerank mode: same as helper-mode smoke.
- Cluster layout: coordinator cluster.
- Isolated one-index-per-table or shared-table surface: isolated fixture tables.
- Result: retained for review/debugging; smoke assertions passed.

### `bash-n.log`

- Command: `script -q -e -c "bash -n scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh" review/30837-spire-trigger-insert-read-after-customscan/artifacts/bash-n.log`
- Lane / fixture: shell syntax check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30837-spire-trigger-insert-read-after-customscan/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
