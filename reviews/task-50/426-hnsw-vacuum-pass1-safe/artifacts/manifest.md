# Packet 426 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/426-hnsw-vacuum-pass1-safe/`
Surface: HNSW vacuum.rs — `plan_page_pass1` + `heap_tid_is_dead` safe-fn lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 425 commit (`9e096b25b`)
Slice commit SHA: `2b8e5321144ee715a9c657149e80c962db892e6a`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

`vacuum::heap_tid_is_dead` lifted from `unsafe fn` to safe `fn` with
one internal unsafe block retained around the PG callback FFI.
`vacuum::plan_page_pass1` lifted with no remaining internal unsafe
blocks (uses safe helpers throughout). Two caller wraps stripped in
`repair_metadata_entry_point_after_vacuum` and `rewrite_page_pass1`.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/vacuum.rs` | -2 |
| **HNSW subsystem subtotal** | **-2** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/` | exact diff (86 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same callback wiring; same tuple plan.

## Performance gate

Vacuum hot path (pass-1 plans every page during ambulkdelete). Bench
deferred per `feedback_coder_push_smoke_checks`.
