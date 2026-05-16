# Artifact Manifest: SPIRE INSERT Prepare Local Cancellation

- Head SHA: `0283ee7baa3ef898c1c8665e27fb8774778f8440`
- Packet/topic: `30970-spire-insert-prepare-local-cancel`
- Timestamp: `2026-05-13T05:44:38Z`
- Lane / fixture / storage format / rerank mode: Phase 12.4 PG18
  coordinator-routed INSERT 2PC cancellation fixture; storage format and
  rerank mode not exercised.
- Surface isolation: loopback remote using the shared coordinator/remote
  descriptor and shared-table `ec_spire_placement` surfaces; no
  one-index-per-table isolation.

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check 0283ee7b^ 0283ee7b" review/30970-spire-insert-prepare-local-cancel/artifacts/git-diff-check.log`
- Result lines:
  - Command exited successfully with no diff-check findings.

### `cargo-check-pg18.log`

- Command:
  `script -q -c "cargo check --no-default-features --features pg18" review/30970-spire-insert-prepare-local-cancel/artifacts/cargo-check-pg18.log`
- Result lines:
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 0.12s`
  - Existing warning: `ecaz` lib has unused imports in `src/am/mod.rs`.

### `cargo-pgrx-test-insert-prepare-local-cancel.log`

- Command:
  `script -q -c "cargo pgrx test pg18 test_ec_spire_insert_prepare_local_cancel_rolls_back" review/30970-spire-insert-prepare-local-cancel/artifacts/cargo-pgrx-test-insert-prepare-local-cancel.log`
- Result lines:
  - `test tests::pg_test_ec_spire_insert_prepare_local_cancel_rolls_back ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1697 filtered out; finished in 31.84s`
  - The test asserts `local_query_cancelled`, no matching remote
    `pg_prepared_xacts`, and no visible remote row after cancellation.
