# Task 56 Packet 001 — Execution Plan: SPIRE Unsafe Burndown

Status: **plan**

## Branch + baseline

- Branch: `task-56-spire-burndown` (forked from main at `329925109`,
  the Task 57 merge).
- HEAD baseline: `329925109`.
- Full per-file SPIRE block counts captured in
  `artifacts/baseline-block-counts.txt`.

## Baseline (`src/am/ec_spire/`)

| File | Unsafe blocks |
| --- | ---: |
| `dml_frontdoor/mod.rs` | 28 |
| `custom_scan/cost_helpers.rs` | 16 |
| `custom_scan/plan_private.rs` | 16 |
| `custom_scan/begin_exec.rs` | 15 |
| `page.rs` | 13 |
| `custom_scan/planner.rs` | 11 |
| `coordinator/debug.rs` | 11 |
| `coordinator/snapshots.rs` | 10 |
| `vacuum/mod.rs` | 10 |
| `build/drafts.rs` | 7 |
| `custom_scan/dml.rs` | 7 |
| `coordinator/remote_candidates/dispatch.rs` | 5 |
| `scan/relation.rs` | 5 |
| `storage/relation_plan.rs` | 5 |
| `insert.rs` | 4 |
| `storage/relation_store.rs` | 4 |
| `coordinator/maintenance.rs` | 3 |
| `cost/mod.rs` | 3 |
| `custom_scan/tuple_payload.rs` | 3 |
| `build/publish.rs` | 2 |
| `coordinator/lifecycle.rs` | 2 |
| `custom_scan/explain.rs` | 2 |
| `custom_scan/mod.rs` | 2 |
| (remainder, 1 each) | 12 |
| **SPIRE subsystem total** | **194** |
| **§Exit target** | **≤ 115 (-41%)** |
| **§Exit floor** (per Task 50 -30%) | **≤ 135** |
| `src/` total | 832 (post-Task-57) |

## Phase-1 wrapper inventory available

| Wrapper surface | Source | Expected SPIRE consumption |
| --- | --- | --- |
| `LockedBufferGuard::read_main_handle` / `_locked_handle` | `src/storage/buffer_guard.rs` (Task 54) | `page.rs` page-read paths; `vacuum/mod.rs` rewrite paths; `storage/relation_store.rs` |
| `wal::WalTxnScope::start_handle` + `RegisteredBufferPage::{init, add_item}` | `src/storage/wal.rs` (Task 54) | `page.rs` mutation; `storage/relation_store.rs`; `build/drafts.rs` |
| `DetoastedVarlena::*_from_datum` (P6 datum wrapper) | `src/am/common/datum.rs` (Task 53) | `dml_frontdoor/mod.rs` vector-extraction; `insert.rs`; `build/tuples.rs` |
| P8 typed views (DSM/atomic/SpinLock) | `src/am/common/dsm.rs` (Task 52) | Audit at slice open — SPIRE has no obvious parallel-build path on main, but coordinator may benefit |
| `IndexRelationGuard`, `HeapRelationGuard`, `ActiveSnapshotGuard`, `IndexScanGuard`, `TupleTableSlotGuard` (typed RAII) | `src/storage/{relation,snapshot,scan}_guard.rs` | `custom_scan/begin_exec.rs`; `scan/relation.rs`; `coordinator/snapshots.rs` |
| `RelationHandle` (`NonNull<RelationData>`) | `src/storage/relation.rs` | Anti-pattern A fixes: any `fn(pg_sys::Relation, ...)` safe-fn surface accepting raw pointer can be lifted to typed handle (Task 57 precedent) |

## Migration patterns (per Tasks 54/55/57 precedent)

1. **P3 wrapper consumption** (page.rs / storage / vacuum):
   `unsafe { LockedBufferGuard::read_main(rel, ...) }` →
   `LockedBufferGuard::read_main_handle(handle, ...)` where
   `handle: RelationHandle` is constructed once at function entry.
   Same for `wal::GenericXLogTxn::start` → `WalTxnScope::start_handle`.
   `pg_sys::PageInit / PageAddItemExtended` → `page.init(0)` /
   `page.add_item(...).unwrap_or_else(...)`.

2. **P6 datum wrapper consumption** (dml_frontdoor / insert / build):
   `unsafe { DetoastedVarlena::packed_from_datum(datum) }` already
   at the wrapper boundary in most places — audit for safe-fn lifts
   where caller invariants permit (Task 57 slice 004 pattern).

3. **Safe-fn lifts via Option<Box<T>>** (custom_scan / coordinator):
   Per Task 57 reviewer seq 02 + seq 05 self-correction: any
   `*mut Box<T>` field can be lifted to `Option<Box<T>>`, making
   the matching `free_*` helpers safe `fn`. For fields with
   borrow-checker constraints (closure captures), the `mem::replace`
   + bounded `Box::from_raw` pattern keeps the free helper safe with
   a narrow inner unsafe block.

4. **Typed `RelationHandle` boundaries**: per Task 57 anti-pattern A
   fix, lift any `fn(pg_sys::Relation, ...)` safe-fn surface to take
   `RelationHandle` so the null check moves to the call site.

5. **Adjacent-block consolidation**: per Task 57
   `explain_counters_from_index_scan_state` pattern, merge adjacent
   inner blocks sharing a SAFETY rationale into a single block with
   a unified SAFETY comment.

6. **Debug helper lifts**: per Task 57 slice 004 + HNSW
   `scan_debug.rs` precedent, promote `#[cfg(test)]` debug helpers
   from `fn` with inner unsafe to `unsafe fn` with caller-supplied
   contract; drop redundant inner blocks. All callers in test
   fixtures already use the `ec_spire_debug!` (or equivalent) macro
   so the promotion is transparent.

## Slice plan

1. **001 — plan** (this packet). No code.

2. **002 — `page.rs` + `storage/` P3 wrapper consumption**:
   `page.rs` (13), `storage/relation_store.rs` (4),
   `storage/relation_plan.rs` (5), `storage/tests/helpers.rs` (1).
   Target slice delta: -8 to -12 across these files.

3. **003 — `dml_frontdoor/mod.rs` + `insert.rs` P6 + safe-fn pass**:
   `dml_frontdoor/mod.rs` (28) is the densest single file. Audit
   for repeated datum-extraction patterns that can consolidate via
   the P6 surface; safe-fn lift for helpers whose bodies compose
   already-safe operations after consumption.
   Target: dml_frontdoor -8 to -12, insert.rs -1 to -2.

4. **004 — `custom_scan/` debug + planner + helper lifts**:
   `cost_helpers.rs` (16), `plan_private.rs` (16), `begin_exec.rs`
   (15), `planner.rs` (11), `dml.rs` (7), `tuple_payload.rs` (3),
   `explain.rs` (2), `mod.rs` (2). Heavy concentration of cost /
   plan-private helpers and exec begin paths — many likely consume
   the Task 57 typed-handle / Option<Box<T>> patterns.
   Target: cumulative -25 to -35.

5. **005 — `coordinator/` + `vacuum/` cleanup**:
   `debug.rs` (11), `snapshots.rs` (10), `vacuum/mod.rs` (10),
   `dispatch.rs` (5), `maintenance.rs` (3), `lifecycle.rs` (2),
   remainder. Target: cumulative -8 to -12.

6. **006 — bench gate + closeout**: SPIRE bench at task close
   (modeled on `benchmarks/task-50-m5-hnsw-baseline/`). Target file
   under `benchmarks/task-56-m5-spire-baseline/`. Closeout packet
   records per-file final distribution, Phase-1 wrappers consumed,
   any extensions, `src/` total change, and structural-ceiling
   rationale for sub-30% files.

Cumulative slice target: 194 → ≤ 115 (subsystem -40%+) with margin
following the Task 57 reviewer seq 01 zero-margin lesson.

## Bench gate strategy

Per task spec §Performance Gate, run a SPIRE read-efficiency bench
profile at closeout. Modeled on
`benchmarks/task-50-m5-hnsw-baseline/` and Task 57's bench-rerun
approach: run once on Task-56-HEAD, capture suite manifest +
results.jsonl + per-step logs. If a baseline reference packet
already exists (look for `benchmarks/task-56-m5-spire-baseline/`
or sibling), compare; otherwise the post-Task-56 state IS the new
baseline.

## Out of scope (per §Non-Goals)

- HNSW / IVF (closed/merged) and DiskANN (Task 55 closed).
- SPIRE coordinator state-machine refactors (Task 40's domain).
- SIMD micro-optimization passes.

## Validation gates (per slice)

- `cargo check --no-default-features --features pg18 --lib`
- `cargo check --all-targets --no-default-features --features pg18`
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings`
  (verify no new SPIRE clippy findings)
- per-file `grep -c "unsafe {" …` snapshot per packet
- `src/` total after slice
- `unsafe fn` vs `/// # Safety` parity check after any safe-fn
  promotion slice (Task 57 reviewer seq 02 lesson)

Bench gate runs once in slice 006 against the Task-56 SPIRE bench
profile (slice 006 establishes one if needed).

## References

- `plan/tasks/56-spire-unsafe-burndown.md`
- `reviews/task-57/005-closeout/` — Task 57 IVF burndown precedent
  (close-with-margin pattern, Option<Box<T>> + mem::replace
  patterns, anti-pattern A fix, safety-doc parity)
- `reviews/task-57/005-closeout/feedback/2026-05-24-05-reviewer.md`
  — `feedback_dont_defer_safety_fixes` rule lineage; will apply
  from slice 002 onward (no safety regressions deferred as
  "follow-on").
- `reviews/task-54/005-closeout/request.md` — P3 wrapper surface
  (the consumed surface).
- `reviews/task-55/002-consumer-migration/request.md` — DiskANN
  cross-AM consumer precedent.
- Task 50 §Performance Gate template (inherited).
