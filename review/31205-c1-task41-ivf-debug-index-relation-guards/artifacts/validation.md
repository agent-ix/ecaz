# Validation

Head SHA: `aee30476e23b28063b31c96b27ece01e8147430e`

Commands run:

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Results:

- Unsafe baseline updated from `4165` entries to `4146` entries.
- `git diff --check` passed.
- `bash scripts/check_unsafe_comments.sh` passed.
- `make fmt-check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.

Known non-fatal output:

- stable rustfmt warnings for unstable `imports_granularity` and `group_imports`
- PG18 C header unused-parameter warnings
- existing unused re-export warning in `src/am/mod.rs`
