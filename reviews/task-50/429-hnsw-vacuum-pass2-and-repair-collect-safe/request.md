# Task 50/429: HNSW vacuum.rs — pass-2 + repair-collect cascade safe-fn lifts (**crosses -30%**)

## Why this slice

The cascading lifts continue from slice 428. After `plan_page_pass1`,
`rewrite_page_pass1`, and `apply_page_pass1_updates` became safe `fn`,
the matching pass-2 and repair-collect cascade in `vacuum.rs` becomes
liftable. Each function now composes only safe operations.

This slice crosses the Task 50 §Exit Criteria **-30% per-module
target** on HNSW.

## Scope

Seven `unsafe fn` → safe `fn` flips in `src/am/ec_hnsw/vacuum.rs`:

1. `collect_repair_requests_on_page` (page-tuple visitor +
   `graph::load_graph_neighbors`)
2. `collect_repair_requests` (driver over share-locked pages)
3. `plan_page_pass2` (neighbor-tuple decoder via the lifted writable
   visitor)
4. `apply_page_pass2_updates` (writable-tuple visitor)
5. `rewrite_page_pass2` (composes plan + apply)
6. `unlink_deleted_graph_connections` (drives pass-2 plan + rewrite)
7. `load_grouped_rerank_payload_for_linear_repair_candidate` (uses
   `graph::load_grouped_rerank_payload` + same-page tuple visitor —
   both safe)
8. `collect_linear_repair_candidates_on_page` (retains one inner
   `unsafe { metric.score_graph_element(...) }` block for the
   still-`unsafe fn` scoring metric).

Caller-side `unsafe { ... }` wraps stripped:

- `collect_repair_requests` driver in
  `repair_graph_connections_with_storage` (line ~860)
- `plan_page_pass2` + `rewrite_page_pass2` callers in
  `unlink_deleted_graph_connections` (lines ~1021, ~1033)
- `apply_page_pass2_updates` caller in `rewrite_page_pass2`
  (line ~1725)
- `plan_page_pass2` caller in `rewrite_page_pass2` (line ~1716)
- `collect_linear_repair_candidates_on_page` caller in
  `top_up_repair_replacements_from_linear_scan` (line ~1344)

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/vacuum.rs` | 34 | 28 | -6 |
| **HNSW subsystem subtotal** | **390** | **384** | **-6** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 428 | 390 |
| After 429 | 384 |

Net rotation delta: **-165 in HNSW** (**-30.05%**).

## Soundness rationale

Each lifted function had zero or one bounded internal `unsafe { ... }`
block, and the bodies compose only safe helpers after slices 424-428.
No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/429-hnsw-vacuum-pass2-and-repair-collect-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch` (177 lines)
- `cargo-check-pg18.log` — clean, **0 unused_unsafe warnings**.

## Performance gate

Vacuum hot path (pass-2 and repair-collect each scan every page).
Bench deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone — **Task 50 §Exit Criteria target met for HNSW**

Net **-165 (-30.05%)**. The Task 50 §Exit Criteria target is "each
processed module's block count has dropped by at least 30% from its
post-Task-35 state" — this rotation has now crossed that bar on the
densest HNSW file family (graph.rs / shared.rs / scan.rs /
vacuum.rs / insert.rs cluster, considered as the HNSW subsystem).

Remaining HNSW unsafe surface (384 blocks):

- `build_parallel.rs`: 114 (DSM atomics, SpinLock/CV, shm_toc setup)
- `scan.rs`: 87 (scan opaque raw pointer threading + remaining
  unsafe-fn callees inside outer scan callbacks)
- `insert.rs`: 45 (page-mutation chain + plan_backlink_mutations
  family)
- `source.rs`: 37 (FromDatum/detoast/source-vector decoding)
- `vacuum.rs`: 28 (top-level `run_bulkdelete_with_adapter`,
  `repair_metadata_entry_point_after_vacuum`, `repair_graph_connections_with_storage`,
  `plan_repair_replacements`, `plan_repair_replacement`,
  `search_repair_candidates_for_layer`, `load_vacuum_entry_candidate`,
  `top_up_repair_replacements_from_linear_scan`, `apply_repair_plans`,
  `apply_repair_plans_on_page`, plus the linear-repair candidate
  loader cluster — many composing still-unsafe FFI primitives)
- `build.rs`: 22 (initial build state setup + bootstrap)
- `shared.rs`: 21 (`initialize_metadata_page`, `update_metadata_page`,
  `with_locked_metadata_page`, `read_data_page`, etc.)
- `scan_debug.rs`: 18 (test-only debug helpers)
- `graph.rs`: 9 (six remaining `unsafe fn` traversal drivers that
  take FnMut score/keep closures)
- `index_info.rs`: 3 (BuildIndexInfo guard + view, slice 400)

The remaining surface is mostly composed of:
1. Worker/leader entrypoints and `extern "C"` callback shells (must
   remain `unsafe fn` per FFI ABI).
2. PG resource RAII / DSM atomic primitives (irreducible boundary).
3. Closure-bound traversal drivers (closure callees still unsafe).
4. Page-mutation cluster (PageInit / PageAddItem / GenericXLog
   transactional rewrite).

Each is its own slice family; the systematic safe-fn lift pattern
demonstrated across slices 399-429 can be applied to whichever
families the next coder rotation prioritizes.
