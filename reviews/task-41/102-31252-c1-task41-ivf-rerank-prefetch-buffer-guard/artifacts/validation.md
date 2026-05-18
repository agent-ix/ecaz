# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg -n "ReleaseBuffer|read_stream_next_buffer|PinnedBufferGuard" src/am/ec_ivf/scan.rs`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `3839` entries.
- Baseline movement for the code slice: `3840 -> 3839`.
- `bash scripts/unsafe_baseline_report.sh`: `src/am/ec_ivf/scan.rs`
  dropped to `102` entries in the top-file list.
- `rg -n "ReleaseBuffer|read_stream_next_buffer|PinnedBufferGuard" src/am/ec_ivf/scan.rs`:
  no direct `ReleaseBuffer` match remains; `read_stream_next_buffer` is followed
  by `PinnedBufferGuard::from_pinned`.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed; output included the existing PG18 C header unused-parameter warnings
  and the existing unused re-export warning in `src/am/mod.rs`.
