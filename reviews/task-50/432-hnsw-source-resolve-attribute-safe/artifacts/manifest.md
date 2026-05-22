# Packet 432 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/432-hnsw-source-resolve-attribute-safe/`
Surface: HNSW source.rs — `resolve_source_*` chain safe-fn lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 431 commit (`fdca416a1`)
Slice commit SHA: `9b26f99fb9655acbad50055796bc17cc7e588d9a`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Four `unsafe fn` → safe `fn` lifts in source.rs's catalog-lookup
chain. Seven caller-side `unsafe { ... }` wraps stripped across
source.rs (3), insert.rs (1), scan.rs (1), build.rs (2).

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/source.rs` | -3 |
| `src/am/ec_hnsw/scan.rs` | -1 |
| `src/am/ec_hnsw/insert.rs` | -1 |
| `src/am/ec_hnsw/build.rs` | -2 |
| **HNSW subsystem subtotal** | **-7** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | clean |

## Validation rule mapping

- `cargo fmt --all` — not run.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run.

## Performance gate

Aminsert / ambuild / amscan setup; not inner-loop hot. Bench deferred
per `feedback_coder_push_smoke_checks`.
