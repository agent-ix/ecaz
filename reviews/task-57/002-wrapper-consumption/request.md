# Task 57 Packet 002 — P3 Wrapper Consumption + Adjacent-Block Consolidation

Status: **back-filled** (code landed in commits `0e97baf42` and
`d81097686`; packet 005 reviewer seq-01 flagged the missing
`request.md` as a process gap, this packet documents the slice
after-the-fact).

## Why

Task 57 §Techniques #1–#2 call for consuming the Phase-1 P3 page/WAL
wrappers from Task 54 across the IVF subsystem. This packet is the
first code slice (preceding the §Exit target push in packet 004); it
mirrors the Task 54/003 HNSW pattern and the Task 55/002 DiskANN
consumer migration.

## Commits

- `0e97baf42` — main wrapper-consumption + adjacent-block
  consolidation slice.
- `d81097686` — `visit_ivf_posting_refs_for_block_sequence` safe-fn
  follow-up.

## Changes

### `src/am/ec_ivf/build.rs` — 10 → 7 (-3, -30.0%)

`write_data_pages` lifted from `unsafe fn(pg_sys::Relation, ...)` to
safe `fn(handle: RelationHandle, ...)`. Inner ops migrated from raw
PG FFI to Task 54 P3 wrappers:

- `unsafe { wal::GenericXLogTxn::start(relation) }` →
  `WalTxnScope::start_handle(handle)`.
- `unsafe { pg_sys::PageInit(page, page_size, 0) }` → `page.init(0)`.
- `unsafe { pg_sys::PageAddItemExtended(...) }` →
  `page.add_item(tuple).unwrap_or_else(|e| pgrx::error!(...))`.
- `pg_sys::InvalidOffsetNumber` sentinel check replaced by
  `Result<OffsetNumber, PageAddItemError>` discriminant.

`flush_build_plan` (the caller) validates `index_relation` via
`NonNull::new(...).unwrap_or_else(error!)` before passing the typed
handle — null check pushed up to the safety boundary.

### `src/am/ec_ivf/page.rs` — 16 → 14 (-2, -12.5%)

`IvfPageRelation::read_main` and `read_main_locked` consume
`LockedBufferGuard::read_main_handle` and
`LockedBufferGuard::read_main_locked_handle` (Task 54 P3 surface).
Drops 2 inner `unsafe { ... }` wraps at the IVF page-read boundary.

### `src/am/ec_ivf/vacuum.rs` — 5 → 3 (-2, -40.0%)

`debug_vacuum_stats`-side `ambulkdelete` + `amvacuumcleanup` adjacent
unsafe blocks consolidated into one outer `unsafe { ... }` block —
same factorization Task 54/004 (`239923e7d`) applied to HNSW.

### `src/am/ec_ivf/scan.rs` — 73 → 67 (-6, -8.2%)

Adjacent-block consolidations in `palloc_copy_slice`,
`resolve_scan_heap_relation`, and `resolve_scan_snapshot`. Also the
`d81097686` follow-up: `visit_ivf_posting_refs_for_block_sequence`
promoted from inner-`unsafe` invocation to a safe `pub(super) fn` at
one of its two call sites; the other retains the wrap because its
visitor closure derefs the scratch SoA raw pointer (see Anti-pattern
A discussion in `005-closeout/feedback/2026-05-24-02-reviewer.md`).

## §Per-file deltas (Packet 002 cumulative, both commits)

| File | Pre (Pkt 001 baseline) | Post (Pkt 002) | Δ | %Δ |
| --- | ---: | ---: | ---: | ---: |
| `src/am/ec_ivf/scan.rs` | 73 | 67 | -6 | -8.2% |
| `src/am/ec_ivf/page.rs` | 16 | 14 | -2 | -12.5% |
| `src/am/ec_ivf/build.rs` | 10 | 7 | -3 | -30.0% |
| `src/am/ec_ivf/vacuum.rs` | 5 | 3 | -2 | -40.0% |
| `src/am/ec_ivf/insert.rs` | 6 | 6 | 0 | 0% |
| `src/am/ec_ivf/cost.rs` | 2 | 2 | 0 | 0% |
| `src/am/ec_ivf/admin.rs` | 1 | 1 | 0 | 0% |
| **IVF subsystem total** | **113** | **100** | **-13** | **-11.5%** |
| `src/` total | 880 | 867 | -13 | |

Note: subsystem total reached 100 after this packet. That state was
proposed for close in packet 003 (since superseded — close was below
the §Exit target ≤65, blocked, then continued in packet 004 to reach
65).

## §Phase-1 wrappers consumed

| Wrapper | Module | Sites |
| --- | --- | --- |
| `LockedBufferGuard::read_main_handle` / `_locked_handle` | `src/storage/buffer_guard.rs` (Task 54) | `page.rs::IvfPageRelation::read_main` / `read_main_locked`; `build.rs::write_data_page` |
| `wal::WalTxnScope::start_handle` + `RegisteredBufferPage::{init, add_item}` | `src/storage/wal.rs` (Task 54) | `build.rs::write_data_page` |

No P6 datum wrapper consumption in this slice (already at the wrapper
boundary in `build.rs::detoasted_varlena_bytes`). No P8 wrapper
consumption (IVF has no DSM/parallel-build path).

## Wrapper extensions

**None.** Both commits consumed existing Task 54 surface directly.

## Validation

- `cargo check --no-default-features --features pg18 --lib` — passed
  (per `0e97baf42` commit msg: "No behavior change. cargo check
  --features pg18 --lib clean.").
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings`
  — pre-existing repo-wide lints unchanged; this slice introduced 0
  new IVF clippy findings.

## What's NOT in this slice

- No insert.rs / cost.rs / admin.rs touches (structural-ceiling
  rationale documented in packet 005 §Structural-ceiling rationale).
- No safe-fn lift campaign — that's packet 004's strategy.
- No bench gate — bench evidence belongs to packet 005 closeout (per
  the slice-005-bench-on-final-HEAD convention).

## References

- `plan/tasks/57-ivf-unsafe-burndown.md`
- `reviews/task-57/001-execution-plan/request.md`
- `reviews/task-54/005-closeout/request.md` — Task 54 P3 wrapper
  surface (the consumed surface).
- `reviews/task-55/002-consumer-migration/request.md` — DiskANN
  cross-AM precedent for this pattern.
- `reviews/task-57/004-additional-burndown/request.md` — follow-on
  slice that took 100 → 65.
- `reviews/task-57/005-closeout/feedback/2026-05-24-02-reviewer.md`
  — reviewer code/safety review covering both 002 commits, with
  anti-pattern A discussion for the
  `visit_ivf_posting_refs_for_block_sequence` follow-up.
