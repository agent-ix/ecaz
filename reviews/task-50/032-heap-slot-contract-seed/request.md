# Task 50 Heap Slot Contract Seed

This packet starts P5 from the comprehensive unsafe burndown plan:
heap source, tuple slot, snapshot, and scorer contracts.

It introduces `src/am/common/heap_slot.rs` as the first shared boundary for:

- `ExecClearTuple`
- `table_tuple_fetch_row_version`
- `slot_getsomeattrs_int`
- `tts_isnull`
- `tts_values`

The helper is consumed by:

- SPIRE heap rerank source-vector loading in `src/am/ec_spire/scan/relation.rs`
- DiskANN scan heap-source loading in `src/am/ec_diskann/scan_state.rs`
- HNSW source loading in `src/am/ec_hnsw/source.rs`

IVF heap rerank already goes through the HNSW source helper, so this seed also
starts moving the IVF/RaBitQ path behind the same shared slot contract without
touching IVF scan ordering in this packet.

## Unsafe Movement

| File | Before | After | Notes |
| --- | ---: | ---: | --- |
| `src/am/ec_spire/scan/relation.rs` | 35 | 29 | local slot fetch/datum helpers removed |
| `src/am/ec_diskann/scan_state.rs` | 24 | 20 | local slot fetch/datum internals delegated |
| `src/am/ec_hnsw/source.rs` | 52 | 51 | HNSW source helper now delegates to common slot contract |
| `src/am/common/heap_slot.rs` | 0 | 7 | new shared boundary helper |

Net touched-file movement: 111 -> 107 direct unsafe blocks.

This is a seed packet, not P5 completion. The next P5 packets should continue
from this helper into a fuller `HeapSourceScorer` / `HeapSlotReader` shape so
callers can stop invoking raw-slot helpers under `unsafe`.

## Ledger

Generated `artifacts/unsafe-ledger-after.jsonl` after the code change.

`make unsafe-ledger-check` passed:

```text
ledger covers 2445 current unsafe rows
```

The project is still far from Task 50 closeout. This packet reduces one net
direct unsafe row in `src/` and centralizes a repeated slot contract; it does
not claim broad burndown completion.

## Validation

Passed:

- `rustfmt src/am/common/heap_slot.rs src/am/ec_hnsw/source.rs src/am/ec_spire/scan/relation.rs src/am/ec_diskann/scan_state.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `make unsafe-ledger`
- `make unsafe-ledger-check`

Blocked / failed:

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  fails on existing repository-wide clippy debt. The first failure is unused
  imports in `src/am/mod.rs`; the log records 109 prior clippy errors. The
  clippy failures are not introduced by this P5 helper slice.

No runtime PostgreSQL tests were run because this packet only centralizes tuple
slot access and does not change scan ordering, candidate ordering, payload
bytes, or WAL behavior.

