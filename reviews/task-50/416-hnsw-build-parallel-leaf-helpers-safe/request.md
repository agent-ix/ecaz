# Task 50/416: HNSW build_parallel.rs — three leaf-helper safe-fn lifts

## Why this slice

`build_parallel.rs` is now the densest HNSW file (114 blocks after this
slice's lifts). Three small leaf helpers in the file each contain a
single internal `unsafe { ... }` block around a PG FFI call and were
declared `unsafe fn` only because they take raw PG pointers. Their
soundness obligation ("pass live PG handles") is the same as the
already-safe `read_main_buffer` and `shm_toc_lookup_required<T>`
established in this rotation. Lifting them to safe `fn` removes 4
caller-side `unsafe { ... }` wrappers.

## Scope

Three function lifts in `src/am/ec_hnsw/build_parallel.rs`:

1. `parallel_build_shared_workspace_size(heap_relation, snapshot)` →
   safe `fn`. Internal `unsafe { pg_sys::table_parallelscan_estimate(...) }`
   block retained with SAFETY comment. 1 caller (in
   `try_parallel_build`) drops its `unsafe { ... }` wrap.
2. `parallel_table_scan_from_shared(shared)` → safe `fn`. Internal
   `unsafe { ... }` block around the pointer-arithmetic cast
   retained. 0 unsafe wraps removed (callers were inside larger
   unsafe blocks that this slice doesn't touch), but the
   `unsafe fn` declaration goes away — no behavior change.
3. `send_worker_message(queue_handle, message)` → safe `fn`. Internal
   `unsafe { pg_sys::shm_mq_send(...) }` block retained. 2 caller
   wraps (in `send_build_tuple_message` and `send_done_message`) go
   away.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 117 | 114 | -3 |
| **HNSW subsystem subtotal** | **485** | **482** | **-3** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 415 | 485 |
| After 416 | 482 |

Net rotation delta: **-67 in HNSW** (-12.2%).

## Soundness rationale

Each lifted function follows the same pattern established by
`read_main_buffer` (slice precedent) and `shm_toc_lookup_required<T>`
(packet 402): the function takes raw PG pointers, internally wraps a
single FFI call in `unsafe { ... }` with a SAFETY comment that names
the caller-supplied precondition, and is otherwise pure safe Rust.

- `parallel_build_shared_workspace_size`: precondition is "live heap
  relation + valid snapshot pointer"; FFI is
  `table_parallelscan_estimate`.
- `parallel_table_scan_from_shared`: precondition is "shared points
  at the head of the parallel-build shared workspace allocation
  produced by `InitializeParallelDSM`"; the pointer arithmetic stays
  within that allocation.
- `send_worker_message`: precondition is "queue_handle is an
  attached shm_mq handle"; FFI is `shm_mq_send`.

No anti-pattern B: none of the lifted functions return `&'a T` from a
raw pointer.

## Validation

Artifacts under `reviews/task-50/416-hnsw-build-parallel-leaf-helpers-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Build hot path. No semantic change — same FFI calls with same
arguments. Bench deferred per `feedback_coder_push_smoke_checks`.

## Out of scope

- `parallel_build_worker_main`, `parallel_graph_build_worker_main`,
  `try_parallel_build`, `try_parallel_concurrent_dsm_graph_build`,
  `estimate_chunk`, `estimate_keys`: each still `unsafe fn`. The
  worker entrypoints and `try_*` drivers are tied to the AM-callback
  boundary; lifting requires the parallel-leader unsafe block in
  `try_parallel_build` (the `WaitForParallelWorkersToAttach` loop)
  to be split. Queued.
