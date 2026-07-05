# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `4015` entries.
- Baseline movement for the code slice: `4026 -> 4015`.
- `bash scripts/unsafe_baseline_report.sh`: `src/am/ec_diskann/scan_state.rs`
  dropped to `23` entries in the top-file list.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean after refreshing line numbers.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed after adding explicit `Result<_, String>` annotations; output included
  the existing PG18 C header unused-parameter warnings and the existing unused
  re-export warning in `src/am/mod.rs`.
