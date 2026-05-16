# Artifact Manifest: SPIRE Local Store Read-Overlap Harness

- Head SHA: `975e8d830f1e9e13986e3ab5e5e5133df4ee10f7`
- Packet/topic: `30948-spire-local-store-read-overlap-harness`
- Timestamp: `2026-05-13T01:02:03Z`
- Surface: local PG18 multi-store scan diagnostics and read-overlap harness
- Lane / fixture / storage format / rerank mode: PG18;
  `test_ec_spire_multistore_read_overlap_harness_sql`; relation-backed
  two-store local scan fixture; TurboQuant payloads; `rerank_width = 10`.
- Isolation surface: isolated one-index table fixture; no shared-table remote
  surface.

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check 975e8d83^ 975e8d83" review/30948-spire-local-store-read-overlap-harness/artifacts/git-diff-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Command:
  `script -q -c "cargo fmt --check" review/30948-spire-local-store-read-overlap-harness/artifacts/cargo-fmt-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`
  - rustfmt emitted the repository's stable-toolchain warnings for nightly-only
    import grouping options.

### `unit-scan-placement-diagnostics.log`

- Command:
  `script -q -c "cargo test --no-default-features --features pg18 collect_scan_placement_diagnostics_counts_routed_store_rows --lib" review/30948-spire-local-store-read-overlap-harness/artifacts/unit-scan-placement-diagnostics.log`
- Key result lines:
  - `test am::ec_spire::scan::tests::collect_scan_placement_diagnostics_counts_routed_store_rows ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1689 filtered out`

### `unit-read-batch-prefetch.log`

- Command:
  `script -q -c "cargo test --no-default-features --features pg18 prefetch_store_object_read_groups_prefetches_every_store_before_scoring --lib" review/30948-spire-local-store-read-overlap-harness/artifacts/unit-read-batch-prefetch.log`
- Key result lines:
  - `test am::ec_spire::scan::tests::prefetch_store_object_read_groups_prefetches_every_store_before_scoring ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1689 filtered out`

### `pg18-read-overlap-harness.log`

- Command:
  `script -q -c "cargo pgrx test pg18 test_ec_spire_multistore_read_overlap_harness_sql" review/30948-spire-local-store-read-overlap-harness/artifacts/pg18-read-overlap-harness.log`
- Key result lines:
  - `Discovered 814 SQL entities: ... 811 functions`
  - `test tests::pg_test_ec_spire_multistore_read_overlap_harness_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1689 filtered out`
  - `COMMAND_EXIT_CODE="0"`
