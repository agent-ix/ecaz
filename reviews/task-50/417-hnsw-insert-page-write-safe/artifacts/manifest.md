# Packet 417 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/417-hnsw-insert-page-write-safe/`
Surface: HNSW insert.rs — `InsertPageWrite` constructor safe-fn lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 416 commit (`566ec2223`)
Slice commit SHA: `edcbd367c61aa5281e1bf8e93f4fadef8be48cb7`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Three `unsafe fn` constructors on `InsertPageWrite` lifted to safe
`fn`. Six caller-side `unsafe { ... }` wrappers removed across the
six `append_*_tuple*` helpers in insert.rs. Internal cross-call
unsafe wraps inside `open_tail` and `open_new` disappear as their
callees become safe.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/insert.rs` | -8 |
| **HNSW subsystem subtotal** | **-8** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/insert.rs` | exact diff (117 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same `LockedBufferGuard::read_main*` calls,
  same WAL transaction begin, same page initialization.

## Performance gate

Insert hot path. Bench deferred per `feedback_coder_push_smoke_checks`.
