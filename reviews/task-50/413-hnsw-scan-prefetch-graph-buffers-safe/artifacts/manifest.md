# Packet 413 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/413-hnsw-scan-prefetch-graph-buffers-safe/`
Surface: HNSW scan.rs — `prefetch_graph_buffers` safe-fn lift
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 412 commit (`413aae271`)
Slice commit SHA: `1d5f3c79e845b2675ac11724fa782b5438c3c1f3`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

`prefetch_graph_buffers` lifted from `unsafe fn(&mut TqScanOpaque, ...)`
to safe `fn(&mut TqScanOpaque, ...)`. Body had no internal unsafe
blocks remaining after the prior rotation; this slice is the pure
signature flip + caller-side `unsafe { ... }` removal.

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
- Runtime tests — not run. Same semantics: same block list, same
  read-stream invocation.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.
