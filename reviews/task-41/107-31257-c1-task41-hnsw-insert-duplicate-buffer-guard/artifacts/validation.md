# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg -n "coalesce_duplicate_(heap_tid|turbo_hot_heap_tid|grouped_heap_tid)|ReadBufferExtended|LockBuffer|UnlockReleaseBuffer|BufferGetPageSize" src/am/ec_hnsw/insert.rs`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `3782` entries.
- Baseline movement for the code slice: `3797 -> 3782`.
- `bash scripts/unsafe_baseline_report.sh`: `src/am/ec_hnsw/insert.rs`
  dropped from `210` to `195` entries.
- `rg -n "coalesce_duplicate_(heap_tid|turbo_hot_heap_tid|grouped_heap_tid)|ReadBufferExtended|LockBuffer|UnlockReleaseBuffer|BufferGetPageSize" src/am/ec_hnsw/insert.rs`:
  the three duplicate-coalescing helper bodies no longer contain direct
  PostgreSQL buffer API calls; remaining matches are other insert paths left
  for follow-up slices.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed; output included the existing PG18 C header unused-parameter warnings
  and the existing unused re-export warning in `src/am/mod.rs`.
