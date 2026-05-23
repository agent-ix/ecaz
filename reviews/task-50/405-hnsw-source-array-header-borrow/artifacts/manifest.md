# Packet 405 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/405-hnsw-source-array-header-borrow/`
Surface: HNSW source.rs — collapse repeated ArrayType field derefs
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 404 commit (`9b543318e`)
Slice commit SHA: `86740c26e5bd04e16a7054cdaa2f6db689ea0d68`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Single inline `unsafe { &*array_ptr }` borrow at function-frame scope,
replacing two `unsafe { (*array_ptr).field }` field-read blocks in the
real-array source validation path. Same shape as packets 403 and 404.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/source.rs` | -1 |
| **HNSW subsystem subtotal** | **-1** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/source.rs` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: see `per-file-after.log`.
- Runtime tests — not run. Field reads preserve byte-for-byte semantics.

## Performance gate

Not on a scoring or traversal hot path; source decode helper runs at
build/insert time. Bench deferred per
`feedback_coder_push_smoke_checks`.
