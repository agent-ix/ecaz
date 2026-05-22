# Packet 414 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/414-hnsw-scan-upper-layer-seed-candidate-safe/`
Surface: HNSW scan.rs — `cached_upper_layer_seed_candidate` safe-fn lift
Branch: `task-50-hnsw` (renamed from `task-50-unsafe-closeout` this packet)
Pre-slice HEAD: packet 413 commit (`1d5f3c79e`)
Slice commit SHA: `7090e638e3d2d9333ae8bf8ffd0c42aa587b45fc`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

`cached_upper_layer_seed_candidate` lifted from `unsafe fn` to safe
`fn`. The body's `graph::greedy_descend_with_successors` closure keeps
its `unsafe { cached_scan_successor_candidates_for_layer(...) }`
block (callee not lifted yet); the closure reborrows the parent's
`&mut TqScanOpaque` as `*mut` at each call.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -1 |
| **HNSW subsystem subtotal** | **-1** |

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
- Runtime tests — not run. Same semantics: same greedy-descent
  driver, same successor-loading closure, same FnMut borrow pattern.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.

## Branch rename note

Per user direction 2026-05-22: branch renamed from
`task-50-unsafe-closeout` to `task-50-hnsw` to reflect the
HNSW-only rotation scope. Old remote ref deleted as part of the
push cycle; all 15 prior packet commits remain in history under the
new branch name.
