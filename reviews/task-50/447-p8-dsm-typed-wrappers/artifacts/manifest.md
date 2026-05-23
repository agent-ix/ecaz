# Packet 447 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/447-p8-dsm-typed-wrappers/`
Surface: New `src/am/common/dsm.rs` + first HNSW migration
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 446 review-packet commit (`868269cf8`)
Slice commit SHA: `ab09c2a07`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Opens P8. New typed wrapper module with five primitives. First
migration removes two unsafe blocks from HNSW (DSM atomic cell).
Nine new unsafe blocks in `src/am/common/dsm.rs` sit outside the
HNSW subsystem.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/common/mod.rs` | +1 line (mod declaration) |
| `src/am/common/dsm.rs` | +9 (new module; not in HNSW total) |
| `src/am/ec_hnsw/build_parallel.rs` | -2 |
| **HNSW subsystem subtotal** | **-2** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git show HEAD` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | clean |

## Rotation milestone

**-222 (-40.44%)** on HNSW: 549 → 327. P8 is now open.
