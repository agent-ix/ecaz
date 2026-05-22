# Packet 403 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/403-hnsw-shared-header-inline-fix/`
Surface: HNSW build_parallel — anti-pattern B fix from packet 401
Branch: `task-50-unsafe-closeout`
Pre-slice HEAD: packet 402 commit / merge with origin
Slice commit SHA: `cfc0ecc714041cb6e338d83ca2545bd1ba7a238c`
Slice timestamp: 2026-05-22 (PT)

## Slice summary

- Reverts packet 401's `shared_header_ref<'a>(*mut T) -> &'a T` helper.
- Inlines `NonNull::new(...).unwrap_or_else(...).as_ref()` at each of the
  two worker entrypoints inside a bounded `unsafe { ... }` block with a
  context-specific SAFETY comment.
- Preserves the call-site (*shared).field deletions and the
  `participant_count` hoist from packet 401 — only the helper signature
  was the problem.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | +1 |
| **HNSW subsystem subtotal** | **+1** |

Cumulative rotation delta remains -20 in HNSW.

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | `for f in src/am/ec_hnsw/*.rs; do c=$(grep -c 'unsafe\\s*{' "$f"); printf "%4d  %s\\n" "$c" "$f"; done \| sort -rn` | request.md unsafe table |
| `diff.patch` | `git diff src/am/ec_hnsw/build_parallel.rs` | exact diff |
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` (lib smoke) | compile validation |

## Validation rule mapping (Task 50 §Validation)

- `cargo fmt --all` — not run; formatting-neutral edits.
- `cargo check --no-default-features --features pg18` — captured.
  Clean, no errors, no `unused_unsafe` warnings.
- Direct unsafe-block count per touched file: see `per-file-after.log`.
- Runtime tests — not run. The slice is semantically identical to packet
  401: same borrow, same field reads, same downstream call shape.

## Performance gate

Build hot path. No semantic change vs slice 401. Bench evidence deferred per
`feedback_coder_push_smoke_checks` (2026-05-21).
