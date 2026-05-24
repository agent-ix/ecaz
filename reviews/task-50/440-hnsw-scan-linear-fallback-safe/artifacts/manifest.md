# Packet 440 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/440-hnsw-scan-linear-fallback-safe/`
Surface: HNSW scan.rs — linear-fallback selection chain lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 439 commit (`79f400292`)
Slice commit SHA: `d58a7c7c4`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Three `unsafe fn` → safe `fn` lifts in the linear-fallback chain.
Four caller-side `unsafe { ... }` wraps stripped. The
`produce_next_scan_heap_tid` linear-fallback dispatcher arm is now
a bare call; only the graph-traversal arm remains wrapped pending
the prefetch cascade lift.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -4 |
| **HNSW subsystem subtotal** | **-4** |

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

Linear-scan fallback path; signature-only. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**-204 (-37.2%)** on HNSW: 549 → 345. The -30% Exit Criteria
target has a 7.2-point cushion.
