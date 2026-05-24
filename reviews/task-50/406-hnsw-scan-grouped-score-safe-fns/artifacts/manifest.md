# Packet 406 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/406-hnsw-scan-grouped-score-safe-fns/`
Surface: HNSW scan.rs — grouped-score family `unsafe fn` → safe `fn`
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 405 commit (`e710d56fd`)
Slice commit SHA: `cbee2a09d2d3efc8a507268c00b79e6a4df56c10`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Convert six functions in the `score_grouped_*` family from
`unsafe fn(opaque: *mut TqScanOpaque, ...)` to safe `fn(opaque: &T, ...)`,
removing 9 caller-side `unsafe { ... }` wrappers across scan.rs. The
dispatcher `score_grouped_candidate_context` is restructured to compute
both predicate booleans up front so the read borrow ends before any
`scan_opaque_mut` call.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -9 |
| **HNSW subsystem subtotal** | **-9** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/scan.rs` | exact diff (223 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral apart from one
  dispatcher restructure (added two predicate let-bindings).
- `cargo check --no-default-features --features pg18` — captured. Clean,
  no errors, no `unused_unsafe` warnings.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Each conversion preserves call-site arguments
  and field-access semantics; the only structural change is moving a
  `scan_opaque_ref`/`scan_opaque_mut` call from the callee body to the
  caller-side argument expression.

## Performance gate

Scan hot path. Disposition: bench deferred per
`feedback_coder_push_smoke_checks`. No allocation, no scoring math
change, no field semantics change.
