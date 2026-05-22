# Task 50/398: HNSW `shared::read_metadata_page` safe facade

## Why this slice

Per the HNSW-only redirect in
`reviews/task-50/392-completion-gate-audit/feedback/2026-05-21-02-reviewer.md`,
new Task 50 slices target HNSW structural reduction. This slice applies
Technique 1 (encapsulate at the FFI boundary) plus Technique 6 (delete dead
unsafe at callers) to `shared::read_metadata_page`, which is `pub(crate) unsafe
fn` purely so callers carry an `unsafe { ... }` wrapper despite the function
already operating entirely behind `LockedBufferGuard` (which is itself safe).

The function follows the same shape as `shared::read_main_buffer`, which is
already safe — a precedent established earlier in the burndown. Aligning
`read_metadata_page` with that precedent removes nine caller-side `unsafe { }`
blocks across HNSW (and one in `src/am/common/cost.rs`, whose only relevance
to this surface is the HNSW planner cost callback).

## Scope

- Convert `pub(crate) unsafe fn read_metadata_page` → `pub(crate) fn` in
  `src/am/ec_hnsw/shared.rs` and retain only the localized
  `unsafe { std::slice::from_raw_parts(raw_page, page_size) }` block whose
  contract is bounded by the `LockedBufferGuard` lifetime.
- Drop the `unsafe { ... }` wrapper at every direct caller:
  - `src/am/ec_hnsw/shared.rs:159, 530, 661, 726, 943` (in-file callers).
  - `src/am/ec_hnsw/vacuum.rs:38` (vacuum reader wrapper).
  - `src/am/ec_hnsw/scan_debug.rs:104` (debug helper).
  - `src/am/ec_hnsw/insert.rs:699` (first-insert refresh).
  - `src/am/common/cost.rs:381` (HNSW planner cost callback).

No new `unsafe` introduced. No helpers moved into shared modules. No behavior
change: function body is identical except that the no-longer-needed `unsafe
fn` declaration is dropped and stale SAFETY comments are pruned.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/shared.rs` | 35 | 30 | -5 |
| `src/am/ec_hnsw/vacuum.rs` | 56 | 55 | -1 |
| `src/am/ec_hnsw/scan_debug.rs` | 23 | 22 | -1 |
| `src/am/ec_hnsw/insert.rs` | 65 | 64 | -1 |
| `src/am/common/cost.rs` | 5 | 4 | -1 |
| **HNSW subsystem subtotal** | **549** | **541** | **-8** |
| **`src/` total** | **1119** | **1110** | **-9** |

The function body retains its single internal `unsafe { std::slice::from_raw_parts }`
block; conversion does not add any new `unsafe` site.

## Validation

Artifacts under `reviews/task-50/398-hnsw-read-metadata-page-safe/artifacts/`:

- `manifest.md` — head SHA, packet path, command, timestamps.
- `per-file-after.log` — post-change per-file block counts for HNSW files +
  `src/am/common/cost.rs`.
- `hnsw-unsafe-block-lines-after.log` — post-change line-by-line listing for
  every direct `unsafe { ... }` block in `src/am/ec_hnsw/`.
- `read-metadata-page-callsites.log` — every remaining `read_metadata_page`
  reference (only inside `LockedBufferGuard`-driven calls; no remaining
  `unsafe { ... read_metadata_page ... }` callers).
- `diff.patch` — exact diff applied.
- `cargo-check-pg18-bench.log` — `cargo check --all-targets
  --no-default-features --features pg18,bench` (full repo).

## Performance gate

Not on a hot path. `read_metadata_page` runs once per scan setup, once per
vacuum setup, and once per planner-cost callback invocation. It reads a single
shared-locked buffer and decodes a fixed-size metadata page. No traversal,
scoring, or cache hot path is touched. Per Task 50 §Performance Gate, no
before/after bench is required for this slice.

## Out of scope

- `src/am/ec_hnsw/build_parallel.rs:2797` `BuildIndexInfo` split into
  `IndexInfoGuard` (owns pfree) + `IndexInfoView<'a>` (borrows) — named by
  reviewer feedback `358/2026-05-21-02-reviewer.md` and queued as the next
  HNSW slice in this rotation.
- Any SPIRE / IVF / RaBitQ / DiskANN / common-parallel work — HNSW-only
  rotation per `392/2026-05-21-02-reviewer.md`.
