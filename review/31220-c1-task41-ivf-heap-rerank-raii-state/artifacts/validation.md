# Validation

Head SHA: `bd07d424`

## Commands

```text
cargo fmt
cargo check --all-targets --no-default-features --features pg18,bench
bash scripts/check_unsafe_comments.sh --update-baseline
git diff --check
bash scripts/check_unsafe_comments.sh
bash scripts/unsafe_baseline_report.sh
make fmt-check
cargo check --all-targets --no-default-features --features pg18,bench
```

## Key Results

```text
wrote scripts/unsafe_comment_baseline.txt with 4102 entries
```

```text
unsafe comment baseline
file: scripts/unsafe_comment_baseline.txt
entries: 4102
files: 106
```

`make fmt-check` completed successfully. Rustfmt emitted the existing stable
toolchain warnings for unstable `imports_granularity` and `group_imports`
settings.

`cargo check --all-targets --no-default-features --features pg18,bench`
completed successfully before and after refreshing the baseline. It emitted the
existing PG18 C-header warnings and the existing unused re-export warning in
`src/am/mod.rs`.

No `cargo pgrx test pg18` run was performed for this ownership refactor. Review
should focus on the boxed state lifetime across rescan/endscan and `pgrx::error!`
unwind behavior.
