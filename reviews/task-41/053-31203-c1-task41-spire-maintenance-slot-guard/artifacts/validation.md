# Validation

Head SHA: `b619d27686de098646d6ce5eadcb0ea96f0a2637`

Commands run:

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Results:

- Unsafe baseline updated from `4174` entries to `4171` entries.
- `git diff --check` passed.
- `bash scripts/check_unsafe_comments.sh` passed.
- `make fmt-check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.

Known non-fatal output:

- stable rustfmt warnings for unstable `imports_granularity` and `group_imports`
- PG18 C header unused-parameter warnings
- existing unused re-export warning in `src/am/mod.rs`
