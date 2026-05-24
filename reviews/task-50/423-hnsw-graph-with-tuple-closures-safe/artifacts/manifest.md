# Packet 423 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/423-hnsw-graph-with-tuple-closures-safe/`
Surface: HNSW graph.rs — with_*_graph_tuple closure-entry safe-fn lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 422 commit (`4ca917fb1`)
Slice commit SHA: `6d4a1a8c39a8851ce64adc2ff21a854095bbfca3`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Six closure-entry `unsafe fn` in graph.rs lifted to safe `fn`. Three
caller-side `unsafe { ... }` wraps stripped (one in graph.rs's
codebook chain loader, two in scan.rs's `cached_graph_element`
loaders).

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/graph.rs` | -4 |
| `src/am/ec_hnsw/scan.rs` | -2 |
| **HNSW subsystem subtotal** | **-6** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation; 0 `unused_unsafe` |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same closure invocations.

## Performance gate

Scan/insert hot path. Bench deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**Past the -22% threshold** on HNSW: 549 → 426, net -123 (-22.4%).
