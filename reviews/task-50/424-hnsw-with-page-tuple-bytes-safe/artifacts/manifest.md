# Packet 424 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/424-hnsw-with-page-tuple-bytes-safe/`
Surface: HNSW shared.rs — `with_page_line_tuple_bytes` + `with_writable_page_tuple_bytes` safe-fn lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 423 commit (`6d4a1a8c3`)
Slice commit SHA: `538852da1c4430d30c0ffcc08d9eb3b6a39410ac`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Two heavily-called `unsafe fn` page-tuple visitors in shared.rs
lifted to safe `fn`. Nineteen caller-side `unsafe { ... }` wraps
stripped across shared.rs, scan.rs, insert.rs, vacuum.rs. Where the
unsafe block held the entire call expression, the closing `}` was
merged with the call's trailing `);` to preserve statement
terminators (avoids E0308 / E0028 in caller blocks).

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/shared.rs` | -2 |
| `src/am/ec_hnsw/scan.rs` | -1 |
| `src/am/ec_hnsw/insert.rs` | -7 |
| `src/am/ec_hnsw/vacuum.rs` | -9 |
| **HNSW subsystem subtotal** | **-19** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/` | exact diff (466 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation; 0 `unused_unsafe` warnings |

## Validation rule mapping

- `cargo fmt --all` — not run; the wrap removal left some indent
  hangs in the closure bodies.
- `cargo check --no-default-features --features pg18` — captured.
  Clean, **0 unused_unsafe warnings**.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same item-id checks, same byte-slice
  bounds, same callback invocations.

## Performance gate

Scan/insert/vacuum hot path. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**Past the -25% threshold** on HNSW: 549 → 407, net -142 (-25.9%).
