# Task 50 Review Request: Loaded Tuple Slot Reader Cleanup

Code commit: `3165448cac1dec24f333eadaa1b17ff6208362db`

This packet continues the P5 heap source / tuple slot contract work by splitting
the existing heap-fetch reader into a reusable loaded-slot reader and removing
now-obsolete raw helper surfaces.

## What Changed

`src/am/common/heap_slot.rs` now has two typed contracts:

- `TupleSlotReader`: owns a loaded `TupleTableSlot` boundary for clearing and
  required datum materialization.
- `HeapSlotReader`: owns heap relation + snapshot + tuple slot for row-version
  fetches and delegates loaded-slot reads to `TupleSlotReader`.

The old free helper functions for raw slot clear/fetch/datum lookup were
deleted. Dead DiskANN scan-state and HNSW source raw helper wrappers were also
deleted.

`src/am/ec_hnsw/build.rs` now uses `TupleSlotReader` for the heap scan slot
that PostgreSQL fills with `heap_getnextslot`, so the build loop no longer
calls the old raw `source::required_slot_datum` helper.

## Unsafe Movement

Packet-local evidence:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-check.log`

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/common/heap_slot.rs` | 13 | 7 |
| `src/am/ec_hnsw/build.rs` | 33 | 32 |
| `src/am/ec_hnsw/source.rs` | 47 | 46 |
| `src/am/ec_diskann/scan_state.rs` | 20 | 18 |

Overall current `src/` direct unsafe count: `2416` blocks across `131` files.

Ledger check:

```text
ledger covers 2416 current unsafe rows
```

## Validation

Passed:

```text
cargo check --all-targets --no-default-features --features pg18,bench
```

The compile log still reports the existing unused-import warning in
`src/am/mod.rs`; this packet does not address that unrelated warning.

No runtime benchmark was run. This is a slot-access contract cleanup and does
not change candidate ordering, scoring math, persisted payloads, or WAL order.
