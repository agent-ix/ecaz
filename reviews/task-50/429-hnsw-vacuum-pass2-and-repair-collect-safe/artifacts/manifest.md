# Packet 429 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/429-hnsw-vacuum-pass2-and-repair-collect-safe/`
Surface: HNSW vacuum.rs — pass-2 + repair-collect cascade lifts (**crosses -30%**)
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 428 commit (`b6201ba31`)
Slice commit SHA: `f2d59823ea40b2b49947bcfe3e9ffcce915d3f7b`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Eight cascading `unsafe fn` → safe `fn` lifts in vacuum.rs covering
the repair-request collector, pass-2 plan/apply/rewrite chain, the
deleted-graph-connection unlinker, the same-page rerank payload
loader, and the linear-repair candidate collector. Six caller-side
`unsafe { ... }` wraps stripped.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/vacuum.rs` | -6 |
| **HNSW subsystem subtotal** | **-6** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/vacuum.rs` | exact diff (177 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation; 0 unused_unsafe |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured.
  Clean, 0 unused_unsafe.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run.

## Performance gate

Vacuum hot path. Bench deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone — **Task 50 §Exit Criteria target met for HNSW**

**Net -165 (-30.05%)** on HNSW: 549 → 384.
