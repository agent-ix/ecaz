# Artifact Manifest: SPIRE Boundary Replica Placement Diagnostics

- Head SHA: `5437395e962bfa97f5b6f8657f5b4b80c8b6b4be`
- Packet/topic: `30959-spire-boundary-replica-placement-diagnostics`
- Lane / fixture / storage format / rerank mode: Phase 12.7 boundary replica
  placement diagnostics; PG18 SQL fixture over isolated local `ec_spire`
  indexes; SPIRE relation object storage with test-only placement state
  rewrites; no heap rerank path exercised.
- Surface isolation: isolated one-index-per-table fixtures. One fixture builds
  without boundary replicas to prove missing coverage reporting; one fixture
  builds with `boundary_replica_count = 1` and rewrites a boundary-replica leaf
  through unavailable, skipped, and stale states. It does not use shared-table
  multi-index surfaces.
- Timestamp: `2026-05-12T19:33:56-07:00`

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check 5437395e962bfa97f5b6f8657f5b4b80c8b6b4be^ 5437395e962bfa97f5b6f8657f5b4b80c8b6b4be" review/30959-spire-boundary-replica-placement-diagnostics/artifacts/git-diff-check.log`
- Result:
  `Script done on 2026-05-12 19:30:55-07:00 [COMMAND_EXIT_CODE="0"]`

### `cargo-fmt-check.log`

- Command:
  `script -q -c "cargo fmt --check" review/30959-spire-boundary-replica-placement-diagnostics/artifacts/cargo-fmt-check.log`
- Result:
  `Script done on 2026-05-12 19:30:57-07:00 [COMMAND_EXIT_CODE="0"]`
- Notes:
  Rustfmt emitted the existing stable-toolchain warnings for unstable
  `imports_granularity` and `group_imports` settings.

### `pg18-boundary-replica-placement-diagnostics.log`

- Command:
  `script -q -c "cargo pgrx test pg18 test_ec_spire_boundary_replica_placement_diagnostics_sql" review/30959-spire-boundary-replica-placement-diagnostics/artifacts/pg18-boundary-replica-placement-diagnostics.log`
- Key result lines:
  `test tests::pg_test_ec_spire_boundary_replica_placement_diagnostics_sql ... ok`
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1693 filtered out; finished in 32.79s`
  `Script done on 2026-05-12 19:33:45-07:00 [COMMAND_EXIT_CODE="0"]`
