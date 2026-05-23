# Task 50/445: HNSW vacuum.rs — `apply_repair_plans` chain safe-fn lifts

## Why this slice

After slice 424 made `shared::with_writable_page_tuple_bytes` safe
and the `VacuumIndexRelation` RAII helpers (`read_main_locked`,
`begin_page_rewrite`) were already safe, the `apply_repair_plans`
chain composes only safe operations. The historical `unsafe fn`
contracts are no longer needed.

## Scope

Two `unsafe fn` → safe `fn` lifts in `src/am/ec_hnsw/vacuum.rs`:

1. `apply_repair_plans` — dispatcher over per-page runs of
   layer-repair plans.
2. `apply_repair_plans_on_page` — body uses
   `VacuumIndexRelation::read_main_locked` (safe RAII) +
   `with_writable_page_tuple_bytes` (safe) +
   `unlink_deleted_neighbor_refs` (safe) +
   `apply_repair_plan` (safe).

Caller-side `unsafe { ... }` wraps stripped (three):

- `apply_repair_plans` per-block dispatch to
  `apply_repair_plans_on_page`.
- `apply_repair_plans_on_page` internal call to
  `with_writable_page_tuple_bytes`.
- `repair_graph_connections_with_storage` call to
  `apply_repair_plans` (the wrap moved out of the larger
  multi-call unsafe block).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/vacuum.rs` | 21 | 19 | -2 |
| **HNSW subsystem subtotal** | **332** | **330** | **-2** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 444 | 332 |
| After 445 | 330 |

**Net rotation delta: -219 in HNSW (-39.9%).**

## Soundness rationale

The `VacuumIndexRelation` carries the live vacuum relation guard;
`read_main_locked` returns a `LockedBufferGuard` whose Drop releases
the lock; `begin_page_rewrite` returns a `VacuumPageRewrite` whose
Drop finalizes or discards the WAL transaction. All used helpers
are already safe. The lifts are pure signature.

No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/445-hnsw-vacuum-apply-repair-safe/artifacts/`:

- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

amvacuumcleanup repair-application path; signature-only change.
Bench evidence gathered out-of-band per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**-219 (-39.9%)** on HNSW: 549 → 330. The -30% Exit Criteria
target now has a **9.9-point cushion**. One block away from -40%.
