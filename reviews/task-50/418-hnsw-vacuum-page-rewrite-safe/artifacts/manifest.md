# Packet 418 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/418-hnsw-vacuum-page-rewrite-safe/`
Surface: HNSW vacuum.rs — `VacuumPageRewrite::start` safe-fn lift
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 417 commit (`edcbd367c`)
Slice commit SHA: `3e22f63a6d7923a4d0400a96bcc6cfb386f9b4c5`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

`VacuumPageRewrite::start(relation, &LockedBufferGuard) -> Self`
lifted from `unsafe fn` to safe `fn`. Body keeps a single internal
unsafe block around the `wal::GenericXLogTxn::start` + register call
chain. Single caller `HnswVacuumIndexRelation::begin_page_rewrite`
drops its `unsafe { ... }` wrap (the stale SAFETY comment is also
removed).

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/vacuum.rs` | -1 |
| **HNSW subsystem subtotal** | **-1** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/vacuum.rs` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same WAL transaction begin, same
  registered page pointer.

## Performance gate

Vacuum hot path. Bench deferred per `feedback_coder_push_smoke_checks`.
