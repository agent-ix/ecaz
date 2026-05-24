# Task 50/444: HNSW vacuum.rs — `finalize_fully_dead_elements_*` chain safe-fn lifts

## Why this slice

After slice 424 made `shared::with_page_line_tuple_bytes` safe
and slice 428 made `apply_page_pass1_updates` safe, the
finalize-dead-elements chain composes only safe operations
(plus `VacuumIndexRelation` RAII helpers, which were already
safe). The historical `unsafe fn` contracts are no longer
needed.

## Scope

Two `unsafe fn` → safe `fn` lifts in `src/am/ec_hnsw/vacuum.rs`:

1. `finalize_fully_dead_elements_with_storage` — dispatcher over
   per-block runs of dead-element finalization.
2. `finalize_fully_dead_elements_on_page_with_storage` — body
   uses `VacuumIndexRelation::read_main_locked` (safe RAII
   buffer guard) + `with_page_line_tuple_bytes` (safe) +
   `begin_page_rewrite` (safe RAII WAL transaction) +
   `apply_page_pass1_updates` (safe).

Caller-side `unsafe { ... }` wraps stripped (three):

- `finalize_fully_dead_elements_with_storage` per-block dispatch
  to `finalize_fully_dead_elements_on_page_with_storage`.
- `finalize_fully_dead_elements_on_page_with_storage` internal
  call to `with_page_line_tuple_bytes`.
- `VacuumFormatAdapter::finalize_fully_dead_elements` method
  wrap.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/vacuum.rs` | 23 | 21 | -2 |
| **HNSW subsystem subtotal** | **334** | **332** | **-2** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 443 | 334 |
| After 444 | 332 |

**Net rotation delta: -217 in HNSW (-39.5%).**

## Soundness rationale

The `VacuumIndexRelation` carries the live vacuum relation guard
and its `read_main_locked` returns a `LockedBufferGuard` whose
Drop releases the lock; `begin_page_rewrite` returns a
`VacuumPageRewrite` whose Drop finalizes or discards the WAL
transaction. All used helpers are already safe. The lifts are
pure signature.

No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/444-hnsw-vacuum-finalize-dead-safe/artifacts/`:

- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

amvacuumcleanup pass-2 finalization path; signature-only change.
Bench evidence gathered out-of-band per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**-217 (-39.5%)** on HNSW: 549 → 332. The -30% Exit Criteria
target now has a **9.5-point cushion**. Approaching the **-40%**
threshold.
