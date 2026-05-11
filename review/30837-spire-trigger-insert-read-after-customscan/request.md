# SPIRE Trigger INSERT Read-After-CustomScan

## Scope

This packet adds the PG18 multicluster fixture for actual
`INSERT INTO coordinator_table ...` through the trigger front door from packet
`30835`, now that packet `30836` made descriptor refresh automatic.

Changes:

- `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh` now
  accepts `--insert-mode helper|trigger`.
- Helper mode remains the existing direct
  `ec_spire_prepare_coordinator_insert_tuple_payload(...)` path.
- Trigger mode installs `ec_spire_enable_coordinator_insert(...)`, performs an
  actual SQL `INSERT INTO` on the coordinator table, and asserts:
  - the coordinator heap row is suppressed;
  - the row is committed on the remote;
  - placement is staged with the trigger's canonical bigint primary-key bytes;
  - the remote descriptor is refreshed to the post-insert identity;
  - `Custom Scan (EcSpireDistributedScan)` returns the inserted remote row.
- The Phase 11 task tracker now marks trigger read-after-insert multicluster
  coverage complete.

## Validation

- `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh --skip-install --insert-mode trigger --artifact-dir review/30837-spire-trigger-insert-read-after-customscan/artifacts --run-id 20260511T185000Z-trigger --remote-port 39244 --coord-port 39245`
  - result: pass.
  - key lines:
    - `insert_mode=trigger`
    - `descriptor_row=93,2,2,1566947d2ec7c239`
    - `insert_result=trigger_insert_committed`
    - `coordinator_row_count=0`
    - `remote_row=303,remote inserted via coordinator`
    - `placement_row=2,3,2`
    - `Custom Scan (EcSpireDistributedScan)`
    - `read_row=303,remote inserted via coordinator`
- `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh --skip-install --insert-mode helper --artifact-dir review/30837-spire-trigger-insert-read-after-customscan/artifacts/helper-mode --run-id 20260511T185500Z-helper --remote-port 39246 --coord-port 39247`
  - result: pass, proving the helper mode still works after the shared schema
    and mode refactor.
- `bash -n scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh`
  - result: pass.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm that using one script with explicit insert modes is clearer than
  duplicating the multicluster setup.
- Confirm trigger mode is asserting the production behavior that matters:
  coordinator heap suppression, remote row visibility after commit, placement
  directory row, descriptor refresh, and CustomScan read-back.
- Confirm the source-identity column added to the fixture schema does not weaken
  the existing helper-mode coverage.

## Artifacts

- `review/30837-spire-trigger-insert-read-after-customscan/artifacts/manifest.md`
- `review/30837-spire-trigger-insert-read-after-customscan/artifacts/multicluster-insert-read-after-customscan.log`
- `review/30837-spire-trigger-insert-read-after-customscan/artifacts/remote-postgres.log`
- `review/30837-spire-trigger-insert-read-after-customscan/artifacts/coord-postgres.log`
- `review/30837-spire-trigger-insert-read-after-customscan/artifacts/helper-mode/multicluster-insert-read-after-customscan.log`
- `review/30837-spire-trigger-insert-read-after-customscan/artifacts/helper-mode/remote-postgres.log`
- `review/30837-spire-trigger-insert-read-after-customscan/artifacts/helper-mode/coord-postgres.log`
- `review/30837-spire-trigger-insert-read-after-customscan/artifacts/bash-n.log`
- `review/30837-spire-trigger-insert-read-after-customscan/artifacts/git-diff-check.log`
