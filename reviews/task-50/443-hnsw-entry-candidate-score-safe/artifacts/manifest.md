# Packet 443 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/443-hnsw-entry-candidate-score-safe/`
Surface: HNSW insert.rs + vacuum.rs — entry-candidate + score_graph_element lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 442 commit (`80349ed55`)
Slice commit SHA: `7654b0afd`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Three `unsafe fn` → safe `fn` lifts across insert.rs and vacuum.rs.
Five caller-side `unsafe { ... }` wraps stripped. Net -5 in HNSW.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/insert.rs` | -1 |
| `src/am/ec_hnsw/vacuum.rs` | -4 |
| **HNSW subsystem subtotal** | **-5** |

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

aminsert entry-candidate path + amvacuumcleanup repair-search
path; signature-only. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**-215 (-39.2%)** on HNSW: 549 → 334. The -30% Exit Criteria
target has a 9.2-point cushion.
