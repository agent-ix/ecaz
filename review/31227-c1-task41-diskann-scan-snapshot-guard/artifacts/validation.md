# Validation

Head SHA: `d3ef03b5`

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
wrote scripts/unsafe_comment_baseline.txt with 4077 entries
```

```text
unsafe comment baseline
file: scripts/unsafe_comment_baseline.txt
entries: 4077
files: 106

top files
369 src/am/ec_hnsw/scan_debug.rs
268 src/am/ec_hnsw/scan.rs
224 src/am/ec_ivf/page.rs
214 src/am/ec_hnsw/build_parallel.rs
210 src/am/ec_hnsw/insert.rs
```

`make fmt-check` completed successfully. Rustfmt emitted the existing stable
toolchain warnings for unstable `imports_granularity` and `group_imports`
settings.

`cargo check --all-targets --no-default-features --features pg18,bench`
completed successfully. It emitted the existing PG18 C-header warnings and the
existing unused re-export warning in `src/am/mod.rs`.

No `cargo pgrx test pg18` run was performed for this focused ownership refactor.
Review should focus on registered-vs-borrowed snapshot lifetime.
