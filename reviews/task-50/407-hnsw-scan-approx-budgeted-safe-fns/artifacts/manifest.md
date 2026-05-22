# Packet 407 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/407-hnsw-scan-approx-budgeted-safe-fns/`
Surface: HNSW scan.rs — approx + budgeted scoring lift
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 406 commit (`67a2818f6`)
Slice commit SHA: `74c1ae248534c16189f1feaa11b9a1ca05b42f89`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Continuation of slice 406. Two more `unsafe fn`s in the
`score_grouped_*` family converted to safe `fn` taking `&mut
TqScanOpaque`:

- `score_grouped_candidate_context_approx`
- `score_budgeted_grouped_traversal_candidates`

5 caller-side `unsafe { ... }` wrappers removed across scan.rs.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -5 |
| **HNSW subsystem subtotal** | **-5** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/scan.rs` | exact diff (163 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Identical semantics to slice 406's
  conversions.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.
