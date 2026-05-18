# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg -n "append_(heap_tuple|turbo_hot_cold_tuple|pq_fastscan_tuple)_to_new_page|ReadBufferExtended|UnlockReleaseBuffer|BufferGetPageSize|BufferGetBlockNumber" src/am/ec_hnsw/insert.rs`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `3751` entries.
- Baseline movement for the code slice: `3763 -> 3751`.
- `bash scripts/unsafe_baseline_report.sh`: `src/am/ec_hnsw/insert.rs`
  dropped from `176` to `164` entries.
- `rg -n "append_(heap_tuple|turbo_hot_cold_tuple|pq_fastscan_tuple)_to_new_page|ReadBufferExtended|UnlockReleaseBuffer|BufferGetPageSize|BufferGetBlockNumber" src/am/ec_hnsw/insert.rs`:
  the three fallback `*_to_new_page` helper bodies no longer contain direct
  PostgreSQL buffer API calls; remaining matches are the main append paths left
  for follow-up slices.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed; output included the existing PG18 C header unused-parameter warnings
  and the existing unused re-export warning in `src/am/mod.rs`.
