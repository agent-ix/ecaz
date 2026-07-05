# Packet 445 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/445-hnsw-vacuum-apply-repair-safe/`
Surface: HNSW vacuum.rs — apply_repair_plans chain lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 444 commit (`4eeb5a750`)
Slice commit SHA: `60438372b`
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

amvacuumcleanup repair-application; signature-only. Bench
deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**-219 (-39.9%)** on HNSW: 549 → 330. The -30% Exit Criteria
target has a 9.9-point cushion. One block away from -40%.
