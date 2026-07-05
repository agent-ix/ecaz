# Task 58 Packet 001 — Execution Plan: `build_parallel.rs` P8 Consumer Migration

Status: **plan**

## Baseline (HEAD = task-58 from main `528fb6a74`)

| File | Unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | **112** |
| §Exit target | ≤ 70 (-37%) |
| §Exit floor (per Task 50) | ≤ 78 (-30%) without structural-ceiling rationale |
| `src/` total | 904 |

## Unsafe-block categorization (112 total)

Inventory from `grep -n "unsafe {"` and surrounding context:

| Category | Approx count | Migration path |
| --- | ---: | --- |
| Accessor methods returning `&T` (`fn(&self) -> &EcHnswConcurrentDsmGraphHeader`, `&EcHnswConcurrentDsmNode`) | 6 | Replace with `with_*` closure operations per `feedback_view_operations_not_accessors` |
| Raw pointer arithmetic (`addr_of_mut!`, `.add(offset)`) without deref | ~12 | Wrap behind safe-fn returning `*mut T` (raw pointers are themselves safe to return); push `unsafe` inside the wrapper |
| `slice::from_raw_parts{,_mut}` over DSM arrays | ~10 | Wrap behind safe accessor on `EcHnswConcurrentDsmGraphParts` that takes a layout reference for bounds; SAFETY contract moves to wrapper construction |
| Function-pointer dispatch (`(self.acquire_shared)(lock)`) for LWLock guards | ~6 | Lock dispatcher exposes safe `shared(lock_view)` / `exclusive(lock_view)` operations that internalize the unsafe-fn pointer call |
| PG FFI calls (`EnterParallelMode`, `shm_toc_lookup`, `ii_ParallelWorkers` deref, leader/worker drain, etc.) | ~30 | Mostly structural ceiling; consolidate adjacent unsafe blocks where they share a SAFETY proof |
| Atomic/spinlock operations | ~3 | Already wrapper-internal via Task 52 `PgAtomicU32Ref` / `SpinLockGuard` surface — minimal additional work |
| `unsafe fn` call wraps (chained worker / leader / drain helpers) | ~25 | Bottom-up: when a callee becomes safe `fn`, drop its caller's `unsafe { ... }` wrap |
| Bytemuck / cast at DSM segment boundaries | ~10 | Wrap behind safe-fn accessors on the layout struct |
| Misc (heap-tuple decode, source-vector decode in worker loop) | ~10 | Consume P6 datum wrappers from Task 53 where applicable |

## Slice plan

1. **001 — plan** (this packet). No code.

2. **002 — Accessor → operation methods** on `EcHnswConcurrentDsmGraphParts`. Replace:
   - `fn header(&self) -> &EcHnswConcurrentDsmGraphHeader` → `fn with_header<R>(&self, f: impl FnOnce(&Header) -> R) -> R`
   - `fn header_mut(&mut self) -> &mut Header` → `fn with_header_mut<R>(&mut self, f: impl FnOnce(&mut Header) -> R) -> R`
   - `fn node(&self, idx) -> &Node` → `fn with_node<R>(&self, idx, f: impl FnOnce(&Node) -> R) -> R`
   - `fn node_mut(&mut self, idx) -> &mut Node` → `fn with_node_mut<R>(...)`
   
   Net delta: callers that did `let node = parts.node(idx); use(node)` become `parts.with_node(idx, |node| use(node))`. The unsafe block moves into the wrapper's body (still wrapper-internal). Caller-side wraps drop.
   
   Target slice delta: **-6 to -12 src/**.

3. **003 — Pointer arithmetic + slice::from_raw_parts wrappers**. Add safe accessors on `EcHnswConcurrentDsmGraphParts` that take a layout reference for bounds: `nodes_slice(&self, layout) -> &[Node]`, `codes_slice(&self, layout) -> &[u8]`, `sources_slice(&self, layout) -> &[f32]`, etc. The SAFETY contract is enforced by requiring `&Layout` (which is itself only constructed via the validated `from_header` path).
   
   Target slice delta: **-8 to -15 src/**.

4. **004 — Function-pointer dispatch unification**. The `EcHnswConcurrentDsmLockOps` shared/exclusive methods consume the function-pointer through `LwLockGuard::acquire_*` which is already a safe-fn (the dispatch ABI is what's unsafe). Wrap the dispatch in a typed view that exposes safe `shared(lock_view) -> LwLockGuard` / `exclusive(lock_view) -> LwLockGuard`.
   
   Target slice delta: **-4 to -8 src/**.

5. **005 — Worker-side P3 + P6 consumption**. The worker drain and leader page-mutation paths consume Tasks 53/54 wrappers. Same pattern as Task 54's `write_data_pages` and Task 55's DiskANN `apply_tuple_rewrites_handle`.
   
   Target slice delta: **-5 to -10 src/**.

6. **006 — Bench gate + closeout**. HNSW build at `parallel_workers ∈ {0, 2, 4}` against `benchmarks/task-50-m5-hnsw-baseline/` and the Task 54 latency window. No wall-clock regression beyond 5%, recall + per-row storage bit-for-bit identical.

## Migration patterns

- **Operation, not accessor**: `with_header(|h| h.field)` instead of `header().field`. Enforces `feedback_view_operations_not_accessors`.
- **Layout-bounded slice**: `nodes_slice(layout: &Layout)` validates `layout.node_count * size_of::<Node>() ≤ segment_size` inside the unsafe wrapper, so callers get a safe `&[Node]`.
- **PG-extern boundary kept**: `EnterParallelMode`, `shm_toc_lookup`, `ii_ParallelWorkers`, leader/worker FFI shells stay unsafe at the boundary. Structural ceiling per Task 50/448.

## Validation gates (per slice)

Same as Tasks 54 / 55:

- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings`
- per-slice `unsafe { ... }` count for `build_parallel.rs`
- `src/` total

Bench gate (slice 006 only): `ecaz bench suite` against
`benchmarks/task-50-m5-hnsw-baseline/` plus a new build-time bench
at `parallel_workers ∈ {0, 2, 4}`.

## Out of scope

- HNSW scan path (`scan.rs`) — separate task if ever opened.
- DiskANN / SPIRE / IVF — own tasks.
- DSM segment layout changes — out per §Non-Goals.
- Parallel-build algorithm changes — out per §Non-Goals.

## Realistic close estimate

Hitting -37% (≤ 70) is the stretch target. Hitting the -30% floor
(≤ 78) is the close gate. Initial estimates per category suggest
realistic delta of **-25 to -45 blocks**. The exact number depends
on how many `&T`-returning accessor sites I can convert to
operations without breaking the parallel-build coordination
invariants. If I land below -37% but above -30%, closeout includes
a structural-ceiling rationale for the residue.

## References

- `plan/tasks/58-hnsw-build-parallel-p8-consumer-migration.md`
- `plan/tasks/52-common-p8-build-parallel-typed-views.md` (Phase-1 P8 surface — primary consumption)
- `src/am/common/dsm.rs` (`PgAtomicU32Ref`, `SpinLockGuard`, `ConditionVariableRef`)
- `reviews/task-54/005-closeout/request.md` (HNSW residue analysis flagging this work)
- `reviews/task-55/002-consumer-migration/request.md` (cross-AM proof of wrapper consumption)
