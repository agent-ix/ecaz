# Packet 398 — Artifact Manifest

Task: 50 — Unsafe Structural Reduction
Packet: `reviews/task-50/398-hnsw-read-metadata-page-safe/`
Surface: HNSW shared metadata read facade
Branch: `task-50-unsafe-closeout`
Head SHA at slice start: `e4c4749d6debd0a2b9d25f370825088a7ab453a8`
(packet 358 + reviewer feedback 358/02 redirect)
Slice commit SHA: `37115e7764ac48d696eaf620c766ecf1c276ffb5`
Slice timestamp: 2026-05-21 (PT)
Isolation: N/A — no benchmark evidence; the slice does not touch any scoring,
traversal, or cache hot path.

## Slice summary

- Convert `pub(crate) unsafe fn read_metadata_page` (HNSW shared module) to
  `pub(crate) fn`, retaining only the localized `from_raw_parts` block whose
  contract is bounded by the existing `LockedBufferGuard`.
- Strip nine `unsafe { ... }` wrappers at callers in HNSW + the HNSW-facing
  planner cost callback in `src/am/common/cost.rs`.

## Files touched

| File | Δ unsafe blocks |
| --- | ---: |
| `src/am/ec_hnsw/shared.rs` | -5 |
| `src/am/ec_hnsw/vacuum.rs` | -1 |
| `src/am/ec_hnsw/scan_debug.rs` | -1 |
| `src/am/ec_hnsw/insert.rs` | -1 |
| `src/am/common/cost.rs` | -1 |
| **Total** | **-9** |

## Artifacts

| File | Source command | Cites |
| --- | --- | --- |
| `per-file-after.log` | `for f in src/am/ec_hnsw/*.rs src/am/common/cost.rs; do c=$(grep -c 'unsafe\\s*{' "$f"); printf "%4d  %s\\n" "$c" "$f"; done \| sort -rn` | request.md unsafe table |
| `hnsw-unsafe-block-lines-after.log` | `grep -rn 'unsafe\\s*{' src/am/ec_hnsw/` | line-level coverage check |
| `read-metadata-page-callsites.log` | `grep -rn 'read_metadata_page' src/am/ec_hnsw/ src/am/common/cost.rs` | confirms no remaining `unsafe { ... read_metadata_page ... }` callers |
| `diff.patch` | `git diff <touched files>` | exact diff |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | compile validation |

## Validation rule mapping (Task 50 §Validation)

- `cargo fmt --all` — not run; only formatting-neutral edits (removed lines and
  one keyword removal). Re-run is cheap if reviewer wants explicit evidence.
- `cargo check --all-targets --no-default-features --features pg18,bench` —
  captured in `cargo-check-pg18-bench.log`.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D
  warnings` — not run for this slice; only callsite-unsafe deletion + one
  function signature flip. No new types, traits, or surface area.
- Direct unsafe-block count per touched file: in `per-file-after.log` and
  `request.md`.
- Runtime tests on touched module — not run. The function body is unchanged;
  the conversion is signature-only. No callback ordering, no tuple
  visibility, no WAL mutation, no vector decoding.

## Performance gate

Not applicable. `read_metadata_page` is invoked on scan-setup, vacuum-setup,
planner-cost callback, and debug helpers. It is not in any scoring, traversal,
or cache hot path. Per Task 50 §Performance Gate, no bench is required.
