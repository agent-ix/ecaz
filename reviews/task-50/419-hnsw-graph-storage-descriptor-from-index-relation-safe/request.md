# Task 50/419: HNSW graph.rs — `GraphStorageDescriptor::from_index_relation` safe-fn lift

## Why this slice

`graph::GraphStorageDescriptor::from_index_relation(index_relation,
metadata)` is the canonical entry point for resolving a `GraphStorageDescriptor`
from an HNSW index relation. It is called from five HNSW production
modules (vacuum.rs, insert.rs, scan.rs, shared.rs, scan_debug.rs).

The function body has **zero internal unsafe blocks**: it calls
`Self::from_metadata(metadata)?` (safe) and
`options::relation_options(NonNull::new(index_relation).expect(...))`
(safe — `relation_options` takes `NonNull<RelationData>`). The
`unsafe fn` declaration was legacy; the function's actual soundness
obligation ("caller provides a live index relation pointer") is
already enforced by the `NonNull::new(...).expect(...)` null check
inside `relation_options`.

Lifting to safe `fn` removes 5 caller-side `unsafe { ... }`
wrappers across HNSW with no body change.

## Scope

- `graph::GraphStorageDescriptor::from_index_relation` lifted from
  `pub(crate) unsafe fn` to `pub(crate) fn`. Body unchanged.
- 5 caller wraps removed:
  - `vacuum.rs:473` (in `repair_graph_connections`)
  - `insert.rs:614` (in `run_insert_with_adapter`)
  - `scan.rs:1126` (in scan setup)
  - `shared.rs:163` (in `count_element_tuples`)
  - `scan_debug.rs:114` (in debug helper)

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 96 | 95 | -1 |
| `src/am/ec_hnsw/insert.rs` | 56 | 55 | -1 |
| `src/am/ec_hnsw/vacuum.rs` | 54 | 53 | -1 |
| `src/am/ec_hnsw/shared.rs` | 30 | 29 | -1 |
| `src/am/ec_hnsw/scan_debug.rs` | 22 | 21 | -1 |
| **HNSW subsystem subtotal** | **473** | **468** | **-5** |

(`graph.rs` is unchanged in block count — the lifted function had
zero internal unsafe blocks. The keyword removal does not subtract
any blocks from graph.rs itself.)

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 418 | 473 |
| After 419 | 468 |

Net rotation delta: **-81 in HNSW** (-14.8%).

## Soundness rationale

The function's only soundness-sensitive operation is the
`NonNull::new(index_relation).expect(...)` null check inside the
`options::relation_options(NonNull<RelationData>)` call. That call
chain is entirely safe Rust: the caller-supplied raw pointer is
validated up front (panic on null) before being dereferenced through
the typed `NonNull` wrapper.

No anti-pattern B: the function returns `Result<Self, String>`,
not `&'a T`.

## Validation

Artifacts under `reviews/task-50/419-hnsw-graph-storage-descriptor-from-index-relation-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch` (83 lines)
- `cargo-check-pg18.log` — clean.

## Performance gate

Cold path (graph-storage discriminator resolution at scan/vacuum
setup). Bench deferred per `feedback_coder_push_smoke_checks`.

## Out of scope

- The test caller in `src/tests/ec_hnsw_recall_debug_exports.rs`
  still uses the lifted function but inside its own `unsafe fn` test
  shell; the unsafe block at the call site there does not affect
  HNSW production block counts.
