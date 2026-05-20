# Task 50: HNSW Shared Read Stream Visitor

## Summary

This packet continues the Task 50 P9 read-stream/prefetch contract rollout.
`ec_hnsw` live tuple counting now uses the shared
`visit_relation_linear_read_stream` helper instead of owning a local PG18
`read_stream_begin_relation` / `read_stream_next_buffer` /
`read_stream_end` loop.

The direct unsafe reduction is intentionally structural: read stream lifetime,
reset, buffer acquisition, per-buffer block-number payloads, and stream cleanup
are centralized in `src/am/common/stream.rs`; the HNSW caller now supplies only
the tuple-counting visitor.

## Code Under Review

- code commit: `dc07382fc235502ee9d2f5de3d44615b4f2c2e2c`
- previous packet baseline: `b44167f3d9075ea3a690e7fcfd6128ab05521c58`
- touched file: `src/am/ec_hnsw/shared.rs`

## Unsafe Movement

Packet-local count artifacts:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`

Direct unsafe blocks in the touched/read-stream files:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_hnsw/shared.rs` | 50 | 44 |
| `src/am/common/stream.rs` | 16 | 16 |

Current `src/` ledger after this packet:

- `2375` direct unsafe blocks
- `131` files
- ledger check: `ledger covers 2375 current unsafe rows`

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

Benchmarks were not run for this slice. This change only replaces local
read-stream traversal plumbing with the already-existing shared visitor helper;
it does not change scoring, ordering, tuple payload format, WAL, or search
semantics.
