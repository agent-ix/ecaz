# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg -n "ReadBufferExtended|LockBuffer|UnlockReleaseBuffer|ReleaseBuffer|BufferGetPageSize|BufferGetPage" src/am/ec_hnsw/vacuum.rs`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `3797` entries.
- Baseline movement for the code slice: `3817 -> 3797`.
- `bash scripts/unsafe_baseline_report.sh`: `src/am/ec_hnsw/vacuum.rs`
  dropped from `157` to `137` entries.
- `rg -n "ReadBufferExtended|LockBuffer|UnlockReleaseBuffer|ReleaseBuffer|BufferGetPageSize|BufferGetPage" src/am/ec_hnsw/vacuum.rs`:
  remaining matches are the exclusive rewrite/WAL mutation paths and explicit
  exclusive reopen points left for a follow-up slice.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed; output included the existing PG18 C header unused-parameter warnings
  and the existing unused re-export warning in `src/am/mod.rs`.
