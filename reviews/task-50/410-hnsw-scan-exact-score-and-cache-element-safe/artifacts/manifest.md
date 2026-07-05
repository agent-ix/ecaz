# Packet 410 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/410-hnsw-scan-exact-score-and-cache-element-safe/`
Surface: HNSW scan.rs — exact_score + score_and_cache lifts
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 409 commit (`f336719c7`)
Slice commit SHA: `aba646bb68878913b09f6a7f275b6409135ee0b9`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Two more lifts in the scan.rs exact-scoring chain:

- `score_and_cache_scan_element` → safe `fn(&mut TqScanOpaque, ...)`
- `exact_score_cached_graph_element` → safe `fn(&mut TqScanOpaque, ...)`

Six caller-side `unsafe { ... }` wrappers removed. The
`refine_grouped_frontier_head_exact` site has its now-redundant
`opaque_ptr` binding dropped from the `CandidateScoreDispatch::Exact`
branch.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -6 |
| **HNSW subsystem subtotal** | **-6** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/scan.rs` | exact diff (184 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same semantics: same scoring branch
  selection on same inputs; same caching; same record_* call ordering.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.
