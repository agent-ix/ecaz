# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg -n -U "#\\[pg_guard\\]\\nunsafe extern \\\"C-unwind\\\" fn ec_.*amvalidate|unsafe extern \\\"C-unwind\\\" fn ec_.*amvalidate" src/am/ec_hnsw/routine.rs src/am/ec_ivf/routine.rs src/am/ec_diskann/routine.rs src/am/ec_spire/routine.rs`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `3701` entries.
- Baseline movement for the code slice: `3701 -> 3701`; only DiskANN line
  numbers moved after adding `#[pg_guard]`.
- The targeted `rg -U` command shows each `ec_*_amvalidate` callback has an
  immediately preceding `#[pg_guard]` attribute.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed; output included the existing PG18 C header unused-parameter warnings
  and the existing unused re-export warning in `src/am/mod.rs`.
