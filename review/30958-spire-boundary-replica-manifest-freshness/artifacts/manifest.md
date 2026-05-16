# Artifact Manifest: SPIRE Boundary Replica Manifest Freshness

- Head SHA: `e7dc395626adca8a773f90d3dcf3f59b4711ea7d`
- Packet/topic: `30958-spire-boundary-replica-manifest-freshness`
- Lane / fixture / storage format / rerank mode: Phase 12.7 boundary replica
  manifest readiness; PG18 SQL fixture over one local `ec_spire` index with
  synthetic remote placement rewrite; SPIRE relation object storage and remote
  epoch-manifest catalog; no heap rerank path exercised.
- Surface isolation: isolated one-index fixture. The test rewrites one leaf
  placement to remote node/local store `2`, persists the remote epoch manifest,
  and then drifts the persisted manifest entry; it does not use shared-table
  multi-index surfaces.
- Timestamp: `2026-05-12T19:15:45-07:00`

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check e7dc3956^ e7dc3956" review/30958-spire-boundary-replica-manifest-freshness/artifacts/git-diff-check.log`
- Result:
  `Script done on 2026-05-12 19:13:08-07:00 [COMMAND_EXIT_CODE="0"]`

### `cargo-fmt-check.log`

- Command:
  `script -q -c "cargo fmt --check" review/30958-spire-boundary-replica-manifest-freshness/artifacts/cargo-fmt-check.log`
- Result:
  `Script done on 2026-05-12 19:13:10-07:00 [COMMAND_EXIT_CODE="0"]`
- Notes:
  Rustfmt emitted the existing stable-toolchain warnings for unstable
  `imports_granularity` and `group_imports` settings.

### `pg18-boundary-replica-manifest-freshness.log`

- Command:
  `script -q -c "cargo pgrx test pg18 test_ec_spire_boundary_replica_manifest_freshness_sql" review/30958-spire-boundary-replica-manifest-freshness/artifacts/pg18-boundary-replica-manifest-freshness.log`
- Key result lines:
  `test tests::pg_test_ec_spire_boundary_replica_manifest_freshness_sql ... ok`
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1692 filtered out; finished in 29.88s`
  `Script done on 2026-05-12 19:15:38-07:00 [COMMAND_EXIT_CODE="0"]`
