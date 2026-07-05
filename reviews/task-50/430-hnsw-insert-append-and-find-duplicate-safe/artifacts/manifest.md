# Packet 430 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/430-hnsw-insert-append-and-find-duplicate-safe/`
Surface: HNSW insert.rs — append/find_duplicate cascade lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 429 commit (`f2d59823e`)
Slice commit SHA: `55966fd6e8f53bcc34356a2ff8b74371b56a8444`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Eleven `unsafe fn` → safe `fn` lifts in insert.rs. Six caller-side
`unsafe { ... }` wraps stripped. One incidental syntax error from a
prior mechanical cleanup also repaired during this slice.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/insert.rs` | -6 |
| **HNSW subsystem subtotal** | **-6** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/insert.rs` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation; 0 unused_unsafe |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured.
  Clean, 0 unused_unsafe.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Function bodies unchanged.

## Performance gate

Insert hot path. Bench deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone

Net **-171 (-31.1%)** on HNSW: 549 → 378.
