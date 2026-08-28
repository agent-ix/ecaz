# Task 230 packet 002 artifact manifest

- Head SHA: `7b8edce68`
- Task bucket: `reviews/task-230/002-format-and-read-path/`
- Packet: Graph V2 locator-trailer checkpoint (descriptor foundation retained)
- Timestamp: 2026-08-28 America/Los_Angeles
- Lane / fixture / storage format / rerank mode: not applicable (pure format
  descriptor and reloption unit tests)
- Isolation: not applicable; no index, table, corpus, or benchmark fixture was
  created

## Seq-04 Graph V2 review-gap artifacts

### `graph-v2-tests-seq-04.log`

- Command: `cargo test --no-default-features --features pg18 distann_physical_node`
- Cited result: `5 passed; 0 failed`, including rejection of a valid cold
  locator by both V1 writers and V2-to-V1 pooled decode locator clearing.

### `graph-v2-fixtures-seq-04.log`

- Command: `cargo test --no-default-features --features pg18 --test on_disk_fixtures distann_physical_graph_record`
- Cited result: `2 passed; 0 failed`

### `format-check-seq-04.log`

- Command: `cargo fmt --all -- --check`
- Cited result: exit status 0 (stable-rustfmt nightly-option warnings are
  non-failures)

### `clippy-seq-04.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Cited result: nonzero only for the same five pre-existing failures in
  `ambuild.rs:139`, `generation_descriptor.rs:798`, `head_sample.rs:1818`,
  `remote_endpoint.rs:1069`, and `ec_distann_physical_lifecycle.rs:7951`; no
  failure in `tuple.rs`.

## Seq-03 Graph V2 artifacts

### `graph-v2-tests-seq-03.log`

- Command: `cargo test --no-default-features --features pg18 distann_physical_node`
- Expected cited result: `3 passed; 0 failed`

### `graph-v2-fixtures-seq-03.log`

- Command: `cargo test --no-default-features --features pg18 --test on_disk_fixtures distann_physical_graph_record`
- Expected cited result: `2 passed; 0 failed`

### `format-check-seq-03.log`

- Command: `cargo fmt --all -- --check`
- Expected cited result: exit status 0 (stable-rustfmt nightly-option warnings
  are non-failures)

### `clippy-seq-03.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Expected cited result: nonzero only for the same five pre-existing failures
  recorded under seq-02; no failure in a Graph V2 touched line. The first local
  attempt found an MSRV error in `Option::is_none_or`; code `9b13d2aca` fixed it
  before these artifacts were regenerated.

## Seq-02 descriptor artifacts

### `row-layout-tests-seq-02.log`

- Command: `cargo test --no-default-features --features pg18 row_layout::tests`
- Expected cited result: `5 passed; 0 failed`

### `row-layout-reloption-test-seq-02.log`

- Command: `cargo test --no-default-features --features pg18 hot_payload_attnums_and_layout_are_canonical`
- Expected cited result: `1 passed; 0 failed`

### `format-check-seq-02.log`

- Command: `cargo fmt --all -- --check`
- Expected cited result: exit status 0 (the host's stable-rustfmt warnings about
  nightly-only import grouping are non-failures)

### `clippy-seq-02.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Expected cited result: nonzero because of five pre-existing failures outside
  the touched files: `ambuild.rs:139`, `generation_descriptor.rs:798`,
  `head_sample.rs:1818`, `remote_endpoint.rs:1069`, and
  `ec_distann_physical_lifecycle.rs:7951`.
- Checkpoint result: no clippy failure in `row_layout.rs`, `row_schema.rs`, or
  `options.rs`; reviewer seq-01's new `options.rs:1652` failure is absent.

All commands use the host's shared `CARGO_TARGET_DIR`; no runtime output is
written under the repository `target/` directory.

The seq-01 logs without a suffix remain the immutable validation artifacts for
code checkpoint `ef558a669`; seq-02 artifacts belong to `8faac4bad`.
