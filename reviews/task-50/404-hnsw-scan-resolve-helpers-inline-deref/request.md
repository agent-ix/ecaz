# Task 50/404: HNSW scan resolve-helpers — inline single deref

## Why this slice

`scan.rs` is the densest HNSW file (139 blocks pre-slice). The two
`unsafe fn resolve_scan_*` helpers each deref the same
`pg_sys::IndexScanDesc` two or three times, once per inspected field. Each
deref is its own `unsafe { (*scan).field }` block. Collapsing to a single
`unsafe { &*scan }` at function entry and then reading fields off the
typed `&pg_sys::IndexScanDescData` borrow removes 1-2 blocks per
function without introducing anti-pattern B (the borrow is frame-bounded
by the local `let scan_ref = ...` binding inside the unsafe fn body).

Same shape as packet 403's fix: bounded borrow, explicit type, use-site
SAFETY comment.

## Scope

- `resolve_scan_heap_relation` in `src/am/ec_hnsw/scan.rs`: collapse three
  `unsafe { (*scan).field }` blocks (heapRelation null-check,
  heapRelation read, indexRelation read) into one `unsafe { &*scan }`
  at function entry.
- `resolve_scan_snapshot` in the same file: collapse two
  `unsafe { (*scan).xs_snapshot }` blocks into one `unsafe { &*scan }`.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 139 | 136 | -3 |
| **HNSW subsystem subtotal** | **529** | **526** | **-3** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 399 | 541 |
| After 400 | 540 |
| After 401 | 535 |
| After 402 | 528 |
| After 403 (anti-pattern B fix) | 529 |
| After 404 | 526 |

Net rotation delta: **-23 in HNSW**.

## Validation

Artifacts under
`reviews/task-50/404-hnsw-scan-resolve-helpers-inline-deref/artifacts/`:

- `manifest.md` — head SHA, files touched, validation mapping.
- `per-file-after.log` — post-change HNSW per-file block counts.
- `diff.patch` — exact diff applied.
- `cargo-check-pg18.log` — `cargo check --no-default-features --features
  pg18` (lib smoke). Clean.

## Performance gate

Not on a scoring or traversal hot path. `resolve_scan_heap_relation` and
`resolve_scan_snapshot` run once per scan-setup pre-roll (the grouped
heap rerank state configuration); they do not influence inner-loop
performance. No bench evidence required per Task 50 §Performance Gate.

## Out of scope

- The `explain_counters_from_index_scan_state` chain that walks
  `index_state -> scan_desc -> opaque`: each pointer must be checked
  non-null before deref, so collapsing to a single `&*p` borrow doesn't
  remove blocks. Kept as-is.
- Further structural lifts on scan.rs (`unsafe fn` -> safe + reference
  threading on the score_grouped_* family, the cached_graph_element
  chain): queued as future slices.
