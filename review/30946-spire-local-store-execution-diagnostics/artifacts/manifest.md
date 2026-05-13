# Artifact Manifest: SPIRE Local Store Execution Diagnostics

- Head SHA: `cb4a6fd8496ea38ae365bd5473921582e3f7b05c`
- Packet/topic: `30946-spire-local-store-execution-diagnostics`
- Timestamp: `2026-05-13T00:24:54Z`
- Surface: local PG18 scan diagnostics
- Lane / fixture / storage format / rerank mode: PG18;
  `test_ec_spire_scan_placement_snapshot_sql`; relation-backed local scan
  fixture; existing `rerank_width = 10` fixture setting.
- Isolation surface: isolated one-index table fixture; no shared-table remote
  surface.

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check cb4a6fd8^ cb4a6fd8" review/30946-spire-local-store-execution-diagnostics/artifacts/git-diff-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Command:
  `script -q -c "cargo fmt --check" review/30946-spire-local-store-execution-diagnostics/artifacts/cargo-fmt-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`
  - rustfmt emitted the repository's stable-toolchain warnings for nightly-only
    import grouping options.

### `pg18-scan-placement-execution-diagnostics.log`

- Command:
  `script -q -c "cargo pgrx test pg18 test_ec_spire_scan_placement_snapshot_sql" review/30946-spire-local-store-execution-diagnostics/artifacts/pg18-scan-placement-execution-diagnostics.log`
- Key result lines:
  - `Discovered 812 SQL entities: ... 809 functions`
  - `test tests::pg_test_ec_spire_scan_placement_snapshot_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1688 filtered out`
  - `COMMAND_EXIT_CODE="0"`
