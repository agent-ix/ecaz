# Packet 419 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/419-hnsw-graph-storage-descriptor-from-index-relation-safe/`
Surface: HNSW graph.rs — `GraphStorageDescriptor::from_index_relation` safe-fn lift
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 418 commit (`3e22f63a6`)
Slice commit SHA: `19189cd5420a5e6c364cbbdfbb12b903d196584a`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

`graph::GraphStorageDescriptor::from_index_relation` lifted from
`pub(crate) unsafe fn` to `pub(crate) fn`. Body had zero internal
unsafe blocks (`relation_options` already takes
`NonNull<RelationData>`). Five caller-side `unsafe { ... }` wraps
removed across HNSW.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/scan.rs` | -1 |
| `src/am/ec_hnsw/insert.rs` | -1 |
| `src/am/ec_hnsw/vacuum.rs` | -1 |
| `src/am/ec_hnsw/shared.rs` | -1 |
| `src/am/ec_hnsw/scan_debug.rs` | -1 |
| `src/am/ec_hnsw/graph.rs` | 0 (signature only) |
| **HNSW subsystem subtotal** | **-5** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/am/ec_hnsw/` | exact diff (83 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Function body is unchanged; the lift is
  signature-only.

## Performance gate

Cold path (graph-storage discriminator resolution). Bench deferred
per `feedback_coder_push_smoke_checks`.
