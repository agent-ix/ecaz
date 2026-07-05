# Packet 427 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/427-hnsw-scan-debug-load-callsite-cleanup/`
Surface: HNSW scan_debug.rs — strip redundant unsafe wraps
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 426 commit (`2b8e53211`)
Slice commit SHA: `6bc4cae9376ce346ce5e7196cdce9dd44bc4d1db`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Three caller-side `unsafe { ... }` wraps in `scan_debug.rs` stripped:
the `with_page_line_tuple_bytes`, `graph::load_exact_graph_element`,
and `graph::load_exact_graph_adjacency` callers. Each callee was
lifted to safe `fn` in an earlier rotation slice.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan_debug.rs` | -3 |
| **HNSW subsystem subtotal** | **-3** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/scan_debug.rs` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Pure wrapper removal; no semantic change.

## Performance gate

Debug helpers, no production hot path. Bench deferred per
`feedback_coder_push_smoke_checks`.
