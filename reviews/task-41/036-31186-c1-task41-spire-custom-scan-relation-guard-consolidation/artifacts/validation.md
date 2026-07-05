# Validation Log

## Code Commit

`3bd20ba60e1cd0b9a268b9a22661befdc97d609a`

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
```

### cargo check --all-targets --no-default-features --features pg18,bench

Result: passed.

Notes: emitted existing PG18 C-header warnings and the existing unused re-export warning in `src/am/mod.rs`.

```text
Finished `dev` profile [unoptimized + debuginfo] target(s)
```
