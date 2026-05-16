# Artifact Manifest: 30797 SPIRE Row Materialization Provider Seam

- head SHA: `fce3919293deef2c64bddde6e656c4498759b714`
- packet/topic: `30797-spire-row-materialization-provider-seam`
- timestamp: `2026-05-10T19:19:59-07:00`
- lane: Task 30 Phase 11.5 Stage D remote row materialization provider seam
- fixture: Rust unit tests under `--no-default-features --features pg18`
- storage format: synthetic production scan output rows; no index storage mutation
- rerank mode: not applicable
- isolated/shared surface: isolated in-memory scan result stream helpers; no shared-table SQL fixture

## Artifacts

### `row-materialization-provider.log`

- command: `script -q -c "cargo test production_scan_row_materialization --no-default-features --features pg18" review/30797-spire-row-materialization-provider-seam/artifacts/row-materialization-provider.log`
- key result lines:
  - `running 4 tests`
  - `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1595 filtered out`
- coverage:
  - valid full-identity mapping converts a remote-origin output to a coordinator materialized heap TID owner.
  - missing mapping remains a classified `remote_row_materialization` blocker.
  - requested epoch, served epoch, origin node, vec-id, row locator, heap relation, and scan visibility mismatches are rejected before AM delivery.

### `am-output-materialized-owner.log`

- command: `script -q -c "cargo test production_scan_result_stream_am_outputs --no-default-features --features pg18" review/30797-spire-row-materialization-provider-seam/artifacts/am-output-materialized-owner.log`
- key result lines:
  - `running 3 tests`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1596 filtered out`
- coverage:
  - AM output conversion still accepts coordinator-local heap rows.
  - AM output conversion accepts `coordinator_materialized_heap` rows.
  - AM output conversion still blocks raw remote-origin rows.

## Notes

- The default provider is intentionally empty. This packet adds the validation seam and preserves the current fail-closed behavior until a catalog-backed materialized-row mapping provider is implemented.
