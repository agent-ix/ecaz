# Task 50/431: HNSW insert.rs — `coalesce_duplicate_*` safe-fn lifts

## Why this slice

The three `coalesce_duplicate_*_heap_tid` helpers in insert.rs each
retain two internal `unsafe { ... }` blocks (LockedBufferGuard +
GenericXLogTxn), but they were `unsafe fn` only to push the live-
relation precondition to callers. Lifting the dispatcher (line ~468
in `InsertFormatAdapter::coalesce_duplicate`) is a one-block
savings.

## Scope

Three `unsafe fn` → safe `fn` flips in `src/am/ec_hnsw/insert.rs`:

1. `coalesce_duplicate_heap_tid`
2. `coalesce_duplicate_turbo_hot_heap_tid`
3. `coalesce_duplicate_grouped_heap_tid`

Caller-side `unsafe { ... }` wrap stripped:
`InsertFormatAdapter::coalesce_duplicate` dispatcher.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/insert.rs` | 39 | 38 | -1 |
| **HNSW subsystem subtotal** | **378** | **377** | **-1** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 430 | 378 |
| After 431 | 377 |

Net rotation delta: **-172 in HNSW** (**-31.3%**).

## Soundness rationale

Each lifted function retains its two internal unsafe blocks with
original SAFETY contracts (LockedBufferGuard pinning + GenericXLog
transaction). Lifting is signature-only.

## Validation

Artifacts under `reviews/task-50/431-hnsw-insert-coalesce-duplicate-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Insert hot path. Bench deferred per `feedback_coder_push_smoke_checks`.
