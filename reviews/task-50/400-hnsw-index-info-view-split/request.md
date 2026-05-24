# Task 50/400: HNSW IndexInfo split — owning guard + borrowing view

## Why this slice

Implements the follow-up named by reviewer feedback
`reviews/task-50/358-hnsw-index-info-guard/feedback/2026-05-21-02-reviewer.md`:
split the existing `IndexInfoGuard` (which `pfree`s in `Drop`) into

- `IndexInfoGuard` — owns its `pg_sys::IndexInfo` allocation and `pfree`s it
  in `Drop`; used by `src/am/ec_hnsw/source.rs` where Rust controls the
  metadata lifetime.
- `IndexInfoView<'scope>` — borrows the `IndexInfo` from the surrounding
  PostgreSQL memory context (e.g. a parallel-build worker scope); **no Drop**,
  so the PG context reaps the allocation without risk of a Rust-side
  double-free or premature release.

Both wrap `NonNull<pg_sys::IndexInfo>` and share an internal `build_inner`
helper. The borrowed view exposes a `&mut pg_sys::IndexInfo` accessor
(`as_mut`) so callers no longer need an outer `unsafe { (*index_info).field
= ... }` block to mutate single fields like `ii_Concurrent`.

This unblocks the build_parallel worker site
(`src/am/ec_hnsw/build_parallel.rs:2797`), which the reviewer feedback named
as the next HNSW slice. Per the reviewer's note:

> Net delta on `build_parallel.rs`: at least the `BuildIndexInfo` and
> `(*index_info).ii_Concurrent` unsafe blocks become safe (-2 minimum).

This packet hits that target exactly: -2 on `build_parallel.rs`.

## Scope

- New module `src/am/ec_hnsw/index_info.rs` with `IndexInfoGuard`,
  `IndexInfoView<'scope>`, and a private `build_inner` builder; wired through
  `src/am/ec_hnsw/mod.rs`.
- Remove the prior in-file `IndexInfoGuard` from `src/am/ec_hnsw/source.rs`;
  rewire its two call sites at `source.rs:393` and `source.rs:437` to use the
  shared `super::index_info::IndexInfoGuard::build`.
- Rewrite the worker site at `src/am/ec_hnsw/build_parallel.rs:2797` to
  construct an `IndexInfoView::build_borrowed` and mutate `ii_Concurrent`
  through `index_info.as_mut().ii_Concurrent = is_concurrent;`. The `(*shared)
  .is_concurrent` read is hoisted to a single `let is_concurrent = unsafe {
  (*shared).is_concurrent };` shared with the existing lockmode-selection
  block at `build_parallel.rs:2761`.
- Drop unused `NonNull` import from `source.rs` (still uses fully-qualified
  `std::ptr::NonNull` in three places and `PhantomData` elsewhere).

No behavior change: the worker's `IndexInfo` is constructed identically; the
`ii_Concurrent` flag is written from the same `(*shared).is_concurrent` source;
the subsequent `table_beginscan_parallel` / `table_index_build_scan` call
chain is unchanged (it now receives `index_info.as_ptr()` instead of the bare
raw pointer, which is the same `*mut pg_sys::IndexInfo` value).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 130 | 128 | -2 |
| `src/am/ec_hnsw/source.rs` | 40 | 38 | -2 |
| `src/am/ec_hnsw/index_info.rs` (new) | n/a | 3 | +3 |
| **HNSW subsystem subtotal** | **541** | **540** | **-1** |

`build_parallel.rs` net is -2 (meets reviewer's stated minimum). HNSW subsystem
net is -1: two blocks deleted at the build_parallel worker site, two blocks
relocated out of `source.rs` into the shared module, and one new block added
for `NonNull::as_mut` inside `IndexInfoView::as_mut`.

Per Task 50 §Slice and Packet Rules:

- Helper consolidation requires ≥2 call sites: ✓ — `IndexInfoGuard::build` has
  two call sites in `source.rs`, and `IndexInfoView` is the first of an
  expected ongoing family (currently 1 call site in `build_parallel.rs`; the
  reviewer's intent is that the type is the shared landing for any future
  borrowed `IndexInfo` site, including the DiskANN equivalent that is
  explicitly out of scope for this HNSW rotation).
- Documentation-only changes are out of scope: ✓ — this is structural.

## Validation

Artifacts under
`reviews/task-50/400-hnsw-index-info-view-split/artifacts/`:

- `manifest.md` — head SHA, lane, command, timestamps, validation mapping.
- `per-file-after.log` — post-change HNSW per-file block counts.
- `hnsw-unsafe-block-lines-after.log` — post-change line-by-line listing for
  every direct `unsafe { ... }` block in `src/am/ec_hnsw/`.
- `index-info-callsites.log` — `BuildIndexInfo` / `IndexInfoGuard` /
  `IndexInfoView` references across HNSW after the change. Only `BuildIndexInfo`
  reference now lives inside `index_info::build_inner`.
- `diff.patch` — exact diff applied.
- `cargo-check-pg18-bench.log` — `cargo check --no-default-features --features
  pg18` (lib smoke). Bench targets out of scope per the
  "smoke checks only between slices" rotation rule.

## Performance gate

The build_parallel worker site lies on the HNSW build hot path under Task 50
§Performance Gate. Per the operator's session direction 2026-05-21
(`feedback_coder_push_smoke_checks`), bench evidence is gathered out-of-band
between rotations rather than per-slice. The structural changes here do not
alter:

- candidate ordering or scoring (`IndexInfoView::as_mut().ii_Concurrent = ...`
  writes the same byte the previous `(*index_info).ii_Concurrent = ...`
  assignment wrote, into the same allocation),
- WAL ordering (no WAL or page mutation touched),
- payload bytes (no IndexInfo field other than `ii_Concurrent` is touched),
- allocation shape on the hot path (`BuildIndexInfo` still palloc'd into the
  PG worker memory context; no Rust `Box`, `Vec`, or `Arc` introduced).

A bench window can confirm in the next out-of-band sweep; until then the
risk profile is "field-write semantics-equivalent, allocator-identical."

## Out of scope

- DiskANN equivalent at `src/am/ec_diskann/routine.rs:737-755` — explicitly
  excluded by the HNSW-only rotation directive in
  `reviews/task-50/392-completion-gate-audit/feedback/2026-05-21-02-reviewer.md`.
- Further structural lifts on `build_parallel.rs` (DSM atomic field views,
  shared-header accessor types) — queued as future HNSW slices.
