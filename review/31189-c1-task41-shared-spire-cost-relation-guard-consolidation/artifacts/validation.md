# Validation

Head SHA: `4e1e50541313ab6e415f46c677b3ae9974554fd6`

Timestamp: `2026-05-17T05:36:57Z`

## Commands

### `cargo fmt`

Passed.

Known stable rustfmt warnings were emitted for unstable formatting options:

- `imports_granularity = Crate`
- `group_imports = StdExternalCrate`

### `bash scripts/check_unsafe_comments.sh --update-baseline`

Passed.

Key line:

```text
wrote scripts/unsafe_comment_baseline.txt with 4254 entries
```

### `git diff --check`

Passed with no output.

### `bash scripts/check_unsafe_comments.sh`

Passed with no output.

### `make fmt-check`

Passed.

Known stable rustfmt warnings were emitted for unstable formatting options.

### `bash scripts/unsafe_baseline_report.sh`

Passed.

Key lines:

```text
unsafe comment baseline
file: scripts/unsafe_comment_baseline.txt
entries: 4254
files: 106
```

### `cargo check --all-targets --no-default-features --features pg18,bench`

Passed.

Known warnings:

- PG18 C header unused-parameter warnings from PostgreSQL headers.
- Existing unused re-export warning in `src/am/mod.rs`.

Key line:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.20s
```
