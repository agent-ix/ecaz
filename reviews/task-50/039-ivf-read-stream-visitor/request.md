# Task 50 Review Request: IVF Read-Stream Visitor

Code commit: `4b17638f85e19deff77f3cd3067737f66f2582c7`

This packet extends P9 from prefetch-only helpers to read-stream visitors used
by IVF posting-page scans.

## What Changed

`src/am/common/stream.rs` now owns PG18 read-stream visitor helpers:

- `visit_relation_linear_read_stream`
- `visit_relation_block_sequence_read_stream`

The helpers own `read_stream_begin_relation`, `read_stream_next_buffer`,
`LockedBufferGuard::lock_pinned`, per-buffer block-number extraction, and
`read_stream_end`. `read_stream_end` is now guarded by `Drop`, so early visitor
errors do not need local manual cleanup.

`src/am/ec_ivf/page.rs` now uses these helpers for:

- list-range posting scans;
- posting block-sequence scans;
- posting-ref block-sequence scans.

## Unsafe Movement

Packet-local evidence:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-check.log`

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/common/stream.rs` | 9 | 16 |
| `src/am/ec_ivf/page.rs` | 90 | 72 |

Overall current `src/` direct unsafe count: `2381` blocks across `131` files.

Ledger check:

```text
ledger covers 2381 current unsafe rows
```

## Validation

Passed:

```text
cargo check --all-targets --no-default-features --features pg18,bench
```

The compile log still reports the existing unused-import warning in
`src/am/mod.rs`; this packet does not address that unrelated warning.

No runtime benchmark was run. This centralizes read-stream traversal and buffer
pin/lock handling without changing posting tuple decoding, candidate ordering,
scoring math, payload bytes, or WAL order.
