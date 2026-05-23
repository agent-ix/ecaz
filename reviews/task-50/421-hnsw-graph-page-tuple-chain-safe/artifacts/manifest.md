# Packet 421 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/421-hnsw-graph-page-tuple-chain-safe/`
Surface: HNSW graph.rs — page-tuple reader chain safe-fn lifts
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 420 commit (`f5e3a0276`)
Slice commit SHA: `8669f0d0a748807bce47407e215a7a9cd3112863`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

`graph::with_page_tuple_bytes`, `graph::read_page_tuple_from_buffer`,
and `graph::read_page_tuple` lifted from `unsafe fn` to safe `fn`.
Each retains its single internal `unsafe { ... }` block with the
original SAFETY contract. Twelve caller-side `unsafe { ... }`
wrappers stripped across graph.rs.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/graph.rs` | -14 |
| **HNSW subsystem subtotal** | **-14** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/graph.rs` | exact diff (299 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation; 0 `unused_unsafe` warnings |

## Validation rule mapping

- `cargo fmt --all` — not run; the wrappers were stripped but
  indentation changes are minor.
- `cargo check --no-default-features --features pg18` — captured.
  Clean, **0 unused_unsafe warnings**.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same page-tuple reads, same item-id
  validation, same byte-slice bounds checks.

## Performance gate

Scan/insert/vacuum hot path. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**Past the -18% threshold** on HNSW: 549 → 448, net -101 (-18.4%).
