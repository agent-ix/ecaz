# Task 58: HNSW `build_parallel.rs` P8 Consumer Migration

Status: **proposed** — second HNSW deep-burndown follow-up to Task 54.
Targets the largest remaining HNSW unsafe-block file
(`src/am/ec_hnsw/build_parallel.rs` = 112 blocks) by consuming the
Task 52 P8 typed-view wrappers (DSM / atomic / SpinLock).

## Why

Post-Task-54 HNSW residue is dominated by `build_parallel.rs` at 112
unsafe blocks — by itself larger than the entire DiskANN subsystem
(38 post-Task-55) or the entire common-wrapper surface. These come
from the concurrent-DSM parallel-build path:

- Raw `*mut EcHnswConcurrentDsmGraph{Header,Node,...}` accessors
  (`&*self.header`, `&mut *self.header`, `self.nodes.add(idx)`,
  `&*self.node_ptr(idx)`, `ptr::addr_of_mut!((*node).lock)`)
- `crate::am::common::dsm::PgAtomicU32Ref::from_raw(self.0)` — already a
  typed view, but the unsafe-fn boundary remains
- Function-pointer indirection (`(self.acquire_shared)(lock)`,
  `(self.acquire_exclusive)(lock)`) — DSM-resident lock dispatch
- Raw `*mut` derefs for header / node / neighbor-slot / code / source
  views after the layout-resolution split (`from_header`,
  `EcHnswConcurrentDsmGraphLayout::from_header`)
- `slice::from_raw_parts` over DSM-backed node arrays

The Task 52 P8 wrapper surface (typed DSM / atomic / SpinLock views
under `src/am/common/dsm.rs`) is the consumption target. Task 52
proved out the view pattern; this task graduates HNSW's parallel-build
consumer sites onto that surface.

## Non-Goals

- Do not touch HNSW scan path (`scan.rs`, ~74 unsafe blocks) — that's
  a separate hot-path file with its own bench gate considerations.
- Do not touch DiskANN / SPIRE / IVF — those have their own tasks.
- Do not reshape DSM segment layout. The on-disk + in-memory format
  is invariant; this task is purely a structural unsafe-block lift.
- Do not optimize the parallel-build algorithm. Scoring, neighbor
  selection, and refinement passes are unchanged.
- Do not refactor `concurrent_dsm_state.rs` (already 0 unsafe).

## Scope

Audit and structurally reduce `unsafe { ... }` blocks in
`src/am/ec_hnsw/build_parallel.rs` (and only that file). Current
distribution:

| Surface (approx., from `grep -n`) | Block count |
| --- | ---: |
| `PgAtomicU32Ref::from_raw` consumer wraps | ~5 |
| Header + node + neighbor-slot + codes + sources DSM accessors | ~30 |
| Lock-dispatch function pointer wraps | ~8 |
| `slice::from_raw_parts` over DSM arrays | ~15 |
| GenericXLog / page write paths in worker loops | ~8 |
| Heap-tuple / source-vector decode in worker drain | ~10 |
| Remaining unsafe-fn chain wraps | ~36 |
| **Total** | **112** |

## Techniques

1. **P8 typed-view consumption**: graduate raw `*mut EcHnswConcurrentDsmGraph*` 
   accessors to Task 52's DSM view wrappers
   (`*View<'a>` operations, not raw `fn(&self) -> &'a T` accessors —
   per `feedback_view_operations_not_accessors`).
2. **Handle-ratchet for relation/buffer access**: same pattern as
   Tasks 54/55 — `LockedBufferGuard::read_main_handle` etc.
3. **Page/WAL P3 consumption** for worker-side page-mutation paths.
4. **Datum P6 consumption** in heap-tuple drain.
5. **Narrow block scoping** + consolidating adjacent unsafe blocks
   that share a SAFETY proof.

## Migration Target

| File | Now | Target | Δ |
| --- | ---: | ---: | --- |
| `build_parallel.rs` | 112 | **≤ 70** (-37%) | -42 |

Per Task 50 §Exit Criteria, ≥ -30% per-module is the floor unless a
structural-ceiling rationale is documented. The -37% target gives
margin for the irreducible DSM-resident lock dispatch + per-node
SIMD math residue.

## Slice and Packet Rules

Same as Tasks 54 / 55. Specifically:

- Each packet must report `unsafe { ... }` block count before / after
  for `build_parallel.rs` plus `src/` total.
- Bench evidence is per task, not per slice. The parallel-build bench
  gate is **explicit**: HNSW 100k build wall-clock under
  `parallel_workers > 0` config must not regress beyond the 5% noise
  band vs the post-Task-54 baseline at
  `benchmarks/task-50-m5-hnsw-baseline/`.
- If a slice has to extend a Task 52 P8 wrapper, the extension lands
  separately in `src/am/common/dsm.rs` with its own commit.

## Performance Gate

The parallel-build path is performance-sensitive. Required evidence
at task close:

- HNSW 100k build with `parallel_workers ∈ {0, 2, 4}` (matching the
  Task 50/449 M5 baseline parameters if the baseline includes them;
  otherwise run all three as the new reference).
- Build wall-clock per worker count must not regress beyond 5% noise
  band.
- Recall + per-row storage from a follow-up scan must be bit-for-bit
  identical vs the post-Task-54 baseline (the wrappers are call-site
  moves; format must be unchanged).
- Acceptance evidence is artifact-committed under the closeout
  packet, with `before-after-summary.md` matching Task 53/54/55
  format.

## Validation

- `cargo fmt --all`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings`
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
- focused `cargo pgrx test pg18 ec_hnsw::build_parallel::*` if the
  dyld `_BufferBlocks` macOS blocker has cleared by then; otherwise
  compile-gated + bench-gated only
- direct unsafe-block count per touched file
- `src/` total snapshot

## Exit Criteria

Task closes when:

- `src/am/ec_hnsw/build_parallel.rs` ≤ 70.
- HNSW parallel-build wall-clock under the bench sweep does not
  regress beyond 5% noise band vs post-Task-54 baseline.
- Recall + per-row storage from a follow-up scan are bit-for-bit
  identical at all sweep points.
- A closing summary packet records:
  - per-file before/after for `build_parallel.rs`;
  - explicit list of Task 52 P8 wrappers consumed;
  - any Task 52 P8 wrapper extensions landed (and the owning Task
    52 commit SHAs);
  - the `src/` total block count change;
  - structural-ceiling rationale for the residual ~70 blocks
    (DSM-resident lock dispatch, per-node SIMD math, raw layout
    accessors that the view pattern can't safely cover).

## Coordination

- **Gate**: opens only after Tasks 52 (P8) has closed (it has — done
  pre-session).
- **Sibling tasks**: Tasks 56 (SPIRE) and 57 (IVF) remain deferred
  per their own gate criteria; this task is in parallel with those
  staying gated.
- **Reviewer scope-lock**: `build_parallel.rs` only on the owning
  branch (`task-58-hnsw-build-parallel-p8`).
- **No overlap** with the residual HNSW scan-path work — `scan.rs` is
  out of scope.
- **Memory rules**:
  - `feedback_view_operations_not_accessors` — DSM view wrappers
    must expose operations, not raw `&'a T` returns.
  - `feedback_anti_pattern_b_unbounded_lifetime` — no
    `fn(*mut T) -> &'a T`; use `unsafe fn` or inline `NonNull::as_ref`.
  - `feedback_main_priority_in_conflicts` — if `build_parallel.rs`
    conflicts with main between branch-open and close, main wins on
    substance; redo lifts against the optimized baseline.

## Cross-References

- `plan/tasks/54-common-p3-page-wal-wrappers.md` — Phase-1 P3 wrappers (consumed here for worker page-mutation paths).
- `plan/tasks/52-common-p8-build-parallel-typed-views.md` — Phase-1 P8 wrappers (primary consumption surface for this task).
- `plan/tasks/55-diskann-unsafe-burndown.md` — proves the cross-AM consumption pattern.
- `reviews/task-54/005-closeout/request.md` — flags HNSW
  parallel-build as the deferred large-file lift (stretch consumer).
- `reviews/task-54-followup-hnsw-stretch/001-consumer-migration/request.md` — proves consumer-side lift pattern on insert.rs + shared.rs.
- `benchmarks/task-50-m5-hnsw-baseline/manifest.md` — pre-state baseline for the build wall-clock + recall + storage gate.
