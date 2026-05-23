# Packet 433 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/433-hnsw-source-indexed-resolvers-safe/`
Surface: HNSW source.rs — indexed-vector + attnum resolver cascade lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 432 commit (`9b26f99fb`)
Slice commit SHA: `d7cee2766b8ad55587bc7c321b2c6c4869a81988`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Six `unsafe fn` → safe `fn` lifts in source.rs's indexed-vector
catalog-lookup chain. Nine caller-side `unsafe { ... }` wraps
stripped across HNSW; one cross-AM cleanup in ec_ivf/scan.rs (with a
let-binding scope wrapped in `{ ... }` to preserve multi-let body
shape).

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/source.rs` | -5 |
| `src/am/ec_hnsw/scan.rs` | -1 |
| `src/am/ec_hnsw/vacuum.rs` | -1 |
| `src/am/ec_hnsw/build.rs` | -2 |
| `src/am/ec_ivf/scan.rs` | -1 (cross-AM cleanup) |
| **HNSW subsystem subtotal** | **-9** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/` | exact diff (193 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | clean |

## Validation rule mapping

- `cargo fmt --all` — not run.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run.

## Performance gate

Setup path; not inner-loop. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**-188 (-34.2%)** on HNSW: 549 → 361. The -30% Exit Criteria
target has a 4.2-point cushion.
