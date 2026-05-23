# Packet 402 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/402-hnsw-shm-toc-lookup-typed/`
Surface: HNSW build_parallel — typed shm_toc_lookup helper
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 401 commit (`0941456f9`)
Slice commit SHA: `be3089a17fc60d24dd65fda5dbaf4b1eb47ab3be`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

- Added `shm_toc_lookup_required<T>(toc, key) -> *mut T` module-private
  helper to `src/am/ec_hnsw/build_parallel.rs`, centralizing the
  `unsafe { pg_sys::shm_toc_lookup(toc, key, false) }.cast::<T>()` pattern.
- Converted 8 caller sites across `parallel_build_worker_main` and
  `parallel_graph_build_worker_main` to safe calls.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | -7 |
| **HNSW subsystem subtotal** | **-7** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | `for f in src/am/ec_hnsw/*.rs; do c=$(grep -c 'unsafe\\s*{' "$f"); printf "%4d  %s\\n" "$c" "$f"; done \| sort -rn` | request.md unsafe table |
| `build-parallel-unsafe-block-lines-after.log` | `grep -rn 'unsafe\\s*{' src/am/ec_hnsw/build_parallel.rs` | line-level coverage |
| `shm-toc-lookup-sites-after.log` | `grep -n 'shm_toc_lookup' src/am/ec_hnsw/build_parallel.rs` | confirms all worker sites use the helper; one leader-side lookup remains intentionally out of scope |
| `diff.patch` | `git diff src/am/ec_hnsw/build_parallel.rs` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` (lib smoke) | compile validation |

## Validation rule mapping (Task 50 §Validation)

- `cargo fmt --all` — not run; new helper follows existing formatting.
- `cargo check --no-default-features --features pg18` — captured. Clean,
  no errors, no `unused_unsafe` warnings.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D
  warnings` — not run; structural change with no new lint surface.
- Direct unsafe-block count per touched file: see `per-file-after.log`.
- Runtime tests — not run. Slice preserves all worker setup semantics
  (same lookup arguments, same key, same `noerror = false` behavior).

## Performance gate

Build hot path. Disposition in `request.md` §Performance gate: bench
evidence deferred to the operator's out-of-band rotation per
`feedback_coder_push_smoke_checks` (2026-05-21).
