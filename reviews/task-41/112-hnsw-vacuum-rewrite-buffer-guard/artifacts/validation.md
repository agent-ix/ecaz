# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg -n "rewrite_page_pass[12]|ReadBufferExtended|LockBuffer|UnlockReleaseBuffer|BufferGetPageSize|BufferGetPage" src/am/ec_hnsw/vacuum.rs`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `3721` entries.
- Baseline movement for the code slice: `3733 -> 3721`.
- `bash scripts/unsafe_baseline_report.sh`: `src/am/ec_hnsw/vacuum.rs`
  dropped from `137` to `125` entries.
- `rg -n "rewrite_page_pass[12]|ReadBufferExtended|LockBuffer|UnlockReleaseBuffer|BufferGetPageSize|BufferGetPage" src/am/ec_hnsw/vacuum.rs`:
  `rewrite_page_pass1` and `rewrite_page_pass2` no longer contain direct
  PostgreSQL buffer API calls; remaining matches are other vacuum write paths
  left for follow-up slices.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed; output included the existing PG18 C header unused-parameter warnings
  and the existing unused re-export warning in `src/am/mod.rs`.
