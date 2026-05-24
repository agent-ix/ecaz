# Task 50 Review Request: SPIRE Relation Store Prefetch

Code commit: `6c5b39cc9f8b316b11cb3b889a2e3506c9368e53`

This packet continues P9 read-stream and prefetch contract rollout into SPIRE
relation-backed object storage.

## What Changed

`src/am/ec_spire/storage/relation_store.rs` now uses
`crate::am::stream::prefetch_relation_blocks` for relation object prefetches.
The local PG18 read-stream helper and non-PG18 `PrefetchBuffer` loop were
deleted.

Because prefetching is now behind a safe common contract, these relation-store
methods no longer need to be unsafe:

- `prefetch_object_tuple`
- `prefetch_object_tuples`
- `prefetch_object_blocks`
- store-set `prefetch_object`
- store-set `prefetch_objects`

`src/am/ec_spire/storage.rs` dropped the now-unused parent `ptr` import.

## Unsafe Movement

Packet-local evidence:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-check.log`

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_spire/storage.rs` | 0 | 0 |
| `src/am/ec_spire/storage/relation_store.rs` | 52 | 38 |

Overall current `src/` direct unsafe count: `2392` blocks across `131` files.

Ledger check:

```text
ledger covers 2392 current unsafe rows
```

## Validation

Passed:

```text
cargo check --all-targets --no-default-features --features pg18,bench
```

The compile log still reports the existing unused-import warning in
`src/am/mod.rs`; this packet does not address that unrelated warning.

No runtime benchmark was run. This centralizes object prefetch plumbing without
changing candidate ordering, scoring math, payload bytes, or WAL order.
