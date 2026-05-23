# Task 52 / 001 — Execution Planning

Branch: `task-52` (renamed from `task-50-diskann` after Task 50 close)
Head SHA: `0d01a77c0`
Task source: `plan/tasks/52-common-p8-build-parallel-typed-views.md`

## Summary

Open Task 52 — the first Phase-1 lane in the post-Task-50 hardening
sequence. This packet does not land a code change. It records the
pre-state baseline, surveys the consumer surface in
`src/am/ec_hnsw/build_parallel.rs`, and lays out the slice plan that
subsequent packets (002+) execute against.

No reviewer disposition is required to land slice 002; this packet is
posted for visibility and to anchor the closeout's before/after delta
math.

## Pre-state baseline

From `artifacts/baseline-unsafe-density.txt`:

| Surface | Pre-state `unsafe { ... }` blocks |
| --- | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 112 (Task 52 primary target) |
| `src/am/common/dsm.rs` (slice-447 base) | 9 |
| `src/` total | 960 |

Task 50 §Exit summary closed HNSW at 549 → 327 (-40.44%). Task 52
chips the documented 112-block ceiling on `build_parallel.rs` by
landing the four typed views named in `plan/tasks/52-...md` §Scope
and migrating the HNSW consumer sites to them.

**Target**: `build_parallel.rs` 112 → ≤ 80 (-29% or better).

## Consumer-site survey

`artifacts/build-parallel-consumer-sites.txt` enumerates every shm_toc /
SpinLock / ConditionVariable / `(*shared)` / `(*pcxt)` touch in
`src/am/ec_hnsw/build_parallel.rs`. Three observations from that survey:

1. **There is exactly one `EcHnswParallelBuildSharedHeader`**, used by
   both the heap-scan worker entry (`parallel_build_worker_main`) and
   the graph-build worker entry (`parallel_graph_build_worker_main`).
   The task spec §Scope names "two distinct shared headers" — that does
   not match the code. The plan absorbs both phases into a **single**
   `EcHnswParallelBuildSharedView<'a>` rather than authoring two
   near-identical views.
2. **The SpinLock + mutate + Release + CV-Signal compound is doubled**
   (lines 2828–2833 and 2897–2902). Both copies feed
   `(*shared).record_worker_counts(...)` with phase-specific deltas.
   The view's `record_workers_done` method takes the (scanned, encoded)
   delta pair, so both call sites consume the same safe method.
3. **`shm_toc_lookup_required<T>` already exists locally** in
   `build_parallel.rs:1997` as the worker-side null-check wrapper.
   `ShmTocReader::lookup_required` supersedes it; the local helper goes
   away as a side effect of slice 002, shaving one item off the file.

## Slice plan

Each slice is a separate code commit + a matching review-request commit,
both pushed before the next slice begins (per
`feedback_coder_push_smoke_checks`).

| Slice | Packet | Scope | Exit condition |
| --- | --- | --- | --- |
| 002 | `002-shm-toc-wrappers` | Add `ShmTocBuilder<'a>` and `ShmTocReader<'a>` to `src/am/common/dsm.rs`. Wrapper-only. | `cargo check + clippy` clean on `pg18,bench`. |
| 003 | `003-parallel-build-shared-view` | Add `EcHnswParallelBuildSharedView<'a>` (both phases). Absorbs SpinLock+CV compound into `record_workers_done`. | clippy clean; view exercised by a doc-comment example only. |
| 004 | `004-build-parallel-consumer-migration-shm-toc` | Migrate the four shm_toc allocate/insert/lookup chains (×2 leader, ×2 worker) over to the new wrappers. | per-file before/after counts in packet; clippy clean. |
| 005 | `005-build-parallel-consumer-migration-shared-view` | Migrate the two SpinLock+CV worker compounds + the two leader-side init compounds to the view's safe methods. | per-file before/after counts; focused `cargo pgrx test pg18 ec_hnsw::build_parallel` smoke. |
| 006 | `006-build-parallel-deref-cleanup` | Final pass over residual `(*shared).field` / `(*pcxt).field` derefs. Land iff the target file is still > 80; otherwise close at slice 005. | `build_parallel.rs` ≤ 80; per-file before/after. |
| 007 | `007-task-52-closeout` | Closing summary: per-file deltas, full common-wrapper surface added, `src/` total delta, **bench-window evidence** vs `benchmarks/task-50-m5-hnsw-baseline/`. | All four task-52 §Exit Criteria satisfied. |

The "graph-build distinct shared header" §Scope #2 collapses into the
single shared-view slice (003). The remaining four §Scope wrappers map
to slices 002–005 1:1.

## Non-goals (restated from task spec)

- No touch to `src/am/ec_ivf/**`, `src/am/ec_spire/**`, or
  `src/am/ec_diskann/**`.
- No extension of `dsm.rs` beyond what `build_parallel.rs` demands.
- No per-slice bench run. Bench window opens once at task close per
  `feedback_coder_push_smoke_checks` and the task spec.
- No re-shaping of slice-447 `PgAtomicU32Ref` consumers.

## Coordination

- Phase-1 lane — runs before Tasks 53 (P6) and 54 (P3).
- HNSW-only consumer migration. SPIRE/IVF parallel-build are deferred to
  Tasks 56/57.
- No overlap with Task 51 (IVF RaBitQ optimization) — IVF parallel-build
  is not touched here.
- Coder pushes per memory `feedback_coder_push_smoke_checks` (smoke
  checks between slices; bench window once at close).

## Artifacts (in this packet)

- `artifacts/manifest.md` — packet-local manifest, per CLAUDE.md.
- `artifacts/baseline-unsafe-density.txt` — pre-state file-level counts.
- `artifacts/build-parallel-consumer-sites.txt` — consumer-site survey.

## Cross-references

- Supersedes: `reviews/task-50/448-hnsw-burndown-refreshed-closeout`
  §"Next-highest-density modules" P8 continuation queue.
- Builds on: `reviews/task-50/447-p8-dsm-typed-wrappers/` (slice that
  opened the P8 module).
- Bench gate consumes: `benchmarks/task-50-m5-hnsw-baseline/manifest.md`
  at task close (not in this planning packet).
