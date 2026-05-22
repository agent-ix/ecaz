# Packet 409 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/409-hnsw-scan-from-buffer-and-grouped-dispatcher/`
Surface: HNSW scan.rs — buffer-variant cache loader + grouped dispatcher
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 408 commit (`af03335bb`)
Slice commit SHA: `f336719c7d63dbf9c2261e003bb9e22c79b295c0`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

`cached_graph_element_from_buffer` (pg18 feature) and
`score_grouped_candidate_context` (grouped scoring dispatcher) lifted
from `unsafe fn(*mut TqScanOpaque, ...)` to safe `fn(&mut TqScanOpaque, ...)`.
Four caller-side `unsafe { ... }` wrappers removed.

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
- Runtime tests — not run. Same semantics: dispatcher selects same
  branch on same inputs; buffer variant reads same cache.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.
