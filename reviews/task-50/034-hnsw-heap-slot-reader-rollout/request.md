# Task 50 Review Request: HNSW Heap Slot Reader Rollout

Code commit: `6d38b4e1ff6848517151b7f5ab814d677e50f920`

This packet continues the comprehensive unsafe burndown plan:

- Program: P5, heap source / tuple slot / snapshot / scorer contracts.
- Wave/tranche: Wave 3 HNSW scan and scan-debug handle/page/scorer rollout,
  with the P5 heap-source scorer portion applied to HNSW insert, vacuum, and
  grouped scan rerank scoring.

## What Changed

The `HeapSlotReader` contract from packet 033 is now used by HNSW source-backed
scoring paths:

- `src/am/ec_hnsw/insert.rs`: insert-time source-column scoring loads and
  averages heap source vectors through `HeapSlotReader`.
- `src/am/ec_hnsw/vacuum.rs`: vacuum repair source-backed scoring loads graph
  element representatives through `HeapSlotReader`.
- `src/am/ec_hnsw/scan.rs`: grouped heap-f32 rerank fetches rows and reads
  source datums through `HeapSlotReader`.

This deletes call-site unsafe blocks for `with_source_from_heap_row`,
`fetch_heap_row_version`, `required_slot_datum`, and direct slot clearing in
these HNSW scorer paths.

## Unsafe Movement

Packet-local evidence:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-check.log`

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_hnsw/insert.rs` | 93 | 86 |
| `src/am/ec_hnsw/vacuum.rs` | 68 | 66 |
| `src/am/ec_hnsw/scan.rs` | 158 | 157 |
| `src/am/ec_hnsw/source.rs` | 53 | 53 |
| `src/am/common/heap_slot.rs` | 13 | 13 |

Overall current `src/` direct unsafe count: `2432` blocks across `131` files.

Ledger check:

```text
ledger covers 2432 current unsafe rows
```

## Validation

Passed:

```text
cargo check --all-targets --no-default-features --features pg18,bench
```

The compile log still reports the existing unused-import warning in
`src/am/mod.rs`; this packet does not address that unrelated warning.

No runtime benchmark was run. This slice preserves scoring math and candidate
ordering; it changes only the heap slot access boundary used by HNSW source
scorers.
