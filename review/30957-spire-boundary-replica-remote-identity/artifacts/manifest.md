# Artifact Manifest: SPIRE Boundary Replica Remote Identity

- Head SHA: `e1762b5f45678f90ac29f2bc14cc0ce3cd94071a`
- Packet/topic: `30957-spire-boundary-replica-remote-identity`
- Lane / fixture / storage format / rerank mode: Phase 12.7 boundary replica
  readiness; PG18 SQL fixture over one local `ec_spire` index with synthetic
  remote placement rewrite; SPIRE relation object storage; no heap rerank path
  exercised.
- Surface isolation: isolated one-index fixture. The test rewrites one leaf
  placement to remote node/local store `2` to prove the identity diagnostic
  reports a global vec_id spanning coordinator-local and remote placement
  metadata; it does not use shared-table multi-index surfaces.
- Timestamp: `2026-05-12T19:05:55-07:00`

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check e1762b5f^ e1762b5f" review/30957-spire-boundary-replica-remote-identity/artifacts/git-diff-check.log`
- Result:
  `Script done on 2026-05-12 19:03:21-07:00 [COMMAND_EXIT_CODE="0"]`

### `cargo-fmt-check.log`

- Command:
  `script -q -c "cargo fmt --check" review/30957-spire-boundary-replica-remote-identity/artifacts/cargo-fmt-check.log`
- Result:
  `Script done on 2026-05-12 19:03:21-07:00 [COMMAND_EXIT_CODE="0"]`
- Notes:
  Rustfmt emitted the existing stable-toolchain warnings for unstable
  `imports_granularity` and `group_imports` settings.

### `pg18-boundary-replica-identity.log`

- Command:
  `script -q -c "cargo pgrx test pg18 test_ec_spire_boundary_replica_identity_snapshot_global_ids" review/30957-spire-boundary-replica-remote-identity/artifacts/pg18-boundary-replica-identity.log`
- Key result lines:
  `test tests::pg_test_ec_spire_boundary_replica_identity_snapshot_global_ids ... ok`
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1691 filtered out; finished in 30.34s`
  `Script done on 2026-05-12 19:05:49-07:00 [COMMAND_EXIT_CODE="0"]`
