# Task 56: SPIRE Unsafe Burndown

Status: **complete** (2026-06-16) — SPIRE unsafe burndown merged
(`392432134`): 194 → 135 `unsafe` blocks (−30% floor, bench gate
green). The 12 pre-existing missing `/// # Safety` docs were closed
as Task 56.1 (`2c6766503`) with reviewer-approved 100% SPIRE doc
parity. Closeout `reviews/task-56/006-closeout/` +
`007-doc-parity-followup/`. (Prior "deferred" header was a stale
pre-start placeholder.)

## Why

`src/am/ec_spire/` carries **194** `unsafe { ... }` blocks across
~20 files — the largest remaining AM target after HNSW closed. The
448 closeout names SPIRE as the next-highest-density module:

> After HNSW, the next highest-density modules in the post-Task-35
> ledger are (in approximate descending order):
> - `src/am/ec_spire/**` — SPIRE custom scan / coordinator / storage
> - ...

SPIRE is currently under active optimization on its own branches
(custom scan, coordinator, storage). Opening a burndown rotation now
would replay the same merge conflict pattern we just absorbed on IVF
during Task 50's main-merge: branch-side safety scaffolding vs
main-side optimization, with main winning on substance per the memory
rule. That's wasted lift work.

## Gate

This task opens **only when both of**:

1. SPIRE optimization branches have merged to main and AWS testing
   has reported acceptance.
2. Tasks 52 (P8), 53 (P6), and 54 (P3) Phase-1 wrappers have landed.

Until then, SPIRE residual unsafe blocks stay where they are. The
Task 50 §Exit Criteria's "or the request explains why a lower
reduction is the structural ceiling" is acceptable for the
intermediate window because SPIRE was never in Task 50 scope.

## Non-Goals

- Do not start before the gate opens. Reviewer enforces this.
- Do not touch HNSW / IVF / DiskANN when this task opens. Sibling
  tasks own those.
- Do not refactor SPIRE coordinator state machines. Concurrency state
  machine lifts are Task 40's domain.
- Do not run a SIMD micro-optimization pass.

## Scope (preview — exact files refreshed at task open)

Audit and structurally reduce `unsafe { ... }` blocks across
`src/am/ec_spire/`. Current high-density files:

| File | Count |
| --- | ---: |
| `dml_frontdoor/mod.rs` | 28 |
| `custom_scan/plan_private.rs` | 16 |
| `custom_scan/cost_helpers.rs` | 16 |
| `custom_scan/begin_exec.rs` | 15 |
| `page.rs` | 13 |
| `custom_scan/planner.rs` | 11 |
| `coordinator/debug.rs` | 11 |
| `vacuum/mod.rs` | 10 |
| `coordinator/snapshots.rs` | 10 |
| (remainder distributed across ~12 files) | ~64 |
| **SPIRE subsystem total** | **194** |

Refresh exact counts at task open; SPIRE optimization will have
shifted these.

## Techniques

Same as Task 50 + Phase-1 consumer pattern. Specifically:

1. **Consume Phase-1 P6 datum wrappers** in
   `dml_frontdoor/mod.rs` and `insert.rs` for vector-extraction
   paths.
2. **Consume Phase-1 P3 page/WAL wrappers** in `page.rs`,
   `storage/relation_store.rs`, and `vacuum/mod.rs`.
3. **Consume Phase-1 P8 wrappers** if SPIRE has parallel-build paths
   (verify at task open).
4. **Safe-fn lifts** for custom_scan / coordinator helpers whose
   bodies compose already-safe operations.
5. **Narrow block scoping** across the custom_scan layer.

## Migration Targets

Target -40% subsystem-wide, modulo structural-ceiling files. Specific
per-file targets refreshed at task open after SPIRE optimization
stabilizes.

**Aspirational subsystem total**: 194 → ≤ 115 (-41%).

## Slice and Packet Rules

Same as Tasks 50 / 52-55.

## Performance Gate

SPIRE's read-efficiency bench from Task 30 phase 13d is the relevant
lane. Required evidence at task close:

- New benchmark packet `benchmarks/task-56-m5-spire-baseline/`
  modeled on `benchmarks/task-50-m5-hnsw-baseline/`.
- Recall + latency + storage + the SPIRE-specific read-efficiency
  metric must not regress beyond noise.

Pre-state baseline: established by the SPIRE optimization closeout
(referenced via the gate criterion above).

## Validation

Standard validation suite (fmt + check + clippy + focused tests +
block counts + `src/` total).

## Exit Criteria

Task closes when:

- SPIRE subsystem block count ≤ 115.
- Each touched file ≥ -30% reduction or documented ceiling.
- SPIRE bench packet runs cleanly with all steps succeeded.
- A closing summary packet records final per-file distribution,
  Phase-1 wrappers consumed, any wrapper extensions, `src/` total
  change, and structural-ceiling rationale for sub-30% files.

## Coordination

- **Hard gate**: do not open until SPIRE optimization stable signal
  is received AND Phase-1 (Tasks 52/53/54) have closed.
- Reviewer scope-lock: SPIRE-only on `task-56-spire-burndown` branch.
- No overlap with Task 51 (IVF RaBitQ optimization) — different AM.
- Coordinate with Task 40 (concurrency state-machine lifts) if SPIRE
  coordinator paths are in active state-machine refactor.

## Cross-References

- Inherits Task 50 §Performance Gate template.
- Consumes wrappers from Tasks 52, 53, 54.
- Memory rule `feedback_main_priority_in_conflicts` (2026-05-22)
  governs the gate decision.
