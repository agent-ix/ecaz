# Packet 442 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/442-hnsw-insert-backlink-apply-safe/`
Surface: HNSW insert.rs — backlink-mutation apply + plan chain lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 441 commit (`19d20a8b0`)
Slice commit SHA: `cc3e9abba`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Four `unsafe fn` → safe `fn` lifts cascading from slice 437's
plan_backlink_mutation lift. Four caller-side `unsafe { ... }`
wraps stripped. Page-mutation primitives inside
`add_backlinks_on_page` retain their narrow internal unsafe
blocks (buffer guard + WAL transaction + writable page bytes).

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/insert.rs` | -4 |
| **HNSW subsystem subtotal** | **-4** |

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

aminsert backlink-application path; signature-only. Bench
deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**-210 (-38.3%)** on HNSW: 549 → 339. The -30% Exit Criteria
target has an 8.3-point cushion.
