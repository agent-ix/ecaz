# Packet 425 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/425-hnsw-shared-snapshots-safe/`
Surface: HNSW shared.rs — snapshot + count helpers safe-fn cascade
Branch: `task-50-hnsw`
Pre-slice HEAD: packet 424 commit (`538852da1`)
Slice commit SHA: `9e096b25b4d346d4383b0ed1c05d75d93613dcd4`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

Eight `unsafe fn` helpers in shared.rs (the count + snapshot family)
lifted to safe `fn` after the slice-424 cascade removed their last
internal unsafe blocks. Seven caller-side `unsafe { ... }` wraps
stripped across shared.rs / scan.rs / vacuum.rs. The library-wide
`with_live_index_relation!` macro at `src/lib.rs:435` gets
`#[allow(unused_unsafe)]` to stay shape-agnostic for the mix of
safe / unsafe-fn callers.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/shared.rs` | -6 |
| `src/am/ec_hnsw/scan.rs` | -1 |
| `src/am/ec_hnsw/vacuum.rs` | -2 |
| `src/lib.rs` | macro attribute only |
| **HNSW subsystem subtotal** | **-9** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | per-file grep | request.md |
| `diff.patch` | `git diff src/` | exact diff (179 lines) |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | compile validation; 0 `unused_unsafe` warnings |

## Validation rule mapping

- `cargo fmt --all` — not run; formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured. Clean.
- Direct unsafe-block count per touched file: `per-file-after.log`.
- Runtime tests — not run. Same metadata reads, same count loops,
  same snapshot field reads.

## Performance gate

Diagnostic / cost-callback cold paths. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**Past the -27% threshold** on HNSW: 549 → 398, net -151 (-27.5%).
