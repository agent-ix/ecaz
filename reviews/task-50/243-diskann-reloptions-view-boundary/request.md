# Task 50 Review Request: DiskANN Reloptions View Boundary

## Summary

Reworked `src/am/ec_diskann/options.rs` so DiskANN reloption decoding uses a
typed `TqDiskannReloptionsView` instead of a free unsafe string-reloption
helper.

The view now owns the relation-options layout contract:

- opens the relation-owned `rd_options` pointer,
- casts it to `TqDiskannReloptions`,
- reads the `storage_format` string offset,
- converts the view into `TqDiskannOptions`.

This mirrors the IVF/HNSW reloptions view shape and keeps pointer/offset
interpretation scoped to the layout that validates those offsets.

## Unsafe Burndown

- `src/am/ec_diskann/options.rs` unsafe grep count: `7 -> 7`
- repository `src` unsafe grep count: `2478 -> 2478`
- standalone `unsafe fn read_string_reloption` was removed; remaining
  `read_string_reloption` is a safe method on `TqDiskannReloptionsView`.

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_diskann/options.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib ec_diskann --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Review Focus

Please verify that this preserves DiskANN reloption semantics while moving the
string-offset pointer work behind the typed relation-options view.
