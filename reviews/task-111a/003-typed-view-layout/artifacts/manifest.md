# Artifact Manifest: Task 111a Typed Dense Layout

- head SHA: `8f3979f8e`
- task bucket: `reviews/task-111a/`
- packet path: `reviews/task-111a/003-typed-view-layout/`
- timestamp: `2026-06-17`
- isolated one-index-per-table vs shared-table surface: not applicable; this
  packet contains focused Rust library validation, not benchmark data.

## Artifacts

### `cargo-check-lib.log`

- command: `cargo check -q --lib`
- result: pass
- key output: no output; command exited 0.

### `cargo-test-dense-posting-aligned-block.log`

- command:
  `cargo test -q dense_posting_aligned_block_roundtrip_exposes_native_views --lib`
- result: pass
- key output:

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2115 filtered out; finished in 0.00s
```

### `cargo-test-dense-posting-packed.log`

- command: `cargo test -q dense_posting_packed --lib`
- result: pass
- key output:

```text
running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2114 filtered out; finished in 0.00s
```

### `cargo-test-build-state-packed-continuations.log`

- command:
  `cargo test -q build_state_splits_packed_dense_payloads_into_continuations --lib`
- result: pass
- key output:

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2115 filtered out; finished in 0.03s
```

### `cargo-test-dense-posting-block-roundtrip.log`

- command:
  `cargo test -q dense_posting_block_roundtrip_preserves_scan_arrays --lib`
- result: pass
- key output:

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2115 filtered out; finished in 0.00s
```
