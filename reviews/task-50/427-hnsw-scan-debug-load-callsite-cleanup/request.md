# Task 50/427: HNSW scan_debug.rs — strip redundant unsafe wraps on now-safe loaders

## Why this slice

Three caller-side `unsafe { ... }` wraps in `scan_debug.rs` were
left behind by slices 420/422/424 when the underlying `graph::*` and
`shared::*` helpers became safe `fn`s but the test-only debug
callers retained their wrappers. The compiler doesn't emit
`unused_unsafe` from `#[cfg(any(test, feature = "pg_test"))]`
modules in normal builds (the lints only fire when those modules
are included), and the wraps were missed in the previous packets'
cleanup sweep.

This slice strips them directly.

## Scope

- `debug_with_page_line_tuple_bytes` (around `with_page_line_tuple_bytes`)
- `debug_load_graph_element` (around `graph::load_exact_graph_element`)
- `debug_load_graph_adjacency` (around `graph::load_exact_graph_adjacency`)

All three callees are safe `fn` after slices 420/422/424. The
SAFETY comments above each `unsafe { ... }` block (which described
the lifted contract) are removed alongside the wrappers.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan_debug.rs` | 21 | 18 | -3 |
| **HNSW subsystem subtotal** | **396** | **393** | **-3** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 426 | 396 |
| After 427 | 393 |

Net rotation delta: **-156 in HNSW** (-28.4%).

## Soundness rationale

No-op: the wraps were stripped because their wrapped fn became safe
in an earlier slice. No change in invariants.

## Validation

Artifacts under `reviews/task-50/427-hnsw-scan-debug-load-callsite-cleanup/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Debug helpers; not on any production hot path. Bench deferred per
`feedback_coder_push_smoke_checks`.
