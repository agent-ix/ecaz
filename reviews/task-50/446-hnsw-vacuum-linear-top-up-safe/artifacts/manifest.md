# Packet 446 — Artifact Manifest — **-40% MILESTONE**

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/446-hnsw-vacuum-linear-top-up-safe/`
Surface: HNSW vacuum.rs — top_up_repair_replacements_from_linear_scan lift
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 445 commit (`60438372b`)
Slice commit SHA: `67055d7d1`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

One `unsafe fn` → safe `fn` lift. One caller-side
`unsafe { ... }` wrap stripped. Net -1 in HNSW.

**Crosses the -40% milestone**: HNSW 549 → 329 (-220, -40.07%).

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/vacuum.rs` | -1 |
| **HNSW subsystem subtotal** | **-1** |

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

amvacuumcleanup linear-top-up; signature-only. Bench deferred
per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**-220 (-40.07%)** on HNSW: 549 → 329. **The -40% threshold has
been crossed.** The -30% Exit Criteria target has a 10.07-point
cushion.
