# Task 111b Packet 006 Artifact Manifest: Format Compatibility and Tags

- Head SHA: `643928e947bb3dfb42b8074427bd3052c5e0179a`
- Reviewed code commit: `2cabe4fdda00144dfa93747883579e05530fa98e`
- Task bucket: `reviews/task-111b`
- Packet path: `reviews/task-111b/006-format-compatibility-tags`
- Timestamp: `2026-06-17T19:47:46Z`
- Storage format / lane: PG18 focused compatibility fixtures for row postings, legacy dense blocks, and aligned dense blocks.
- Index/table surface: each PG18 test uses an isolated table/index surface.

## Artifacts

### `cargo-test-row-scan-safety.log`

- Command: `cargo test -q test_ec_ivf_insert_vacuum_scan_safety --lib`
- Purpose: row posting (`0x23`) scan/vacuum safety fixture.
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 2126 filtered out`

### `cargo-test-dense-legacy-scan.log`

- Command: `cargo test -q test_ec_ivf_dense_posting_blocks_scan_build_rows --lib`
- Purpose: legacy dense block (`0x25`) build/scan fixture.
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 2126 filtered out`

### `cargo-test-dense-aligned-scan.log`

- Command: `cargo test -q test_ec_ivf_dense_typed_posting_blocks_scan_build_rows --lib`
- Purpose: aligned dense block (`0x28`) build/scan fixture.
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 2126 filtered out`
