# Artifact Manifest

Task bucket: `reviews/task-30/964-spire-phase13e-static-remote-placement-publish`

Head SHA: `6b4c1cfb10185d92fd4d77dcb4d8494a8725f582`

Timestamp: `2026-05-25T10:11:33-07:00`

## cargo-check-ecaz-lib.log

- Command: `script -q -c "cargo check -p ecaz --lib" reviews/task-30/964-spire-phase13e-static-remote-placement-publish/artifacts/cargo-check-ecaz-lib.log`
- Lane: local Rust compile validation
- Fixture: none
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: extension library compile
- Result:
  - `Finished dev profile`
  - `COMMAND_EXIT_CODE="0"`

## cargo-check-ecaz-cli.log

- Command: `script -q -c "cargo check -p ecaz-cli --bin ecaz" reviews/task-30/964-spire-phase13e-static-remote-placement-publish/artifacts/cargo-check-ecaz-cli.log`
- Lane: local Rust compile validation
- Fixture: none
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: operator CLI compile
- Result:
  - `Finished dev profile`
  - `COMMAND_EXIT_CODE="0"`
  - Existing warning: `LoadedDistributedPlacementConfig.path` is never read.

## bash-syntax-spire-aws.log

- Command: `script -q -c "bash -n scripts/spire-aws/*.sh" reviews/task-30/964-spire-phase13e-static-remote-placement-publish/artifacts/bash-syntax-spire-aws.log`
- Lane: local operator-script syntax validation
- Fixture: none
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: AWS operator scripts
- Result:
  - `COMMAND_EXIT_CODE="0"`

## cargo-test-ecaz-lib-pg-symbol-limited.log

- Command: `script -q -c "cargo test -p ecaz --lib ec_spire_publish_static_remote_placement_nodes" reviews/task-30/964-spire-phase13e-static-remote-placement-publish/artifacts/cargo-test-ecaz-lib-pg-symbol-limited.log`
- Lane: attempted local lib test binary execution
- Fixture: none
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: extension lib test harness outside PostgreSQL
- Result:
  - Compile completed.
  - Test binary execution failed before running tests with `undefined symbol: LockBuffer`.
  - `COMMAND_EXIT_CODE="127"`
  - This is not accepted as behavioral validation for the new SQL function; it records the local harness limitation.
