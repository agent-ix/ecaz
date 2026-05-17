# Validation

Head SHA: `f5bcf0fd3dce31e55281829d43a97670d9a28696`

Commands run:

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Results:

- Unsafe baseline updated from `4141` entries to `4140` entries.
- `git diff --check` passed.
- `bash scripts/check_unsafe_comments.sh` passed.
- `make fmt-check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.

Known non-fatal output:

- stable rustfmt warnings for unstable `imports_granularity` and `group_imports`
- PG18 C header unused-parameter warnings
- existing unused re-export warning in `src/am/mod.rs`
