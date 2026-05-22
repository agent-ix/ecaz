# Task 50/413: HNSW scan.rs — `prefetch_graph_buffers` safe-fn lift

## Why this slice

`prefetch_graph_buffers` already takes `opaque: &mut TqScanOpaque` (it
never took the raw-pointer shape) — it was `unsafe fn` only because
its body, before the rotation, contained operations that have since
been moved to safe abstractions. After slices 406-412, the body has
zero internal `unsafe { ... }` blocks: every call goes through
`reset_graph_prefetch_blocks`, `ensure_graph_read_stream`, and the
already-safe `super::stream::{reset_scan_owned_read_stream,
visit_scan_owned_read_stream_pinned}` helpers.

Lifting to safe `fn` removes the one remaining caller-side
`unsafe { ... }` wrapper inside `cached_scan_successor_candidates_for_layer`.

## Scope

- `prefetch_graph_buffers` lifted from `unsafe fn` to `fn`. No body
  change required.
- Single caller in `cached_scan_successor_candidates_for_layer` drops
  the `unsafe { ... }` wrap and (since the parent is still
  `unsafe fn`) acquires the `&mut TqScanOpaque` argument via the
  existing `scan_opaque_mut(opaque)` helper.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 99 | 98 | -1 |
| **HNSW subsystem subtotal** | **488** | **487** | **-1** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 412 | 488 |
| After 413 | 487 |

Net rotation delta: **-62 in HNSW** (-11.3%).

## Soundness rationale

`prefetch_graph_buffers` body uses only safe Rust abstractions:
`HashMap`/`HashSet` mutation, `reset_graph_prefetch_blocks`,
`ensure_graph_read_stream`, and the `super::stream::*` read-stream
helpers. The lift is pure signature flip.

No anti-pattern B: signature takes `&mut TqScanOpaque`.

## Validation

Artifacts under
`reviews/task-50/413-hnsw-scan-prefetch-graph-buffers-safe/artifacts/`.

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.

## Out of scope

The remaining `unsafe fn`s in scan.rs that thread `*mut TqScanOpaque`:

- `cached_scan_successor_candidates_for_layer<KeepFn>` — has a
  long-lived `quantizer` borrow (`cached_quantizer_ref` returns an
  `&'a ProdQuantizer`) that overlaps the function's many
  `scan_opaque_mut(opaque)` calls. Lifting to `&mut TqScanOpaque`
  parameter requires a non-trivial borrow restructure (read the
  Copy config up front, scope the quantizer borrow narrowly per
  scoring branch, or split the function). Queued.
- `cached_upper_layer_seed_candidate` — wraps the unsafe
  `cached_scan_successor_candidates_for_layer` inside a
  `graph::greedy_descend_with_successors` closure. Coupled lift; the
  closure can stay or be lifted together. Queued.
