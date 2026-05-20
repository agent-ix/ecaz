# Task 50: HNSW Scan-Owned Read Stream Contract

## Summary

This packet continues the Task 50 P9 read-stream/prefetch contract rollout.
HNSW scan still owns persistent PG18 read streams in the scan opaque, but direct
reset and next-buffer handling now flows through shared helpers in
`src/am/common/stream.rs`.

The new common helpers cover:

- scan-owned read-stream reset with null-stream validation;
- pinned-buffer visitation for graph prefetch buffers;
- locked-buffer visitation for linear fallback scanning;
- typed optional per-buffer block metadata; and
- early stop for the linear fallback path when a visible result is selected.

This keeps scan-local stream reuse intact while moving the raw PG18
`read_stream_reset` / `read_stream_next_buffer` boundary out of
`src/am/ec_hnsw/scan.rs`.

## Code Under Review

- code commit: `87276167cdb318126463257ceeb40fe66e9dee64`
- previous packet baseline: `1d8588622dde85092d764461c9d51bae33208958`
- touched files:
  - `src/am/common/stream.rs`
  - `src/am/ec_hnsw/scan.rs`

## Unsafe Movement

Packet-local count artifacts:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`

Direct unsafe blocks in the touched files:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 157 | 149 |
| `src/am/common/stream.rs` | 16 | 21 |

Net current `src/` ledger after this packet:

- `2372` direct unsafe blocks
- `131` files
- ledger check: `ledger covers 2372 current unsafe rows`

The common module count increases because this packet creates the shared
scan-owned read-stream boundary. The HNSW caller reduction is larger than that
increase, and the remaining raw read-stream operations are now localized to the
common stream contract.

## Validation

Packet-local validation artifacts:

- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`

Validation run:

```text
cargo check --all-targets --no-default-features --features pg18,bench
```

Result: passed. The log contains the existing unrelated
`src/am/mod.rs` unused-import warning.

Benchmarks were not run for this slice. The change preserves the existing
scan-owned stream reuse and only centralizes read-stream reset, buffer adoption,
locking, and per-buffer metadata handling.
