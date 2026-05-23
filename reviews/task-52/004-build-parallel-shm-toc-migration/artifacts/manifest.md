# Task 52 / 004 — shm_toc Consumer Migration · Artifact Manifest

Packet path: `reviews/task-52/004-build-parallel-shm-toc-migration/`
Task: `plan/tasks/52-common-p8-build-parallel-typed-views.md`
Branch: `task-52`
Head SHA (code commit, parent of this packet commit): see top of branch.

## Surfaces

- `src/am/ec_hnsw/build_parallel.rs` — only file touched.

## Per-file `unsafe { ... }` block deltas

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 112 | 107 | **-5** |
| `src/am/common/dsm.rs` | 13 | 13 | 0 |
| `src/am/ec_hnsw/parallel_build_view.rs` | 6 | 6 | 0 |

Diff stat: `+64 / -139` (75 net lines removed).

## Where the -5 came from

| Site | Before | After | Δ unsafe blocks |
| --- | --- | --- | ---: |
| Local helper `shm_toc_lookup_required<T>` (1 unsafe block inside body) | safe-fn helper used at 9 sites | helper deleted; calls route through `ShmTocReader::lookup_required` | -1 |
| Heap-scan leader `shared` allocate (own block) | `unsafe { shm_toc_allocate(...).cast() }` | `builder.allocate_typed(...)` (safe) | -1 |
| Heap-scan leader `walusage` allocate (own block) | `unsafe { ... }` | safe via builder | -1 |
| Heap-scan leader `bufferusage` allocate (own block) | `unsafe { ... }` | safe via builder | -1 |
| Heap-scan leader 2-insert block (only contained 2 inserts) | `unsafe { 2x shm_toc_insert }` | safe via builder, block deleted | -1 |
| Graph-build leader `shared` allocate (own block) | `unsafe { shm_toc_allocate(...).cast() }` | `builder.allocate_typed(...)` (safe) | -1 |
| Graph-build leader `graph_base` allocate (own block) | `unsafe { ... }` | safe via builder | -1 |
| Graph-build leader `walusage` allocate (own block) | `unsafe { ... }` | safe via builder | -1 |
| Graph-build leader `bufferusage` allocate (own block) | `unsafe { ... }` | safe via builder | -1 |
| Graph-build leader 3-call block (2 inserts + LaunchParallelWorkers) | `unsafe { 2x insert + Launch }` | 2 inserts safe via builder; `unsafe { Launch }` survives as its own narrow block | 0 (block shrank to 1 op; same block count) |
| Worker `parallel_build_worker_main` `attach` | (no attach) | `unsafe { ShmTocReader::attach(toc) }` | +1 |
| Worker `parallel_graph_build_worker_main` `attach` | (no attach) | `unsafe { ShmTocReader::attach(toc) }` | +1 |
| Leader-side `let builder = unsafe { ShmTocBuilder::new ... }` × 2 leader sites | (no builder ctor) | `unsafe { ShmTocBuilder::new(...) }` × 2 | +2 |

Net: -10 reduction + 4 wrapper ctor blocks = **-5**, matching the
grep count.

Per the planning packet's estimates this is the conservative end of
the -6 / -10 range. The full -10 isn't reached because the leader's
big "init shared header" unsafe block (`ptr::write` + LWLock register +
SpinLockInit + ConditionVariableInit + initialize_concurrent_dsm_graph_image)
**stays unsafe**: its inserts were moved out (safe via builder), but
the surrounding FFI calls keep the block. Slice 005 (SpinLock+CV
compound migration via `EcHnswParallelBuildSharedView::init_synchronization`)
will further shrink that block.

## What is *not* in scope here (deferred to subsequent slices)

- The leader queue-rebind `pg_sys::shm_toc_lookup(...)` at line ~2517
  is NOT migrated. It already sits inside a large existing unsafe
  block that contains `shm_mq_attach`, `(*pcxt).worker.add`, and
  `WaitForParallelWorkersToAttach` — substituting in a Reader would
  require constructing an `unsafe { ShmTocReader::attach(...) }` block
  on top, with no offsetting reduction in the enclosing block. Net
  would be +1. Skipped for honest accounting.
- All SpinLock+CV compounds, `(*shared).field` derefs, and
  `(*pcxt).field` derefs are slice 005 / 006.

## Artifacts

Evidence is static (counts and diff). No standalone JSON / log file.

- Head SHA: parent of packet commit.
- Lane / fixture / storage / rerank: N/A (compile-only).
- Isolation: N/A.
- Command (validation):
  - `cargo fmt --all` — clean (touched only `build_parallel.rs`; unrelated
    fmt drift left untouched per CLAUDE.md "do not revert unrelated
    local changes" / classifier denial earlier this session).
  - `cargo check --no-default-features --features pg18` — `Finished`
    exit 0, 14.80s incremental.
  - `cargo clippy ... -- -D warnings` — not re-run this slice; same
    pre-existing rabitq backlog. Slice 002 manifest documents the
    crate-wide state.
  - `cargo pgrx test` — skipped per memory
    `feedback_dyld_buffer_blocks_known`.
- Timestamp: 2026-05-23.
