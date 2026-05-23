# Task 50/418: HNSW vacuum.rs — `VacuumPageRewrite::start` safe-fn lift

## Why this slice

First vacuum.rs-side lift in the rotation. `VacuumPageRewrite::start`
is the page-rewrite RAII guard's constructor; it was `unsafe fn`
only to push the live-relation + locked-buffer precondition outward.
The body had one internal `unsafe { ... }` block around
`wal::GenericXLogTxn::start(relation)` and
`wal_txn.register_locked_buffer_full_image(&buffer)` — identical
shape to `InsertPageWrite::from_locked_buffer` in slice 417.

## Scope

- `VacuumPageRewrite::start(relation, &LockedBufferGuard)` lifted
  from `unsafe fn` to safe `fn`. Internal
  `unsafe { wal::GenericXLogTxn::start(...) ... }` block retained
  with SAFETY comment.
- Single caller `HnswVacuumIndexRelation::begin_page_rewrite` drops
  its `unsafe { ... }` wrap.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/vacuum.rs` | 55 | 54 | -1 |
| **HNSW subsystem subtotal** | **474** | **473** | **-1** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 417 | 474 |
| After 418 | 473 |

Net rotation delta: **-76 in HNSW** (-13.8%).

## Soundness rationale

Same shape as slice 417's `InsertPageWrite::from_locked_buffer`:
caller-supplied raw PG handles, single internal unsafe block, SAFETY
comment naming the precondition. No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/418-hnsw-vacuum-page-rewrite-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Vacuum hot path. No semantic change. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Out of scope

Remaining 20+ `unsafe fn`s in vacuum.rs (`run_bulkdelete_with_adapter`,
`repair_metadata_entry_point_after_vacuum`, `rewrite_page_pass1`,
`plan_page_pass1`, `apply_page_pass1_updates`,
`repair_graph_connections_with_storage`, `collect_repair_requests*`,
`unlink_deleted_graph_connections`, `plan_repair_*`,
`search_repair_candidates_for_layer`, `load_vacuum_entry_candidate`,
`top_up_repair_replacements_from_linear_scan`,
`collect_linear_repair_candidates_on_page`,
`load_grouped_rerank_payload_for_linear_repair_candidate`,
`apply_repair_plans*`, `rewrite_page_pass2`, `plan_page_pass2`): each
holds internal unsafe blocks around `graph::*` / `page::*` /
`shared::*` FFI surfaces that themselves need lifts before the
wrapping `unsafe fn`s can flip. Queued.
