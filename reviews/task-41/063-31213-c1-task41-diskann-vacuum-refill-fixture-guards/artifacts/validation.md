# Validation

Head SHA: `c47ef9bf`

## Commands

```text
cargo fmt
bash scripts/check_unsafe_comments.sh --update-baseline
git diff --check
bash scripts/check_unsafe_comments.sh
bash scripts/unsafe_baseline_report.sh
make fmt-check
cargo check --all-targets --no-default-features --features pg18,bench
```

## Key Results

```text
wrote scripts/unsafe_comment_baseline.txt with 4129 entries
```

```text
unsafe comment baseline
file: scripts/unsafe_comment_baseline.txt
entries: 4129
files: 106
```

`make fmt-check` completed successfully. Rustfmt emitted the existing stable
toolchain warnings for unstable `imports_granularity` and `group_imports`
settings.

`cargo check --all-targets --no-default-features --features pg18,bench`
completed successfully. It emitted the existing PG18 C-header unused-parameter
warnings and the existing unused re-export warning in `src/am/mod.rs`.

No `cargo pgrx test pg18` run was performed for this fixture-only resource
migration. The changed helper is compiled by the `--all-targets` check.
