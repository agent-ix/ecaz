# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg -n "ReleaseBuffer\\(|read_stream_next_buffer|PinnedBufferGuard::from_pinned" src/am/ec_spire/storage/relation_store.rs`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `3701` entries.
- Baseline movement for the code slice: `3701 -> 3701`; only line numbers in
  `src/am/ec_spire/storage/relation_store.rs` moved.
- Targeted `rg` shows `read_stream_next_buffer` followed by
  `PinnedBufferGuard::from_pinned`, with no remaining `ReleaseBuffer` match in
  `src/am/ec_spire/storage/relation_store.rs`.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed; output included the existing PG18 C header unused-parameter warnings
  and the existing unused re-export warning in `src/am/mod.rs`.
