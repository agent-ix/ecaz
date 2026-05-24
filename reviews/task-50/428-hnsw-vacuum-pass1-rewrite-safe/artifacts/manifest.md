# Packet 428 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/428-hnsw-vacuum-pass1-rewrite-safe/`
Surface: HNSW vacuum.rs — `apply_page_pass1_updates` + `rewrite_page_pass1` lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 427 commit (`6bc4cae93`)
Slice commit SHA: `b6201ba31755792868a73cdf2cdf97daea80fead`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Two cascading vacuum.rs lifts: `apply_page_pass1_updates` and
`rewrite_page_pass1` go from `unsafe fn` to safe `fn`. Three caller-side
`unsafe { ... }` wraps stripped.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/vacuum.rs` | -3 |
| **HNSW subsystem subtotal** | **-3** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run.

## Performance gate

Vacuum hot path. Bench deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone

Net -159 (-29.0%) in HNSW. Within 6 unsafe blocks of -30%.
