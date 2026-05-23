# Task 50 Review Request: SPIRE Reloptions View Boundary

## Summary

Reworked `src/am/ec_spire/options/mod.rs` so SPIRE reloption decoding uses a
typed `EcSpireReloptionsView` instead of a free `rd_options` string-offset
helper.

The view now owns:

- relation-owned reloptions blob pointer,
- `EcSpireReloptions` layout reference,
- string-offset decoding for `storage_format`, `quantizer`,
  `source_identity`, `local_store_tablespaces`, and `nprobe_per_level`,
- validation and conversion into `EcSpireOptions`.

This completes the same P7 reloptions boundary pattern already applied to IVF,
HNSW, and DiskANN without adding new unsafe lines.

## Unsafe Burndown

- `src/am/ec_spire/options/mod.rs` unsafe grep count: `7 -> 7`
- repository `src` unsafe grep count: `2443 -> 2443`
- standalone `fn read_string_reloption` was removed; remaining
  `read_string_reloption` is a safe method on `EcSpireReloptionsView`.

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_spire/options/mod.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib ec_spire --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Review Focus

Please verify this preserves SPIRE reloption parsing/validation semantics while
moving raw reloptions string-offset handling behind the typed view.
