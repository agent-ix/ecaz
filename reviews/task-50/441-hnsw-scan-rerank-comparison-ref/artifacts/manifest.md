# Packet 441 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/441-hnsw-scan-rerank-comparison-ref/`
Surface: HNSW scan.rs — grouped_candidate_rerank_comparison_score
         *mut → &mut signature change
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 440 commit (`f641ad097`)
Slice commit SHA: `42613e4ae`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Convert `grouped_candidate_rerank_comparison_score` from
`unsafe fn(*mut TqScanOpaque)` to safe `fn(&mut TqScanOpaque)`.
All five internal `scan_opaque_ref/mut` calls collapse to direct
borrows. Two caller-side `unsafe { ... }` wraps stripped.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -2 |
| **HNSW subsystem subtotal** | **-2** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git show HEAD` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | clean |

## Validation rule mapping

- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run.

## Performance gate

Inner-loop scoring; signature-only. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**-206 (-37.5%)** on HNSW: 549 → 343. The -30% Exit Criteria
target has a 7.5-point cushion.
