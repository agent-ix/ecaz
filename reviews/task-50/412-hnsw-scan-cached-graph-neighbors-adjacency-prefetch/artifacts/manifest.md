# Packet 412 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/412-hnsw-scan-cached-graph-neighbors-adjacency-prefetch/`
Surface: HNSW scan.rs — cached_graph neighbors/adjacency/with_prefetch lifts
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 411 commit (`3a1891b2a`)
Slice commit SHA: `413aae2719f67ba5089b8b35bbb7fca396a97e68`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Three more `unsafe fn(*mut TqScanOpaque, ...)` lifts to safe
`fn(&mut TqScanOpaque, ...)`:

- `cached_graph_neighbors`  (1 internal unsafe block kept)
- `cached_graph_adjacency`  (composes element + neighbors)
- `cached_graph_element_with_prefetch`  (composes from_buffer + element)

Five caller-side `unsafe { ... }` wrappers removed across scan.rs.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -4 |
| **HNSW subsystem subtotal** | **-4** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/scan.rs` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same semantics: same cache lookup, same
  page-load FFI, same Arc reference counting.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.
