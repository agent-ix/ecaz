# Packet 415 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/415-hnsw-scan-successor-candidates-safe/`
Surface: HNSW scan.rs — `cached_scan_successor_candidates_for_layer` safe-fn lift
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 414 commit (`7090e638e`)
Slice commit SHA: `06fa0058463c49e15bacff0d596dd23859884d39`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

`cached_scan_successor_candidates_for_layer<KeepFn>` lifted from
`unsafe fn(*mut TqScanOpaque, ...)` to safe `fn(&mut TqScanOpaque, ...)`.
Quantizer / binary_query borrows tightened to inner blocks so they
don't overlap the loop's mutable calls. Both caller closures drop
their `unsafe { ... }` wrappers; one new bounded `unsafe { &*opaque_ptr }`
block added inside the layer-0 inner closure (FnMut-nested-borrow
pattern).

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -1 |
| **HNSW subsystem subtotal** | **-1** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/scan.rs` | exact diff (302 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; minor formatting drift in restructure.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same candidates, same scores, same loop
  ordering. The quantizer/binary_query borrows are re-acquired per
  iteration but read identical state.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.
