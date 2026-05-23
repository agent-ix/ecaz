# Packet 404 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/404-hnsw-scan-resolve-helpers-inline-deref/`
Surface: HNSW scan.rs — collapse repeated `(*scan)` derefs
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 403 commit (`216ec1720`)
Slice commit SHA: `17b76780a2ebc1aa41bb19f9465ee18d78239969`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Collapse two `unsafe fn resolve_scan_*` helpers' multiple
`unsafe { (*scan).field }` blocks into a single
`unsafe { &*scan }` borrow at function entry. Same frame-bounded
borrow shape used in packet 403 for `shared`.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -3 |
| **HNSW subsystem subtotal** | **-3** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | `for f in src/am/ec_hnsw/*.rs; do c=$(grep -c 'unsafe\\s*{' "$f"); printf "%4d  %s\\n" "$c" "$f"; done \| sort -rn` | request.md unsafe table |
| `diff.patch` | `git diff src/am/ec_hnsw/scan.rs` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping (Task 50 §Validation)

- `cargo fmt --all` — not run; formatting-neutral edits.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: see `per-file-after.log`.
- Runtime tests — not run. Edits preserve every field-access semantic:
  every `scan_ref.field` reads the same byte the previous `(*scan).field`
  read did, in the same order, with the same null-checks.

## Performance gate

Not on a scoring or traversal hot path. Scan-setup-only.
