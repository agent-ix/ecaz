# Packet 408 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/408-hnsw-scan-cached-graph-element-safe-fn/`
Surface: HNSW scan.rs — cached_graph_element lift
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 407 commit (`38a667937`)
Slice commit SHA: `af03335bb83f2578278e243357d551d241db9ea6`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

`cached_graph_element` lifted from `unsafe fn(*mut TqScanOpaque)` to
safe `fn(&mut TqScanOpaque)`. Six caller-side `unsafe { ... }`
wrappers removed; one call site restructured to keep its raw-pointer
binding `opaque_ptr` after the lifted call so borrows don't overlap.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -6 |
| **HNSW subsystem subtotal** | **-6** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/scan.rs` | exact diff (90 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same semantics: same cache lookup, same
  Arc allocation, same record_* call ordering.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.
