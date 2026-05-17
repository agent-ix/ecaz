# Validation Log

## Code Commit

`5a0649136cab9e44784dbdae38f699acb76bb709`

## Commands

### cargo fmt

Result: passed.

Notes: emitted the existing stable-rustfmt warnings:

```text
Warning: can't set `imports_granularity = Crate`, unstable features are only available in nightly channel.
Warning: can't set `group_imports = StdExternalCrate`, unstable features are only available in nightly channel.
```

### bash scripts/check_unsafe_comments.sh --update-baseline

Result: passed.

```text
wrote scripts/unsafe_comment_baseline.txt with 4256 entries
```

### git diff --check

Result: passed with no output.

### bash scripts/check_unsafe_comments.sh

Result: passed with no output.

### make fmt-check

Result: passed.

Notes: emitted the same stable-rustfmt warnings listed above.

### bash scripts/unsafe_baseline_report.sh

Result: passed.

```text
unsafe comment baseline
file: scripts/unsafe_comment_baseline.txt
entries: 4256
files: 106

top files
433 src/am/ec_hnsw/scan_debug.rs
273 src/am/ec_hnsw/scan.rs
224 src/am/ec_ivf/page.rs
214 src/am/ec_hnsw/build_parallel.rs
212 src/am/ec_hnsw/insert.rs
177 src/lib.rs
162 src/am/ec_hnsw/vacuum.rs
158 src/am/ec_spire/dml_frontdoor/mod.rs
137 src/am/ec_ivf/scan.rs
137 src/am/ec_diskann/routine.rs
111 src/am/ec_spire/page.rs
106 src/am/ec_hnsw/shared.rs
```

### cargo check --all-targets --no-default-features --features pg18,bench

Result: passed.

Notes: emitted existing PG18 C-header warnings and the existing unused re-export warning in `src/am/mod.rs`.

```text
Finished `dev` profile [unoptimized + debuginfo] target(s)
```
