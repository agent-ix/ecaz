# Packet 444 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/444-hnsw-vacuum-finalize-dead-safe/`
Surface: HNSW vacuum.rs — finalize_fully_dead chain lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 443 commit (`c5b93a417`)
Slice commit SHA: `b8dab0bff`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Two `unsafe fn` → safe `fn` lifts. Three caller-side
`unsafe { ... }` wraps stripped. Net -2 in HNSW.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/vacuum.rs` | -2 |
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

amvacuumcleanup pass-2 finalization; signature-only. Bench
deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**-217 (-39.5%)** on HNSW: 549 → 332. The -30% Exit Criteria
target has a 9.5-point cushion. Approaching -40% threshold.
