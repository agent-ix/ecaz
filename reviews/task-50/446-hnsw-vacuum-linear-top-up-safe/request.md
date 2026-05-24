# Task 50/446: HNSW vacuum.rs — `top_up_repair_replacements_from_linear_scan` safe-fn lift — **-40% milestone**

## Why this slice

After slices 422 (`load_exact_graph_element`),
424 (`with_page_line_tuple_bytes`), and 443
(`VacuumSearchMetric::score_graph_element`), the linear-scan
top-up function for repair replacements composes only safe
operations. The historical `unsafe fn` contract is no longer
needed.

## Scope

One `unsafe fn` → safe `fn` lift in `src/am/ec_hnsw/vacuum.rs`:

- `top_up_repair_replacements_from_linear_scan` — body uses:
  - `VacuumIndexRelation::main_fork_block_count` (safe)
  - `VacuumIndexRelation::read_main_locked` (safe RAII guard)
  - `collect_linear_repair_candidates_on_page` (already safe fn)

Caller-side `unsafe { ... }` wrap stripped (one):

- `plan_repair_replacement` internal call to
  `top_up_repair_replacements_from_linear_scan`.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/vacuum.rs` | 19 | 18 | -1 |
| **HNSW subsystem subtotal** | **330** | **329** | **-1** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 445 | 330 |
| After 446 | 329 |

**Net rotation delta: -220 in HNSW (-40.07%). Crosses -40%.**

## Soundness rationale

The lifted function has zero internal `unsafe { ... }` blocks
after the cascade. The lift is pure signature.

No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/446-hnsw-vacuum-linear-top-up-safe/artifacts/`:

- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

amvacuumcleanup linear-top-up path; signature-only change.
Bench evidence gathered out-of-band per
`feedback_coder_push_smoke_checks`.

## Rotation milestone — **-40% threshold crossed**

**-220 (-40.07%)** on HNSW: 549 → 329. The -30% Exit Criteria
target now has a **10.07-point cushion**.

| Threshold | Crossed at |
| --- | --- |
| -30% (Exit Criteria) | packet 429 |
| -36% (rotation closeout snapshot) | packet 438 |
| -38% | packet 442 |
| **-40%** | **packet 446** |

## Notes for next rotation

Block-count metric is approaching the irreducible boundary surface
for HNSW. The 329 remaining blocks largely consist of:

- `unsafe extern "C-unwind" fn` AM callback shells (per PG ABI).
- PG DSM atomic + SpinLock primitives in build_parallel.rs.
- `#[target_feature]` SIMD intrinsics in source.rs.
- Closure-bound traversal drivers in graph.rs (FnMut/ScoreFn).
- Page-mutation primitive scaffolding in shared.rs (`PageInit`,
  `wal::GenericXLogTxn`, `from_raw_parts`).

Further reductions require restructuring the closure-CPS pattern
or fully encapsulating the page-mutation primitives — both deeper
refactors than the per-slice safe-fn lift pattern.
