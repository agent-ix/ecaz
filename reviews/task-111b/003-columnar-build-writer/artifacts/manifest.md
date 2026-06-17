# Task 111b Packet 003 Artifact Manifest: Columnar Build Writer

- Head SHA: `e7fed64dfa97870d93e68676f276cd7bff5b9cba`
- Task bucket: `reviews/task-111b`
- Packet path: `reviews/task-111b/003-columnar-build-writer`
- Timestamp: `2026-06-17T18:44:53Z`
- Storage format / lane: unit-level build staging; gated `columnar_frozen_lists = true` path tested with `TurboQuant`, existing dense gates tested separately.
- Index/table surface: not applicable; these are Rust unit tests, not SQL benchmark surfaces.

## Artifacts

### `cargo-test-columnar-list.log`

- Command: `cargo test -q columnar_frozen_list --lib`
- Purpose: validates columnar header/buffer behavior, including item-aligned raw-page splitting for all columns.
- Key result: `7 passed; 0 failed; 0 ignored; 0 measured; 2117 filtered out`

### `cargo-test-columnar-build-writer.log`

- Command: `cargo test -q build_state_can_stage_columnar_frozen_lists_when_gated --lib`
- Purpose: validates the gated build writer stages one header plus raw column pages and points directory tail at the final raw page.
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 2123 filtered out`

### `cargo-test-dense-build-writer.log`

- Command: `cargo test -q build_state_can_stage_dense_posting_blocks_when_gated --lib`
- Purpose: regression check that the existing dense posting block build gate still stages.
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 2123 filtered out`

### `cargo-test-packed-dense-build-writer.log`

- Command: `cargo test -q build_state_can_stage_packed_dense_posting_segments_when_requested --lib`
- Purpose: regression check that the existing packed dense posting segment path still stages.
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 2123 filtered out`
