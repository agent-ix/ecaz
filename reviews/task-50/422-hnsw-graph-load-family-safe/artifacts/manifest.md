# Packet 422 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/422-hnsw-graph-load-family-safe/`
Surface: HNSW graph.rs — load_* family safe-fn cascade
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 421 commit (`8669f0d0a`)
Slice commit SHA: `4ca917fb19ca2036c15b09bac6b53e0ec5187bcc`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Seven `unsafe fn` load-family helpers in graph.rs lifted to safe `fn`
(`load_graph_element`, `load_exact_graph_element`,
`load_grouped_graph_element`, `load_rerank_payload`,
`load_grouped_rerank_payload`, `load_exact_graph_adjacency`,
`load_grouped_graph_adjacency`). Each body had zero internal unsafe
blocks after slice 421. Sixteen caller-side `unsafe { ... }` wraps
stripped across graph.rs, scan.rs, insert.rs, vacuum.rs.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/graph.rs` | -10 |
| `src/am/ec_hnsw/scan.rs` | -2 |
| `src/am/ec_hnsw/insert.rs` | -2 |
| `src/am/ec_hnsw/vacuum.rs` | -2 |
| **HNSW subsystem subtotal** | **-16** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/` | exact diff (275 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation; 0 `unused_unsafe` warnings |

## Validation rule mapping

- `cargo fmt --all` — not run; mostly indentation shifts from
  wrapper removal.
- `cargo check --no-default-features --features pg18` — captured.
  Clean, **0 unused_unsafe warnings**.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same tuple reads, decoders, error paths.

## Performance gate

Scan/insert/vacuum hot path. Bench deferred per
`feedback_coder_push_smoke_checks`.
