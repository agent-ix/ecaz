# Artifact Manifest: SPIRE Delta Reuse Coverage

- Head SHA: `f86fdcca04da208f5b761d8fb9be002ac91895f1`
- Packet/topic: `30947-spire-delta-reuse-coverage`
- Timestamp: `2026-05-13T00:40:40Z`
- Surface: selected-leaf delta decode reuse under local and remote SPIRE
  candidate collection
- Lane / fixture / storage format / rerank mode: PG18;
  `load_delta_rows_for_routes_reads_each_delta_object_once` and
  `test_ec_spire_remote_search_local_heap_resolution_plan`; relation-backed
  scan fixtures; existing fixture rerank settings.
- Isolation surface: isolated one-index table fixtures; no shared-table
  placement surface.

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check f86fdcca^ f86fdcca" review/30947-spire-delta-reuse-coverage/artifacts/git-diff-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Command:
  `script -q -c "cargo fmt --check" review/30947-spire-delta-reuse-coverage/artifacts/cargo-fmt-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`
  - rustfmt emitted the repository's stable-toolchain warnings for nightly-only
    import grouping options.

### `unit-delta-rows-read-once.log`

- Command:
  `script -q -c "cargo test --no-default-features --features pg18 load_delta_rows_for_routes_reads_each_delta_object_once --lib" review/30947-spire-delta-reuse-coverage/artifacts/unit-delta-rows-read-once.log`
- Key result lines:
  - `test am::ec_spire::scan::tests::load_delta_rows_for_routes_reads_each_delta_object_once ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1688 filtered out`
  - `COMMAND_EXIT_CODE="0"`

### `pg18-remote-local-heap-delta-reuse.log`

- Command:
  `script -q -c "cargo pgrx test pg18 test_ec_spire_remote_search_local_heap_resolution_plan" review/30947-spire-delta-reuse-coverage/artifacts/pg18-remote-local-heap-delta-reuse.log`
- Key result lines:
  - `Discovered 812 SQL entities: ... 809 functions`
  - `test tests::pg_test_ec_spire_remote_search_local_heap_resolution_plan ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1688 filtered out`
  - `COMMAND_EXIT_CODE="0"`
