# Task 50 Review Request: HNSW Insert Page Writer

## Summary

This packet handles `src/am/ec_hnsw/insert.rs`, one of the remaining top-15 HNSW files.

Code commit:

- `56263461 Reduce HNSW insert append page unsafe blocks`

The change adds a private `InsertPageWrite` guard for live-insert append pages. It owns the locked buffer, GenericXLog transaction, registered page pointer, page initialization, free-space check, tuple insertion, and finish path for scalar, TurboQuant V3, and PqFastScan append helpers.

## Unsafe Count Status

Counts are from `artifacts/block-count-after.log`.

| File | Task 50 start | Follow-up count | Target | Status |
| --- | ---: | ---: | ---: | --- |
| `src/am/ec_hnsw/insert.rs` | 133 | 93 | <=93 | met |

## Review Notes

- The append data path is unchanged: each format still writes the same neighbor, rerank, and hot/element payloads in the same order.
- The new guard keeps raw page/WAL operations under one invariant instead of re-opening unsafe blocks at every `PageAddItemExtended`, `PageInit`, `PageGetFreeSpace`, and GenericXLog finish site.
- The adapter dispatch wrappers now acknowledge the format/layout contract once per operation instead of once per match arm.

## Validation

- `make unsafe-block-count PATHS='src/am/ec_hnsw/insert.rs'` passed with count 93.
- `rustfmt --edition 2021 --check src/am/ec_hnsw/insert.rs` passed, with only existing stable-rustfmt warnings about unstable import options.
- `git diff --check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.

No benchmark result is claimed in this packet. The slice changes append-page ownership structure but not scoring, graph search, tuple payload bytes, or append ordering.
