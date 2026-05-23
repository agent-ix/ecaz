# Task 50 Review Request: Read-Stream Prefetch Helper

Code commit: `278c72a77d69c2003ad3559b18ac9cd85aeb3fdc`

This packet advances P9, read stream and prefetch contracts, across
production-priority heap-rerank paths.

## What Changed

`src/am/common/stream.rs` now owns a safe `prefetch_relation_blocks` helper.
It centralizes the PG18 `read_stream_begin_relation` /
`read_stream_next_buffer` / `read_stream_end` loop and the non-PG18
`PrefetchBuffer` fallback for block-sequence heap prefetches.

The duplicated prefetch loops were removed from:

- `src/am/ec_ivf/scan.rs`
- `src/am/ec_spire/scan/relation.rs`
- `src/am/ec_diskann/routine.rs`

Those callers now pass candidate heap block numbers into the common stream
contract.

## Unsafe Movement

Packet-local evidence:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-check.log`

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/common/stream.rs` | 4 | 9 |
| `src/am/ec_ivf/scan.rs` | 68 | 62 |
| `src/am/ec_spire/scan/relation.rs` | 25 | 18 |
| `src/am/ec_diskann/routine.rs` | 58 | 56 |

Overall current `src/` direct unsafe count: `2406` blocks across `131` files.

Ledger check:

```text
ledger covers 2406 current unsafe rows
```

## Validation

Passed:

```text
cargo check --all-targets --no-default-features --features pg18,bench
```

The compile log still reports the existing unused-import warning in
`src/am/mod.rs`; this packet does not address that unrelated warning.

No runtime benchmark was run. This moves prefetch plumbing behind a common
contract and does not change scoring math, candidate ordering, payload bytes,
or WAL order.
