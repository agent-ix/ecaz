# Task 50 Review Request: HNSW Shared Metadata/Read Unsafe Reduction

## Summary

This packet handles `src/am/ec_hnsw/shared.rs`, the HNSW shared top-15 file.

Code commit:

- `7e719f6f Reduce HNSW shared unsafe blocks`

The change consolidates repeated unsafe regions around metadata WAL rewrites, main-fork block-count reads, and shared-lock main-fork buffer opens.

## Unsafe Count Status

Counts are from `artifacts/block-count-after.log`.

| File | Task 50 start | Follow-up count | Target | Status |
| --- | ---: | ---: | ---: | --- |
| `src/am/ec_hnsw/shared.rs` | 73 | 50 | <=51 | met |

## Review Notes

- Metadata page initialization/update semantics are unchanged: the registered page is still initialized with the encoded metadata special area before `GenericXLogTxn::finish`.
- Block-count reads now go through one private helper used by diagnostics, vacuum counting, and debug surfaces.
- Data-page reads now share one helper that preserves the same relation/block/mode/lock arguments at each call site.

## Validation

- `make unsafe-block-count PATHS='src/am/ec_hnsw/shared.rs'` passed with count 50.
- `rustfmt --edition 2021 --check src/am/ec_hnsw/shared.rs` passed, with only existing stable-rustfmt warnings about unstable import options.
- `git diff --check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.

No benchmark result is claimed in this packet. The slice changes unsafe ownership structure but not HNSW metadata encoding, WAL ordering, graph traversal, or candidate selection.
