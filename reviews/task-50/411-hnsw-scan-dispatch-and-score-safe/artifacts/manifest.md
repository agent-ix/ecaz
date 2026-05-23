# Packet 411 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/411-hnsw-scan-dispatch-and-score-safe/`
Surface: HNSW scan.rs — score dispatch + and_score lifts
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 410 commit (`aba646bb6`)
Slice commit SHA: `3a1891b2a27670e2823772679b10fafecba753a3`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

`score_cached_graph_element_dispatch` and `cached_graph_element_and_score`
lifted from `unsafe fn(*mut TqScanOpaque)` to safe `fn(&mut TqScanOpaque)`.
Three caller-side `unsafe { ... }` wrappers removed.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -3 |
| **HNSW subsystem subtotal** | **-3** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/scan.rs` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same semantics: same dispatch branch on
  same inputs; same element loading; same scoring.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**Past the -10% threshold** on HNSW: 549 → 492, net -57 (-10.4%).
The Task 50 §Exit Criteria target is -30% per processed module; the
current rotation has cleared the first decile and continues toward
that target via more `unsafe fn` → safe `fn` lifts in the
`cached_graph_*` and `prefetch_*` families.
