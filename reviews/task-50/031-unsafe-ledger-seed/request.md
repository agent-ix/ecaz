# Task 50 Unsafe Ledger Seed

This packet implements Wave 0 from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

It does not claim unsafe burndown completion. It creates the mechanism needed
to prevent blind cleanup:

- `scripts/unsafe_ledger.py generate`
- `scripts/unsafe_ledger.py check`
- `make unsafe-ledger`
- `make unsafe-ledger-check`
- packet-local `unsafe-ledger.jsonl`
- packet-local empty `residual-registry.jsonl`

## Results

`make unsafe-ledger` generated `2446` direct-unsafe ledger rows for current
`src/**/*.rs`.

`make unsafe-ledger-check` passed:

```text
ledger covers 2446 current unsafe rows
```

Every current row is assigned to a contract program from P1-P13. There are no
P0 unclassified rows in this seed.

| Program | Rows | Meaning |
| --- | ---: | --- |
| P1 | 28 | FFI and callback boundary contracts |
| P2 | 819 | PostgreSQL handle views |
| P3 | 247 | Buffer, page, and WAL transaction contracts |
| P4 | 300 | Page tuple and line-pointer views |
| P5 | 95 | Heap source, tuple slot, snapshot, and scorer contracts |
| P6 | 101 | Datum, varlena, vector, and quantized payload contracts |
| P7 | 48 | Reloptions and C string contracts |
| P8 | 141 | DSM, atomics, shared memory, and lock contracts |
| P9 | 29 | Read stream and prefetch contracts |
| P10 | 70 | Scan opaque and raw ownership contracts |
| P11 | 322 | Planner, node, list, and custom scan views |
| P12 | 61 | SIMD, quant, and raw memory kernels |
| P13 | 185 | Tests, debug exports, hardening, crates, vendor |

## Scope Notes

The ledger generated here covers `src/`, matching the first Wave 0 gate in
packet 030. Non-`src` unsafe from hardening, crates, and vendor remains covered
by packet 030 inventory and must get explicit disposition in P13 follow-up
work.

The working tree still contains the paused partial heap-slot helper slice:

- `src/am/common/heap_slot.rs`
- `src/am/common/mod.rs`
- `src/am/ec_spire/scan/relation.rs`
- `src/am/ec_diskann/scan_state.rs`

The ledger includes that current working-tree state. The helper slice can be
resumed as the first P5 implementation packet.

## Validation

- `python3 -m py_compile scripts/unsafe_ledger.py`
- `make unsafe-ledger`
- `make unsafe-ledger-check`

No PostgreSQL runtime tests were run because this packet only adds ledger
tooling and generated review artifacts.

