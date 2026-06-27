# Task 111b Packet 004 Artifact Manifest: Columnar Scan and Vacuum

- Head SHA: `e631ac4c3ca8fa6ad71c46062aa93d952e0ed721`
- Task bucket: `reviews/task-111b`
- Packet path: `reviews/task-111b/004-columnar-scan-vacuum`
- Timestamp: `2026-06-17T19:22:24Z`
- Storage format / lane: gated `columnar_frozen_lists = 1`, TurboQuant PG18 fixture plus Rust unit coverage.
- Index/table surface: PG18 test uses a single table/index with an isolated index surface; Rust unit tests are not SQL surfaces.

## Artifacts

### `cargo-test-columnar-raw-pages.log`

- Command: `cargo test -q columnar_frozen_list_raw_pages_keep_all_column_items_whole --lib`
- Purpose: verifies item-aligned raw page derivation, header-derived page lengths, and copied logical byte decoding.
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 2124 filtered out`

### `cargo-test-columnar-stage.log`

- Command: `cargo test -q build_state_can_stage_columnar_frozen_lists_when_gated --lib`
- Purpose: regression check that the gated columnar build writer still stages header/raw pages after the raw-page guard change.
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 2124 filtered out`

### `cargo-test-columnar-pg-scan-vacuum.log`

- Command: `cargo test -q test_ec_ivf_columnar_frozen_lists_scan_insert_vacuum --lib`
- Purpose: PG18 end-to-end validation for columnar build, scan, inserted row deltas, vacuum bitmap marking, directory counts, and post-vacuum scan.
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 2124 filtered out`
