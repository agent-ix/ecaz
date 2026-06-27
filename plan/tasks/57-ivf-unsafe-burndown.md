# Task 57: IVF Unsafe Burndown

Status: **complete** (2026-06-16) — IVF unsafe burndown landed on
`main` (`f4caf3c23` + `e827d8ee7`): subsystem at 63 (≤65, +2 margin),
all §Exit Criteria met, bench gate bit-for-bit re-confirmed. Closeout
`reviews/task-57/005-closeout/`; final acceptance
`reviews/task-57/005-closeout/feedback/2026-06-16-01-reviewer.md`.
Optional non-blocking residual: restore `Debug` on `EcIvfScanOpaque`.
(Prior "deferred" header was a stale pre-start placeholder.)

## Why

`src/am/ec_ivf/` carries **109** `unsafe { ... }` blocks (refreshed
2026-05-23, post-Task-50 main merge):

| File | Count |
| --- | ---: |
| `scan.rs` | 69 |
| `page.rs` | 16 |
| `build.rs` | 10 |
| `insert.rs` | 6 |
| (other helpers) | ~8 |
| **IVF subsystem total** | **109** |

The Task 50 main-merge (commit `ebb022a7a`) explicitly dropped some
of the branch's earlier IVF safe-fn lifts to honor main's AWS
Graviton RaBitQ optimization. That work can be redone, but only
against the post-optimization baseline — otherwise we replay the
same conflict.

Task 51 is the active IVF RaBitQ optimization round. While Task 51
is open, every IVF safe-fn lift this task would land is at risk of
conflicting with Task 51's quantizer / scan-hot-path changes.

## Gate

This task opens **only when both of**:

1. Task 51 (IVF RaBitQ Second Optimization Round) closes with all
   its §Exit Criteria met.
2. Tasks 52 (P8), 53 (P6), and 54 (P3) Phase-1 wrappers have landed.

Until then, IVF residual unsafe blocks stay where they are.

## Non-Goals

- Do not start before the gate opens. Reviewer enforces this.
- Do not touch HNSW / SPIRE / DiskANN when this task opens.
- Do not refactor RaBitQ scoring math. Quantizer surface is Task 51's
  domain; this task only handles the structural unsafe-block shape.
- Do not change posting layout v2 boundaries. Storage format is
  outside this task's scope.

## Scope

Audit and structurally reduce `unsafe { ... }` blocks across
`src/am/ec_ivf/`. Refresh exact counts at task open (Task 51 will
have shifted them).

Particular attention to `scan.rs` (69 blocks). The 448 closeout
records main-side IVF scan work that came in via the merge:

> hashbrown imports, NEON inner_product_neon kernel, Cauchy-Schwarz
> pre-prune via running_top_k_for_pruning, posting.heaptid_count()
> precompute, _and_bits at store_scan_prepared_query and
> materialize_probe_candidates, differential NEON tests.

Those changes are Task 51's substance. This task's lifts must
preserve them.

## Techniques

Same as Task 50 + Phase-1 consumer pattern:

1. **Consume Phase-1 P6 datum wrappers** in `insert.rs` and
   `scan.rs` for `Datum → EcVector` extraction.
2. **Consume Phase-1 P3 page/WAL wrappers** in `build.rs`,
   `page.rs`, and `vacuum`-side paths.
3. **Consume Phase-1 P8 wrappers** for `build_parallel` paths if IVF
   has them (verify at task open).
4. **Safe-fn lifts** for scan helpers whose bodies compose already-
   safe operations after Phase-1 wrappers consumed.
5. **Re-apply** any pg_am_callback wrap or guard wrapping the merge
   dropped (per `ebb022a7a` commit message:
   "branch's pg_am_callback! macro wrap on ec_ivf_build_callback
   dropped. Task 50 can re-apply later") — that re-application now
   lives here, not on the original burndown branch.

## Migration Targets

Target -40% subsystem-wide, modulo structural-ceiling files.

**Aspirational subsystem total**: 109 → ≤ 65 (-40%).

## Slice and Packet Rules

Same as Tasks 50 / 52-56.

## Performance Gate

IVF's recall + QPS bench is the relevant lane. The Task 51 closeout
will leave a current AWS / M5 baseline; this task's bench gate is
"no regression vs that baseline."

Required evidence at task close:

- Re-run the Task 51 closeout bench suite on the post-burndown HEAD.
- Recall + latency + storage must not regress beyond noise.

## Validation

Standard validation suite (fmt + check + clippy + focused tests +
block counts + `src/` total).

## Exit Criteria

Task closes when:

- IVF subsystem block count ≤ 65.
- Each touched file ≥ -30% reduction or documented ceiling.
- IVF bench against the Task 51 baseline shows no regression.
- A closing summary packet records final per-file distribution,
  Phase-1 wrappers consumed, any wrapper extensions, `src/` total
  change, the pg_am_callback re-application, and structural-ceiling
  rationale for sub-30% files.

## Coordination

- **Hard gate**: do not open until Task 51 closes AND Phase-1 (Tasks
  52/53/54) have closed.
- Reviewer scope-lock: IVF-only on `task-57-ivf-burndown` branch.
- No overlap with Task 56 (SPIRE) or Task 55 (DiskANN) — different
  AMs.

## Cross-References

- Inherits Task 50 §Performance Gate template.
- Consumes wrappers from Tasks 52, 53, 54.
- Memory rule `feedback_main_priority_in_conflicts` (2026-05-22)
  governs the gate decision.
- `ebb022a7a` merge commit names the IVF lifts the burndown branch
  had to drop; those are this task's first re-application targets.
- Task 51 is the upstream gate.
