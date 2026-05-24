# Task 50 Review Request: Heap Slot Reader Rollout

Code commit: `590c57f61622c09fde10b5154be160d7d91cde00`

This packet advances the comprehensive unsafe burndown plan:

- Program: P5, heap source / tuple slot / snapshot / scorer contracts.
- Wave/tranche: Wave 2 SPIRE and IVF/RaBitQ production fanout, plus DiskANN
  heap-vector rerank/backlink/vacuum call-site rollout.

## What Changed

`src/am/common/heap_slot.rs` now exposes `HeapSlotReader`, a typed reader that
binds heap relation, snapshot, tuple slot, and AM label once at the boundary.
Its safe methods own slot clearing, heap row fetch, attribute materialization,
and non-null datum lookup.

Call sites were moved from direct raw-slot fetch/datum unsafe blocks to the
reader in:

- `src/am/ec_ivf/scan.rs`
- `src/am/ec_spire/scan/relation.rs`
- `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`
- `src/am/ec_spire/update/materialization.rs`
- `src/am/ec_diskann/routine.rs`
- `src/am/ec_diskann/scan_state.rs`
- `src/am/ec_hnsw/source.rs`

This intentionally centralizes additional unsafe inside the common heap-slot
contract while deleting caller unsafe in SPIRE, IVF, and DiskANN rerank/source
paths.

## Unsafe Movement

Packet-local evidence:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-check.log`

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/common/heap_slot.rs` | 7 | 13 |
| `src/am/ec_hnsw/source.rs` | 51 | 53 |
| `src/am/ec_ivf/scan.rs` | 69 | 68 |
| `src/am/ec_diskann/routine.rs` | 64 | 58 |
| `src/am/ec_diskann/scan_state.rs` | 20 | 20 |
| `src/am/ec_spire/scan/relation.rs` | 29 | 25 |
| `src/am/ec_spire/update/materialization.rs` | 1 | 1 |
| `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 48 | 48 |

Overall current `src/` direct unsafe count: `2442` blocks across `131` files.

Ledger check:

```text
ledger covers 2442 current unsafe rows
```

## Validation

Passed:

```text
cargo check --all-targets --no-default-features --features pg18,bench
```

The compile log still reports the existing unused-import warning in
`src/am/mod.rs`; this packet does not address that unrelated warning.

No runtime benchmark was run. This slice moves heap slot access behind a typed
helper and does not change scoring math, candidate ordering, persisted payloads,
or WAL mutation order.
