# Task 57 Packet 001 — Execution Plan: IVF Unsafe Burndown

Status: **plan** — gate opened by operator (Task 51 IVF/RaBitQ Second
Optimization Round closed at `5ade0feab`; main was pulled before this
branch).

## Baseline (HEAD = task-57 from main `9afb2c6b8`)

| File | Unsafe blocks |
| --- | ---: |
| `src/am/ec_ivf/scan.rs` | 73 |
| `src/am/ec_ivf/page.rs` | 16 |
| `src/am/ec_ivf/build.rs` | 10 |
| `src/am/ec_ivf/insert.rs` | 6 |
| `src/am/ec_ivf/vacuum.rs` | 5 |
| `src/am/ec_ivf/cost.rs` | 2 |
| `src/am/ec_ivf/admin.rs` | 1 |
| `src/am/ec_ivf/{mod,options,quantizer,routine,training}.rs` | 0 |
| **IVF subsystem total** | **113** |
| **§Exit target** | **≤ 65 (-40%)** |
| **§Exit floor** (per Task 50 -30%) | **≤ 79** |
| `src/` total | 880 |

## Phase-1 wrapper inventory (already in place)

| Wrapper | Module | Provider | Expected IVF consumption |
| --- | --- | --- | --- |
| `LockedBufferGuard::read_main_handle` / `_locked_handle` | `src/storage/buffer_guard.rs` | Task 54 | `page.rs` read_main/read_main_locked sites; `build.rs` data page allocation; `scan.rs` per-block reads |
| `wal::WalTxnScope::start_handle` + `RegisteredBufferPage::{init, add_item}` | `src/storage/wal.rs` | Task 54 | `build.rs` write path; `page.rs` mutation path |
| P6 datum wrappers (`DetoastedVarlena`, `FlatFloat4Source`) | `src/am/common/datum.rs` | Task 53 | `insert.rs` + `scan.rs` datum extraction |
| P8 typed views (DSM/atomic/SpinLock) | `src/am/common/dsm.rs` | Task 52 | Audit IVF for parallel-build / cluster-assignment paths |

## Migration patterns (per Tasks 54/55 precedent)

- `unsafe { LockedBufferGuard::read_main(rel, ...) }` →
  `LockedBufferGuard::read_main_handle(handle, ...)` where
  `handle: RelationHandle` is constructed once at function entry.
- `unsafe { wal::GenericXLogTxn::start(rel) }` →
  `wal::WalTxnScope::start_handle(handle)`.
- `unsafe { pg_sys::PageInit / PageAddItemExtended }` →
  `page.init(special_size)` + `page.add_item(payload).unwrap_or_else(...)`.
- `unsafe { DetoastedVarlena::*_from_datum(datum) }` etc. — already
  at P6 wrapper boundary; audit for safe-fn lifts where caller invariants permit.

## Slice plan

1. **001 — plan** (this packet). No code.

2. **002 — `scan.rs` first pass**: IVF's hot path. Audit for
   wrappable per-block buffer reads, datum decodes, and
   page-mutation chains. Target slice delta: -15 to -25 from
   scan.rs's 73. **Bench-sensitive** — Task 51's NEON kernel and
   bound-prune work landed in this file; preserve those.

3. **003 — `page.rs` + `vacuum.rs` migration**: page-mutation chain
   (PageInit / PageAddItemExtended via WalTxnScope::register_page);
   vacuum-side rewrite paths. Target: page.rs -6 to -10,
   vacuum.rs -2 to -3.

4. **004 — `build.rs` + `insert.rs`**: full P3 chain migration
   mirroring HNSW `write_data_pages` and DiskANN `ambuild::write_data_pages`
   patterns from Tasks 54/55. Target: build.rs -5 to -7,
   insert.rs -2 to -3.

5. **005 — bench gate + closeout**: local IVF M5 bench against the
   `fixtures/m5_diskann_real{10k,100k}` corpora (no IVF-specific
   M5 baseline pre-exists; Task 57 may establish one if needed)
   OR re-cite the Task 51 1M AWS baseline as the reference state.

## Bench gate strategy

Task 51's bench gate is AWS-1M (`benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/`).
Re-running that on M5 macOS is impractical (different host, 1M
corpus). Options:

- **Option A**: Establish a new local M5 IVF baseline at
  `benchmarks/task-57-m5-ivf-baseline/` using the 10k+100k M5
  corpora (mirroring `benchmarks/task-50-m5-hnsw-baseline/`'s
  structure). Run pre-Task-57 baseline (current `main`), then
  re-run after migration; bit-for-bit identical storage + recall
  within ci95 + latency within 5% noise band.
- **Option B**: Compile-gate + scan a subset of IVF profile via
  `ecaz bench` at 10k only. Faster but weaker signal.

Recommend **Option A** for §Exit Criterion #3 evidence with the
substantial post-Task-51 changes carried through.

## Out of scope

- HNSW (closed), SPIRE (still gated), DiskANN (Task 55 closed).
- RaBitQ scoring math (§Non-Goals; Task 51's domain).
- Posting layout v2 boundaries (§Non-Goals).
- Re-applying the `pg_am_callback!` macro wrap on
  `ec_ivf_build_callback` that the `ebb022a7a` merge dropped per
  Task 50's note — **this re-application IS in scope** for slice 004.

## Validation gates (per slice)

- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings`
- per-file `grep -c "unsafe {" …` snapshot per packet
- `src/` total after slice

Bench gate runs once in slice 005 against the Task-57 M5 IVF
baseline (slice 005 establishes it if needed).

## References

- `plan/tasks/57-ivf-unsafe-burndown.md`
- `reviews/task-51/023-round-closeout/` — Task 51 baseline + AWS gate
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/manifest.md` — AWS reference
- `reviews/task-54/005-closeout/request.md` — P3 wrapper surface
- `reviews/task-55/002-consumer-migration/request.md` — DiskANN cross-AM precedent
