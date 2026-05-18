# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg -n "ReleaseBuffer\\(|LockBuffer\\(|UnlockReleaseBuffer|BufferGetPage|BufferGetPageSize" src/am/ec_hnsw/scan.rs src/am/ec_hnsw/graph.rs`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `3701` entries.
- Baseline movement for the code slice: `3705 -> 3701`.
- `bash scripts/unsafe_baseline_report.sh`: `src/am/ec_hnsw/scan.rs`
  dropped from `261` to `258` entries and `src/am/ec_hnsw/graph.rs`
  dropped from `58` to `56` entries.
- `rg -n "ReleaseBuffer\\(|LockBuffer\\(|UnlockReleaseBuffer|BufferGetPage|BufferGetPageSize" src/am/ec_hnsw/scan.rs src/am/ec_hnsw/graph.rs`:
  no matches.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed; output included the existing PG18 C header unused-parameter warnings
  and the existing unused re-export warning in `src/am/mod.rs`.
