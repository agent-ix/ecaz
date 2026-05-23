# Packet 401 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/401-hnsw-shared-header-ref/`
Surface: HNSW build_parallel — shared-header safe borrow helper
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: `af314c2cb` (packet 400 commit)
Slice commit SHA: `ef6e29b8c8de766061155a80fb97aa301a9e7409`
Slice timestamp: 2026-05-21 (PT)

## Slice summary

- Added `shared_header_ref(shared) -> &'a EcHnswParallelBuildSharedHeader`
  module-private helper inside `src/am/ec_hnsw/build_parallel.rs`.
- Converted 5 worker-entry `(*shared).field` reads to use `header.field` /
  `header.method()` after binding a single borrow.
- Hoisted `(*shared).participant_count` out of the
  `insert_concurrent_dsm_graph_participant` call, which lets the outer
  `unsafe { ... }` wrapper around that call disappear (the function itself
  is safe).

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | -5 |
| **HNSW subsystem subtotal** | **-5** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | `for f in src/am/ec_hnsw/*.rs; do c=$(grep -c 'unsafe\\s*{' "$f"); printf "%4d  %s\\n" "$c" "$f"; done \| sort -rn` | request.md unsafe table |
| `build-parallel-unsafe-block-lines-after.log` | `grep -rn 'unsafe\\s*{' src/am/ec_hnsw/build_parallel.rs` | line-level coverage |
| `shared-deref-sites-after.log` | `grep -nE '(\\*shared)' src/am/ec_hnsw/build_parallel.rs` | residual `(*shared)` derefs (all SpinLock/CV/init) |
| `diff.patch` | `git diff src/am/ec_hnsw/build_parallel.rs` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` (lib smoke) | compile validation |

## Validation rule mapping (Task 50 §Validation)

- `cargo fmt --all` — not run; the helper follows existing
  `build_parallel.rs` formatting and the call-site rewrites are
  formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured in
  `cargo-check-pg18.log`. No errors, no `unused_unsafe` warnings after
  the `insert_concurrent_dsm_graph_participant` outer wrapper was removed.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D
  warnings` — not run; no new lint surface, structural change only.
- Direct unsafe-block count per touched file: see `per-file-after.log` and
  request.md.
- Runtime tests — not run. The slice preserves spinlock ordering, the
  scanned/encoded worker tally, the worker count, and the lock-mode
  selection branch.

## Performance gate

Build hot path. Disposition in `request.md` §Performance gate: bench evidence
deferred to the operator's out-of-band rotation per
`feedback_coder_push_smoke_checks` (2026-05-21). The change does not alter
field semantics, worker scheduling, locking, or allocation behavior.
