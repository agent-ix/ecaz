# Artifact Manifest: SPIRE Prepared Xact GID Runbook

- Packet: `30914-spire-prepared-xact-gid-runbook`
- Code checkpoint SHA: `44016967` (`Stabilize SPIRE prepared transaction GIDs`)
- Current branch SHA while final artifacts were written: `99e41159`
  (`Review SPIRE Phase 12.1 / 12.2 design / 12.3 placement gate index`)
- Timestamp: `2026-05-12T10:33:42-07:00`
- Isolated one-index-per-table or shared-table surface: loopback PG18 fixtures
  using the existing shared `ec_spire_placement` table.

## Artifacts

### `git-diff-check.log`

- Head SHA: `44016967`
- Lane / fixture / storage format / rerank mode: static diff check; not
  storage-format or rerank specific.
- Command: `git diff --check 95f7487e..44016967`
- Key result lines: command exited 0 with no whitespace errors.

### `cargo-fmt-check.log`

- Head SHA: `44016967` code tree, with later reviewer-feedback files present
  in the worktree.
- Lane / fixture / storage format / rerank mode: formatting check; not
  storage-format or rerank specific.
- Command: `cargo fmt --check`
- Key result lines: command exited 0. Rustfmt printed the repository's existing
  stable-toolchain warnings for unstable import-grouping options.

### `cargo-test-insert-remote-prepare-lib.log`

- Head SHA: `44016967` code tree, with later reviewer-feedback files present
  in the worktree.
- Lane / fixture / storage format / rerank mode: PG18 loopback remote INSERT
  prepare fixtures; default SPIRE storage; rerank mode not applicable.
- Command: `cargo test insert_remote_prepare --lib --no-default-features --features pg18`
- Key result lines:
  - `test tests::pg_test_ec_spire_insert_remote_prepare_tuple_payload_endpoint_sql ... ok`
  - `test tests::pg_test_ec_spire_insert_remote_prepare_stages_placement_sql ... ok`
  - `test result: ok. 2 passed; 0 failed`

### `cargo-pgrx-test-coordinator-insert-tuple-payload.log`

- Head SHA: `44016967` code tree, with later reviewer-feedback files present
  in the worktree.
- Lane / fixture / storage format / rerank mode: PG18 loopback coordinator
  INSERT tuple-payload fixture; default SPIRE storage; rerank mode not
  applicable.
- Command: `cargo pgrx test pg18 test_ec_spire_prepare_coordinator_insert_tuple_payload_sql`
- Key result lines:
  - `test tests::pg_test_ec_spire_prepare_coordinator_insert_tuple_payload_sql ... ok`
  - `test result: ok. 1 passed; 0 failed`

### `cargo-pgrx-test-coordinator-delete-tuple-payload.log`

- Head SHA: `44016967` code tree, with later reviewer-feedback files present
  in the worktree.
- Lane / fixture / storage format / rerank mode: PG18 loopback coordinator
  DELETE tuple-payload fixture; default SPIRE storage; rerank mode not
  applicable.
- Command: `cargo pgrx test pg18 test_ec_spire_prepare_coordinator_delete_tuple_payload_sql`
- Key result lines:
  - `test tests::pg_test_ec_spire_prepare_coordinator_delete_tuple_payload_sql ... ok`
  - `test result: ok. 1 passed; 0 failed`
