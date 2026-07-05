# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg -n "UnlockReleaseBuffer|BufferGetPage|BufferGetPageSize|LockBuffer\\(" src/am/ec_hnsw/scan.rs`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `3705` entries.
- Baseline movement for the code slice: `3711 -> 3705`.
- `bash scripts/unsafe_baseline_report.sh`: `src/am/ec_hnsw/scan.rs`
  dropped from `267` to `261` entries.
- `rg -n "UnlockReleaseBuffer|BufferGetPage|BufferGetPageSize|LockBuffer\\(" src/am/ec_hnsw/scan.rs`:
  remaining matches are `LockBuffer` at lines `3041` and `3043`, the separate
  graph-prefetch path called out for a later design slice. The linear fallback
  selector no longer has raw `LockBuffer`, `UnlockReleaseBuffer`,
  `BufferGetPage`, or `BufferGetPageSize` matches.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed; output included the existing PG18 C header unused-parameter warnings
  and the existing unused re-export warning in `src/am/mod.rs`.
