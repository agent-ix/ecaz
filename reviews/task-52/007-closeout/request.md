# Task 52 / 007 — Closeout

Branch: `task-52`
Task source: `plan/tasks/52-common-p8-build-parallel-typed-views.md`

## Summary

Task 52 closes with all four §Exit Criteria satisfied. The P8 typed-view
wrapper surface for HNSW parallel build is complete; `build_parallel.rs`
is at the documented ≤ 80 unsafe-block ceiling; and the bench gate is
satisfied against `benchmarks/task-50-m5-hnsw-baseline/` (see
`artifacts/` evidence).

## §Exit Criteria status

| # | Criterion | Status |
| - | --- | --- |
| 1 | Four typed views in `src/am/common/dsm.rs` or sibling modules | **✓** |
| 2 | `src/am/ec_hnsw/build_parallel.rs` block count ≤ 80 | **✓ (80)** |
| 3 | HNSW recall + QPS no regression vs post-Task-50 baseline | **✓** (artifacts) |
| 4 | Closing summary packet with per-file before/after, full wrapper surface, src/ total | **✓** (this packet) |

### §Exit Criterion #1 — Wrappers landed

| Wrapper | Location | Slice |
| --- | --- | ---: |
| `ShmTocBuilder<'a>` | `src/am/common/dsm.rs` | 002 |
| `ShmTocReader<'a>` | `src/am/common/dsm.rs` | 002 |
| `EcHnswParallelBuildSharedView<'a>` | `src/am/ec_hnsw/parallel_build_view.rs` | 003 + 005 |
| `ParallelContextRef<'a>` | `src/am/common/parallel_context.rs` | 006 |

The original spec named **two distinct** shared-header views (heap-scan
+ graph-build). The 001 planning packet established that the code
reuses a single `EcHnswParallelBuildSharedHeader` across both phases;
the 001 reviewer concurred (`reviews/task-52/001-execution-planning/feedback/2026-05-23-01-reviewer.md`).
One view absorbs both phases via `record_workers_done(scan_delta,
encoded_delta)` consumed identically by `parallel_build_worker_main`
and `parallel_graph_build_worker_main`.

The 4th wrapper (`ParallelContextRef<'a>`) was added in slice 006 —
beyond the originally-named four — to clear the `(*pcxt).field`
deref residuals and the PG lifecycle calls. The original spec did
not anticipate the need; the §Scope of "wrappers where the wrapper
deserves its own file" covers it.

### §Exit Criterion #2 — `build_parallel.rs` ≤ 80

| Slice | `build_parallel.rs` | Δ vs prior |
| --- | ---: | ---: |
| Pre-Task-52 (HEAD before slice 001) | **112** | — |
| Post-001 (planning only, no code) | 112 | 0 |
| Post-002 (ShmToc wrappers added) | 112 | 0 |
| Post-003 (View wrapper added) | 112 | 0 |
| Post-004 (shm_toc consumer migration) | 107 | -5 |
| Post-005 (SpinLock+CV compound migration) | 105 | -2 |
| Post-006 (ParallelContextRef + deref cleanup) | **80** | **-25** |
| **Total** | **80** | **-32 (-28.6%)** |

### §Exit Criterion #3 — Bench gate

Bench suite `reviews/task-52/007-closeout/artifacts/suite.json` ran
against the post-Task-50 baseline corpus (`ec_real_10k_hnsw`).
Headline comparison vs `benchmarks/task-50-m5-hnsw-baseline/artifacts/`:

See:
- `artifacts/suite-manifest.json` — bench run audit trail.
- `artifacts/recall-ec_real_10k-hnsw.log` — recall@10 across ef
  sweep.
- `artifacts/latency-ec_real_10k-hnsw.log` — p50/p95/p99 latency
  across ef sweep.
- `artifacts/before-after-summary.md` — explicit numeric
  comparison vs baseline, with tolerance assessment.

Acceptance: same regression tolerance as Task 50 (functional + the
new M5 forward-baseline, per memory `feedback_main_priority_in_conflicts`'s
forward-only framing).

### §Exit Criterion #4 — Closing summary

#### Per-file before/after for `build_parallel.rs`

See §Exit Criterion #2 above. Per-slice deltas in each slice's
`artifacts/manifest.md`.

#### Other HNSW files touched

| File | Touch type | unsafe-block delta |
| --- | --- | ---: |
| `src/am/ec_hnsw/parallel_build_view.rs` (new) | view wrapper added | +12 |
| `src/am/ec_hnsw/build_parallel.rs` | consumer migration + visibility opens + getter additions | -32 |
| `src/am/ec_hnsw/mod.rs` | module declaration | 0 |

No other HNSW files touched.

#### Full `src/am/common/` wrapper surface added

| File | Wrappers added |
| --- | --- |
| `src/am/common/dsm.rs` | `ShmTocBuilder<'a>`, `ShmTocReader<'a>` (extend slice-447 `PgAtomicU32Ref`/`SpinLockGuard`/`ConditionVariableRef` set) |
| `src/am/common/parallel_context.rs` (new) | `ParallelContextRef<'a>` + safe free fns: `enter_parallel_mode`, `exit_parallel_mode`, `instr_start_parallel_query`, `index_info_parallel_workers` (null-safe), `index_info_is_concurrent` (null-safe), `shm_mq_set_sender`, `table_parallelscan_estimate` |
| `src/am/common/mod.rs` | `pub(crate) mod parallel_context;` |

#### `src/` total block count change

| Snapshot | `src/` total |
| --- | ---: |
| Pre-Task-52 (HEAD before slice 001) | 960 |
| Post-Task-52 (HEAD at this packet) | **963** |
| **Δ** | **+3** |

The +3 net is the structural cost of the wrapper investment:
- `src/am/common/dsm.rs`: +4 (slice 002)
- `src/am/ec_hnsw/parallel_build_view.rs` (new): +12 (slices 003+005)
- `src/am/common/parallel_context.rs` (new): +19 (slice 006)
- `src/am/ec_hnsw/build_parallel.rs`: -32 (slices 004-006)
- All other files: 0

Total wrapper-side +35, consumer-side -32 = net +3.

**HNSW subsystem-wide:** 327 (post-Task-50) → 307 (now). The
subsystem itself reduced by -20 even after absorbing the +12
HNSW-side wrapper (parallel_build_view.rs).

## Audit-trail completeness vs the premature-close attempt

The reviewer attempted to close Task 52 at slice 004 (commit
`b907f30e0`, 2026-05-23, see
`reviews/task-52/004-build-parallel-shm-toc-migration/feedback/2026-05-23-02-reviewer.md`).
The coder rebutted that close via
`reviews/task-52/004-.../feedback/2026-05-23-03-coder.md` on the
grounds that:
- `build_parallel.rs` was at 107 (slice 004), not the documented
  ≤ 80 target.
- The bench window had not been opened against the post-Task-50
  baseline (§Exit Criterion #3).
- No formal closeout packet existed (§Exit Criterion #4).

That rebuttal was operator-aligned via `/goal complete 100% of
task 52` (2026-05-23). Slices 005 and 006 then chipped 107 → 80,
and this closeout packet records the bench gate + full delta
accounting.

The new memory rule `feedback_no_premature_task_close` (2026-05-23)
captures the disposition: reviewer drives coder to 100% of each
task plan's §Exit Criteria; structural-ceiling deferrals require
genuine language-level / ABI ceilings, not arbitrary "stop here"
calls.

## Anti-pattern B / view-operations discipline tally

- 9 applications of `feedback_anti_pattern_b_unbounded_lifetime`
  across slices 002, 003 (1 refactor mid-slice), 004, 005, 006.
- 3 applications of `feedback_view_operations_not_accessors` (saved
  as a tightening of anti-pattern B during slice 003 refactor; now
  blocks `fn(&self) -> &'a T` accessors on typed `*View<'a>` wrappers
  by default).

No safe `fn(*mut T) -> &'a T` authored in the wrapper surface. All
`Copy`-field reads return values; pointer fields return raw `*mut`
values; the type/initialization contract stays at the call site
where `unsafe { NonNull::new_unchecked(ptr).as_ref() }` is paired
with the surrounding scope's safety reasoning.

## Cross-references

- All slice packets: `reviews/task-52/00{1..7}-*/`.
- Pre-state baseline: `reviews/task-52/001-execution-planning/artifacts/`.
- Premature-close rebuttal:
  `reviews/task-52/004-.../feedback/2026-05-23-03-coder.md`.
- Bench evidence: this packet's `artifacts/`.
- Memory rules referenced:
  - `feedback_anti_pattern_b_unbounded_lifetime`
  - `feedback_view_operations_not_accessors`
  - `feedback_dont_overclaim_done`
  - `feedback_no_premature_task_close`
  - `feedback_main_priority_in_conflicts`
  - `feedback_dyld_buffer_blocks_known`
  - `feedback_coder_push_smoke_checks`
