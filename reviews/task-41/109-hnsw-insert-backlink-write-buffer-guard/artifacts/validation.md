# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg -n "add_backlinks_on_page|ReadBufferExtended|LockBuffer|UnlockReleaseBuffer|BufferGetPageSize" src/am/ec_hnsw/insert.rs`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `3763` entries.
- Baseline movement for the code slice: `3767 -> 3763`.
- `bash scripts/unsafe_baseline_report.sh`: `src/am/ec_hnsw/insert.rs`
  dropped from `180` to `176` entries.
- `rg -n "add_backlinks_on_page|ReadBufferExtended|LockBuffer|UnlockReleaseBuffer|BufferGetPageSize" src/am/ec_hnsw/insert.rs`:
  `add_backlinks_on_page` no longer contains direct PostgreSQL buffer API
  calls; remaining matches are append paths left for follow-up slices.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed; output included the existing PG18 C header unused-parameter warnings
  and the existing unused re-export warning in `src/am/mod.rs`.
