# Artifact Manifest

Task bucket: `reviews/task-41/`

Packet: `reviews/task-41/121-ffi-audit-inventory/`

Head SHA: `60d20e72c21d8877cf1aa0be6399948a7ba02264`

Timestamp: `2026-05-18T04:21:14Z`

Lane: Task 41 invariant #1, FFI callback guard inventory

Fixture / storage format / rerank mode: not applicable; static audit lane

Index isolation: not applicable; no benchmark run

## Commands

### `python3 scripts/ffi_audit.py --write`

Result: passed.

Key line:

```text
wrote docs/ffi-inventory.md
```

### `make ffi-audit`

Result: passed.

Key lines:

```text
python3 scripts/ffi_audit.py --check
ffi audit passed: 101 direct C ABI functions, 288 pgrx-managed SQL entrypoints
```

### `cargo fmt`

Result: passed.

Known warnings: stable rustfmt reports `imports_granularity` and
`group_imports` are nightly-only.

### `bash scripts/check_unsafe_comments.sh --update-baseline`

Result: passed.

Key line:

```text
wrote scripts/unsafe_comment_baseline.txt with 3690 entries
```

### `git diff --check`

Result: passed.

### `bash scripts/check_unsafe_comments.sh`

Result: passed.

### `bash scripts/unsafe_baseline_report.sh`

Result: passed.

Key lines:

```text
entries: 3690
files: 106
```

### `make fmt-check`

Result: passed.

Known warnings: stable rustfmt reports `imports_granularity` and
`group_imports` are nightly-only.

### `cargo check --all-targets --no-default-features --features pg18,bench`

Result: passed.

Known warnings:

- PG18 C headers emit unused-parameter warnings.
- Existing unused re-export warning in `src/am/mod.rs`.
