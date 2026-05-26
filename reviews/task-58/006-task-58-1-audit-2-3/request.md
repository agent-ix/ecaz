# Task 58.1 / slice 003 — Audit 2 + 3 (delegation absorption + dispatch typing)

**Branch:** `task-58-1-floor-recovery`
**Commit:** `37390579c` "Task 58.1/003: Audit 2 + 3 — unsafe-fn delegation absorption (84 → 74)"
**Parent packet:** `reviews/task-58/004-floor-recovery-followup/` (plan approved at `f64d7a4a7`)
**Predecessor:** slice 002 + 002.1 (`d7d019aa7`, `387103152`) — full approval logged by reviewer pre-slice-003.

## Scope

Audit 2 (per-call `unsafe fn` delegation absorption) and Audit 3
(LWLock dispatch table typing) from the Task 58.1 plan.

The slice 002.1 doc-parity work made every `unsafe fn` in this file
carry a fn-level `/// # Safety` contract. Audit 2 then removes the
inner `unsafe { ... }` blocks whose contract is already covered by
the outer's contract.

Audit 3 (LWLock dispatch typing) was already in place pre-slice
(the `EcHnswConcurrentDsmLockOps` dispatch table carries
`unsafe fn` function-pointer fields, and `shared` / `exclusive`
are themselves `unsafe fn`). This slice consumed those two single-
block bodies via Audit 2.

## Removed inner blocks (10 total)

| Fn                                          | Lines |     Blocks |
|---------------------------------------------|------:|-----------:|
| `EcHnswConcurrentDsmLockOps::shared`        |   571 |       -1   |
| `EcHnswConcurrentDsmLockOps::exclusive`     |   591 |       -1   |
| `EcHnswConcurrentDsmGraphLayout::from_header` |   908 |     -1   |
| `estimate_chunk`                            |  3309 |       -1   |
| `estimate_keys`                             |  3321 |       -1   |
| `test_lock_guard`                           |  4302 |       -1   |
| `try_parallel_build`                        |  2108 |       -2   |
| `try_parallel_concurrent_dsm_graph_build`   |  2181 |       -2   |
| **Total**                                   |       | **-10**    |

Pessimistic plan estimate was -8 (Audit 2 -3 + Audit 3 -5);
optimistic -10. Achieved -10, matching optimistic.

## Reviewer discipline preserved

Per the BLOCK feedback's rule:

> Where the inner block expresses a tighter SAFETY than the
> outer (e.g., an outer-fn `# Safety` that is genuinely broader
> than one specific call), **keep** the inner block — soundness
> over metric.

Inner blocks **kept** (not absorbed) in:

- `EcHnswParallelBuildLeader::begin` (9 blocks): leader
  construction with distinct FFI calls (`EnterParallelMode`,
  `CreateParallelContext`, per-key `shm_toc_*`,
  `RegisterSnapshot`, `InitializeParallelDSM`,
  `LaunchParallelWorkers`, `WaitForParallelWorkersToAttach`).
  Each inner block's SAFETY narrows the outer's broad
  AM-build-callback contract.
- `EcHnswParallelGraphBuildLeader::begin` (8 blocks): same
  pattern for the graph-build phase.
- `parallel_build_worker_main` (7 blocks): heap-scan worker
  entrypoint; distinct unsafe operations over DSM, worker
  number, queue handles, snapshot registration.
- `parallel_graph_build_worker_main` (5 blocks): graph-build
  worker entrypoint; same multi-FFI shape.
- `drain_worker_messages` (5 blocks): per-queue `shm_mq_receive`
  loop with distinct SAFETYs around handle access, message
  decode, and per-tuple ownership.
- `insert_leader_partitions` (2 blocks): leader-side write +
  layout attach.

These six bodies hold the residual 36 inner blocks. They are
honest structural ceiling: removing any one of them would
either inline the SAFETY rationale to a place where the reader
can't see it, or merge the rationale into a less specific outer
doc.

## Metrics

| | pre-slice (after 002.1) | post-slice |
|---|---:|---:|
| `unsafe { ... }` blocks in `build_parallel.rs` | 84 | **74** |
| Cumulative reduction from Task 50 baseline (112) |  | **-33.9%** |
| Task 58.1 §Exit floor (≤78) gap | -6 | **+4 margin** |
| Task 58 §Exit target (≤70) gap | -14 | **-4** (structural) |
| `/// # Safety` parity (15 fn / 15 docs) | ✓ | ✓ |

## Validation

- `cargo check --no-default-features --features pg18,bench` — passes.
- `cargo clippy --no-default-features --features pg18 --lib` — 0
  hits in `src/am/ec_hnsw/build_parallel.rs`. Pre-existing
  crate-wide clippy backlog unrelated (Task 52/002 reviewer
  note).
- Doc parity grep pair holds (15 / 15).
- macOS `dyld _BufferBlocks` blocker
  (`feedback_dyld_buffer_blocks_known`) applies; compile-gate
  sufficient.

## Bench gate

Deferred to slice 004 closeout per Task 58.1 plan §"Performance
gate". This slice is structural-only — removed inner blocks
replace the same FFI call sequence with no semantic difference;
the optimizer sees the same generated code. No worker-loop hot
path was touched.

## Disposition request

Slice 003 disposition request: approve and clear for slice 004
closeout (bench gate + final residual-ceiling rationale).

Per the parent packet's reviewer feedback:

> Wait for reviewer signoff between slice 002 commit and slice
> 003 (and between any subsequent slices).

Awaiting reviewer signoff before opening slice 004.

## Memory rules in play

- `feedback_no_premature_task_close` — slice closes the floor gap
  with margin; no premature close.
- `feedback_dont_defer_safety_fixes` — doc parity 15/15 carried
  through the slice; no safety regression.
- `feedback_view_operations_not_accessors` — Audit 1's `with_*`
  ops still in place; not regressed.
- `feedback_anti_pattern_b_unbounded_lifetime` — no new safe
  `fn(*mut T) -> &'a T` introduced.
- `feedback_full_code_review` — per-slice diff is the 10
  block-removal edits + comment rewrites; small enough for
  full read.
- `feedback_branch_isolation` — slice touches only
  `src/am/ec_hnsw/build_parallel.rs`.

## Cross-references

- Parent packet: `reviews/task-58/004-floor-recovery-followup/request.md`
- Approving feedback for slice 002 + 002.1: forthcoming reviewer file
  in `reviews/task-58/005-task-58-1-audit-1/feedback/` (logged
  inline by the operator)
- Originating BLOCK: `reviews/task-58/003-closeout/feedback/2026-05-23-01-reviewer.md`
- Slice 003 commit: `37390579c`
- Slice 003 metric artifact: `reviews/task-58/006-task-58-1-audit-2-3/artifacts/manifest.md`
