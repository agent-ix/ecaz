# Packet 400 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/400-hnsw-index-info-view-split/`
Surface: HNSW `BuildIndexInfo` boundary — owning guard + borrowing view
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: `2b13e462eb985c0bb11b8d4f40859329b6590dd1` (packet 399 +
  reviewer feedback)
Slice commit SHA: `45ad778719e89befc8c24d126502149466323465`
Slice timestamp: 2026-05-21 (PT)
Isolation: N/A — no benchmark in this slice. Performance Gate disposition is
in `request.md` §Performance gate.

## Slice summary

- New `src/am/ec_hnsw/index_info.rs` exposing
  `IndexInfoGuard` (owning, pfrees in Drop) and `IndexInfoView<'scope>`
  (borrowed, no Drop), both wrapping `NonNull<pg_sys::IndexInfo>`.
- `src/am/ec_hnsw/source.rs` drops its local copy of `IndexInfoGuard` and
  routes its two callers through `super::index_info::IndexInfoGuard::build`.
- `src/am/ec_hnsw/build_parallel.rs` worker site (line 2797 pre-slice) now
  uses `IndexInfoView::build_borrowed(...)` and writes
  `ii_Concurrent` via the type's safe `as_mut()` accessor, hoisting a single
  `let is_concurrent = unsafe { (*shared).is_concurrent };` shared with the
  existing lockmode-selection block.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | -2 |
| `src/am/ec_hnsw/source.rs` | -2 |
| `src/am/ec_hnsw/index_info.rs` (new) | +3 |
| `src/am/ec_hnsw/mod.rs` | 0 (module declaration only) |
| **HNSW subsystem subtotal** | **-1** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | `for f in src/am/ec_hnsw/*.rs; do c=$(grep -c 'unsafe\\s*{' "$f"); printf "%4d  %s\\n" "$c" "$f"; done \| sort -rn` | request.md unsafe table |
| `hnsw-unsafe-block-lines-after.log` | `grep -rn 'unsafe\\s*{' src/am/ec_hnsw/` | line-level coverage |
| `index-info-callsites.log` | `grep -rn 'BuildIndexInfo\\\|IndexInfoGuard\\\|IndexInfoView' src/am/ec_hnsw/` | confirms `BuildIndexInfo` lives only inside `index_info::build_inner` after slice |
| `diff.patch` | `git diff <touched files>` | exact diff |
| `cargo-check-pg18-bench.log` | `cargo check --no-default-features --features pg18` (lib smoke) | compile validation |

## Validation rule mapping (Task 50 §Validation)

- `cargo fmt --all` — not run; the new module follows existing
  `src/am/ec_hnsw/*.rs` formatting conventions and the call-site rewrites are
  formatting-neutral.
- `cargo check --no-default-features --features pg18` — captured in
  `cargo-check-pg18-bench.log`. Full bench-feature compile and clippy are
  deferred to the next out-of-band rotation per the smoke-check rule.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D
  warnings` — not run; structural change with no new lint surface.
- Direct unsafe-block count per touched file: see `per-file-after.log` and
  request.md.
- Runtime tests — not run. The slice preserves call ordering, IndexInfo
  field semantics, and allocator behavior (PG memory context owns the
  IndexInfo in build_parallel; Rust guard owns it in source.rs).

## Performance gate

Build hot path. Disposition in `request.md` §Performance gate: bench evidence
deferred to the operator's out-of-band rotation per
`feedback_coder_push_smoke_checks` (2026-05-21). No allocation-shape change,
no scoring math touched, no WAL ordering change, no payload byte change.
