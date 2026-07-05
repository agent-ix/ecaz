# Packet 420 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/420-hnsw-graph-loaders-safe/`
Surface: HNSW graph.rs — `load_grouped_codebook_model` + `load_graph_neighbors` safe-fn lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 419 commit (`19189cd54`)
Slice commit SHA: `f5e3a0276cbc37d4ca395582f4ab608f2eaaf885`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Two `graph::load_*` leaf loaders lifted from `unsafe fn` to safe
`fn`. Six caller-side `unsafe { ... }` wraps removed across HNSW
(scan.rs -2, insert.rs -1, vacuum.rs -1, graph.rs -2). Each lifted
function retains exactly one internal unsafe block around its
page-tuple reader callee.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -2 |
| `src/am/ec_hnsw/insert.rs` | -1 |
| `src/am/ec_hnsw/vacuum.rs` | -1 |
| `src/am/ec_hnsw/graph.rs` | -2 |
| **HNSW subsystem subtotal** | **-6** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/` | exact diff (88 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same page-tuple reads, same decoding,
  same error paths.

## Performance gate

Codebook load is on PqFastScan setup (cold); neighbor load runs at
every traversal step but unchanged. Bench deferred per
`feedback_coder_push_smoke_checks`.
