# Task 55 Packet 001 — Execution Plan: DiskANN Unsafe Burndown

Status: **plan**

## Baseline at task open (HEAD = task-55 branch tip from main `f29095a00`)

| File | Unsafe blocks | §Target | Δ needed |
| --- | ---: | ---: | --- |
| `src/am/ec_diskann/routine.rs` | 27 | ≤ 16 | -11 (-41%) |
| `src/am/ec_diskann/ambuild.rs` | 19 | ≤ 11 | -8 (-42%) |
| `src/am/ec_diskann/insert.rs` | 8 | ≤ 5 | -3 (-38%) |
| `src/am/ec_diskann/scan_state.rs` | 5 | residual | safe-fn lifts where applicable |
| `src/am/ec_diskann/cost.rs` | 4 | residual | safe-fn lifts where applicable |
| `src/am/ec_diskann/diagnostics.rs` | 1 | — | — |
| `src/am/ec_diskann/options.rs` | 1 | — | — |
| **DiskANN total** | **65** | **≤ 40** | **-25 (-38%)** |
| `src/` total | 949 | — | (negative net expected) |

## Phase-1 wrapper inventory (already in place)

| Wrapper | Module | Provider | Used by DiskANN sites |
| --- | --- | --- | --- |
| `LockedBufferGuard::read_main_handle` / `_locked_handle` | `src/storage/buffer_guard.rs` | Task 54 | `insert.rs:102,111`; `routine.rs:1530,1604`; `ambuild.rs:716-790` |
| `wal::WalTxnScope::start_handle` + `RegisteredBufferPage::{init, add_item}` | `src/storage/wal.rs` | Task 54 | `insert.rs:116`; `routine.rs:1561,1621`; `ambuild.rs:751,781` |
| P6 datum wrappers (`DetoastedVarlena`, `FlatFloat4Source`) | `src/am/common/datum.rs` | Task 53 | `routine.rs:1709,1721`; `insert.rs:1206,1230` |
| P8 typed views (DSM/atomic/SpinLock) | `src/am/common/dsm.rs` | Task 52 | DiskANN does not have a parallel-build path; skip |

## Slice plan

1. **001 — plan** (this packet). No code.
2. **002 — `routine.rs` first pass** — P3 wrapper consumption for the
   `apply_tuple_rewrites` / `apply_*_chain_rebuild` chain (L1530-L1621);
   P6 datum wrapper consumption for the `with_ecvector_datum_slice`
   path. Target slice delta: routine.rs -8 to -10.
3. **003 — `ambuild.rs`** — full P3 chain migration mirroring HNSW
   `write_data_pages` pattern (Task 54/003). Target: ambuild.rs
   -8 to -10.
4. **004 — `insert.rs` + small files** — P3 + P6 consumer migration;
   plus `scan_state.rs` / `cost.rs` interior self-narrows where
   provable. Target: insert.rs -3 to -4; small files -1 to -2.
5. **005 — DiskANN baseline + closeout** — write
   `benchmarks/task-55-m5-diskann-baseline/` (suite config + run) as
   the new reference; closeout summary with per-file deltas, src/
   total change, and confirmation that no Phase-1 wrapper extensions
   were required.

## Migration patterns (carried over from Task 54)

- `unsafe { LockedBufferGuard::read_main(rel, ...) }` →
  `LockedBufferGuard::read_main_handle(handle, ...)` (safe call)
  where `handle: RelationHandle` is constructed once at function
  entry (or already held).
- `unsafe { wal::GenericXLogTxn::start(rel) }` →
  `wal::WalTxnScope::start_handle(handle)` (safe).
- `unsafe { pg_sys::PageInit(...) }` + `unsafe { pg_sys::PageAddItemExtended(...) }`
  → `page.init(special_size)` + `page.add_item(payload).unwrap_or_else(...)`
  on the `RegisteredBufferPage` returned by `WalTxnScope::register_page`.
- `unsafe { DetoastedVarlena::*_from_datum(datum) }` /
  `unsafe { FlatFloat4Source::from_datum(datum, ...) }` is already
  in DiskANN (`ambuild::with_ecvector_datum_slice`); audit if the
  P6 unsafe-fn boundary can be lifted to safe-fn via handle/kind
  validation as Task 53 did for HNSW source.rs.

## Out of scope

- HNSW (closed), SPIRE (Task 56), IVF (Task 57).
- DiskANN scoring math (no SIMD micro-opt pass per §Non-Goals).
- New WAL record formats (§Non-Goals carried over from Task 54).

## Validation gates (per slice)

- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings` (lib-only; pre-existing repo lints are not in scope per §Non-Goals)
- per-file `grep -c "unsafe {" …` snapshot in each packet request
- `src/` total after slice

Bench gate runs once in slice 005 against the freshly established
`benchmarks/task-55-m5-diskann-baseline/`.

## References

- `plan/tasks/55-diskann-unsafe-burndown.md`
- `reviews/task-54/005-closeout/request.md` (P3 wrapper surface + handoff list naming each DiskANN site)
- `reviews/task-53/004-closeout/` (P6 wrapper migration pattern)
- `benchmarks/task-50-m5-hnsw-baseline/manifest.md` (template for the new DiskANN baseline packet)
