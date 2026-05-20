# Task 50 Review Request: HNSW Dead Heap Helper Removal

Code commit: `8ca545d3c4185a5a9fb5b299b4a1db4ef593f953`

This packet continues P5 after the HNSW heap-slot reader rollout by deleting
obsolete raw-slot HNSW helper entry points.

## What Changed

`src/am/ec_hnsw/source.rs` no longer exposes the old raw-pointer helpers for:

- heap row fetch;
- heap-row source visitor;
- indexed ecvector slot visitor.

All current callers use the `HeapSlotReader` variants landed in packets 033 and
034. The only remaining raw slot datum helper in this file is still used by the
HNSW build heap-scan path and remains open for a later loaded-slot reader slice.

## Unsafe Movement

Packet-local evidence:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-check.log`

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_hnsw/source.rs` | 53 | 47 |
| `src/am/common/heap_slot.rs` | 13 | 13 |

Overall current `src/` direct unsafe count: `2426` blocks across `131` files.

Ledger check:

```text
ledger covers 2426 current unsafe rows
```

## Validation

Passed:

```text
cargo check --all-targets --no-default-features --features pg18,bench
```

The compile log still reports the existing unused-import warning in
`src/am/mod.rs`; this packet does not address that unrelated warning.

No runtime benchmark was run. This is dead helper surface deletion only.
