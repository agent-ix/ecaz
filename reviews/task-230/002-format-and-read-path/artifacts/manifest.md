# Task 230 packet 002 artifact manifest

- Head SHA: `ef558a669`
- Task bucket: `reviews/task-230/002-format-and-read-path/`
- Packet: descriptor/reloption foundation checkpoint
- Timestamp: 2026-08-28 America/Los_Angeles
- Lane / fixture / storage format / rerank mode: not applicable (pure format
  descriptor and reloption unit tests)
- Isolation: not applicable; no index, table, corpus, or benchmark fixture was
  created

## Artifacts

### `row-layout-tests.log`

- Command: `cargo test --no-default-features --features pg18 row_layout::tests`
- Expected cited result: `5 passed; 0 failed`

### `row-layout-reloption-test.log`

- Command: `cargo test --no-default-features --features pg18 hot_payload_attnums_and_layout_are_canonical`
- Expected cited result: `1 passed; 0 failed`

### `format-check.log`

- Command: `cargo fmt --all -- --check`
- Expected cited result: exit status 0 (the host's stable-rustfmt warnings about
  nightly-only import grouping are non-failures)

Both commands use the host's shared `CARGO_TARGET_DIR`; no runtime output is
written under the repository `target/` directory.
