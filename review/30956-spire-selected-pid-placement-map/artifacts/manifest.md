# Artifact Manifest: SPIRE Selected PID Placement Map

- Head SHA: `5a066d0583a61eb6266fbbfa368124557d1a51fd`
- Packet/topic: `30956-spire-selected-pid-placement-map`
- Lane / fixture / storage format / rerank mode: Phase 12.7 diagnostics;
  PG18 SQL fixture over one local `ec_spire` index; SPIRE object/placement
  manifests; no heap rerank path exercised.
- Surface isolation: isolated one-index fixture. The test rewrites one selected
  PID placement to a synthetic remote node/local store pair; it does not use a
  shared-table multi-index surface.
- Timestamp: `2026-05-12T18:51:10-07:00`

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check 5a066d05^ 5a066d05" review/30956-spire-selected-pid-placement-map/artifacts/git-diff-check.log`
- Result:
  `Script done on 2026-05-12 18:48:32-07:00 [COMMAND_EXIT_CODE="0"]`

### `cargo-fmt-check.log`

- Command:
  `script -q -c "cargo fmt --check" review/30956-spire-selected-pid-placement-map/artifacts/cargo-fmt-check.log`
- Result:
  `Script done on 2026-05-12 18:48:34-07:00 [COMMAND_EXIT_CODE="0"]`
- Notes:
  Rustfmt emitted the existing stable-toolchain warnings for unstable
  `imports_granularity` and `group_imports` settings.

### `pg18-selected-pid-placement-snapshot.log`

- Command:
  `script -q -c "cargo pgrx test pg18 test_ec_spire_selected_pid_placement_snapshot_sql" review/30956-spire-selected-pid-placement-map/artifacts/pg18-selected-pid-placement-snapshot.log`
- Key result lines:
  `test tests::pg_test_ec_spire_selected_pid_placement_snapshot_sql ... ok`
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1691 filtered out; finished in 30.69s`
  `Script done on 2026-05-12 18:51:04-07:00 [COMMAND_EXIT_CODE="0"]`
