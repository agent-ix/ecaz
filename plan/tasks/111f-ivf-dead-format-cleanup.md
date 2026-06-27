# Task 111f: IVF Dead Dense-Format Cleanup

Status: **proposed**.
Priority: P1 (integration / maintenance — do before the 111 → main merge).
Parent: `111-ivf-scan-dense-posting-block-layout.md`.
Evidence anchor: `reviews/task-111a/{004,007,008}`,
`reviews/task-111c/{002,003,004,005}` (the dominated/abandoned formats), and the
111a/111c closeouts.

## Goal

Remove the dead dense-posting format code from the 111 investigation — the
formats that were built, benchmarked, and **dominated by contiguous-copy** — so
the 111 lane lands on `main` carrying only the keepers. Preserve the rationale in
an ADR and the existing review packets; do not carry provably-slower, default-off
code or its on-disk tags forward.

This is the "strip dead code" half of the 111 → main integration decision. It
must not change any behavior of the keeper paths.

## Why

Across 111/111a/111b/111c the measured outcome was: page-local dense RaBitQ-1 +
Approach A coalescing + typed views is the only dense shape that wins; the
page-spanning packed format (Approach B), the columnar frozen-list format, and
the zero-copy page-scatter scorer were all dominated by the simple
contiguous-copy path (copy is cheap; locality beats scattered zero-copy;
pigeonhole: a large-payload width-≥32 batch can't be both contiguous and
zero-copy). Task 111e's `coarse_rerank` was then built on the **keeper** dense
path, not on any of the dead formats. So the dead code is unused, default-off,
provably slower, and a permanent on-disk-tag + maintenance liability. Strip it.

## Scope — REMOVE (dead)

1. **Approach B page-spanning packed** — tags `0x26`/`0x27`,
   `IvfDensePostingPacked*` tuple/ref/rewrite types, the packed build writer, the
   packed scan assembly (`IvfDensePackedPending`, `from_header`,
   `append_continuation`, `drain_dense_packed_group_to_scratch`), and the packed
   vacuum (`bulkdelete_dense_packed_segment`).
2. **111b columnar frozen-list** — tag `0x29`, `IvfColumnarFrozenList*`
   (columns/ref/pinned-pages/header), the columnar build writer + decode +
   page-aware reader, columnar vacuum, the `columnar_frozen_lists` reloption, and
   the columnar EXPLAIN counters (`columnar_frozen_lists_visited`,
   `columnar_postings_visited`, `columnar_logical_bytes_copied`).
3. **111c scatter scorer** — `IvfColumnarFrozenListPinnedPages`,
   `single_page_slice`, `payload_page_runs`, the borrowed columnar scan path, and
   the `ec_ivf.columnar_page_scatter` GUC.

## Scope — KEEP (do not touch behavior)

- Dense posting blocks (`0x25`) + aligned/typed layout (`0x28`) + the
  aligned LE typed-view accessors.
- Approach A scan-side coalescing (`ec_ivf.dense_posting_coalescing`,
  `dense_posting_typed_views`) and the dense/coalesced EXPLAIN counters.
- `coarse_rerank` (Task 111e) and its contract reloptions/admin snapshot.
- The `ecaz bench suite` extensions, corpus-subset CLI, and EXPLAIN counters
  unrelated to the removed formats.
- `dense_posting_blocks` and `dense_posting_typed_layout` reloptions.

## Known entanglements (handle carefully — these are why this is a real task)

- **`dense_posting_pack_pages`**: the `pack_pages > 1` branch is the dead
  Approach-B knob (`build.rs` ~L930 `if pack_pages > 1`, ~L1008 `if pack_pages
  <= 1`); `coarse_rerank` and dense force `pack_pages = 1`. Remove the `> 1`
  packed branch and the tests that set `pack_pages = 4` (`build.rs` ~L1826,
  ~L1860). Either drop the reloption and hardcode the one-page behavior, or pin
  it to 1 — but the one-page dense path must be byte-identical after the change.
- **`IvfPostingEntryRef` enum** (`page.rs` ~L2341): keep `Row` + `DenseBlock`;
  remove `DensePackedSegment`, `DensePackedContinuation`, `ColumnarHeader`
  variants and every match arm — scan dispatch (`scan.rs` ~L1689–1904 and
  ~L2034–2178), vacuum (`vacuum.rs` ~L301), and the tag decode/visitor
  (`page.rs` ~L4541–4563).
- **Tag decode**: keep `0x23` (row) / `0x25` (dense) / `0x28` (aligned); drop the
  `0x26`/`0x27`/`0x29` decode arms.
- Struct-field churn: removing reloptions touches the `EcIvfOptions` constructors
  in `build.rs`, `cost.rs`, `insert.rs`, `page.rs`, `scan.rs`, `options.rs`,
  `admin.rs`.

## Phases

1. **Map + remove behind compile.** Remove the three dead lines file by file
   (page.rs codec/tags/structs → scan.rs dispatch/scatter → build.rs writers →
   vacuum.rs → options.rs/admin.rs reloptions+GUC → cost.rs/insert.rs fields),
   keeping `cargo check --no-default-features --features pg18` green at each step.
2. **Regression-verify the keepers.** The dense + coarse_rerank paths must be
   unchanged: run the PG18 fixtures `coarse_rerank`, `dense_posting_blocks`,
   `dense_typed_posting_blocks`, `ivf_explain`, plus
   `cargo clippy --no-default-features --features pg18 -- -D warnings`.
3. **Docs + format-tag set.** Update `docs/on-disk-format.md` to the surviving
   tag set (`0x23`/`0x25`/`0x28`) and reconcile with Task 42.
4. **ADR.** Add an ADR capturing the durable negative result so the deletion
   doesn't erase the lesson: score-in-place columnar + scatter and page-spanning
   packed were built and benchmarked and lost to contiguous-copy; `coarse_rerank`
   on the keeper dense path is the path that survived.

## Acceptance Criteria

1. Tags `0x26`/`0x27`/`0x29` and all `IvfDensePostingPacked*` /
   `IvfColumnarFrozenList*` / pinned-pages / `columnar_page_scatter` symbols are
   gone from `src/`.
2. `coarse_rerank` and the dense/aligned/coalescing paths are behaviorally
   unchanged — the kept PG18 fixtures pass; clippy clean.
3. `docs/on-disk-format.md` lists only the surviving tags; Task 42 reconciled.
4. An ADR records the negative-result rationale.
5. No change to recall, scoring math, or the keeper on-disk bytes.

## Dependencies and Coordination

- This is the strip step of the 111 → main integration; do it on the 111 lane
  **before** merging the keepers to `main` and before branching 112/113/115 off
  a clean `main`.
- **Branch tangle:** a local `task-112-ivf-lazy-heap-f32-rerank` branch already
  exists (off the 111 line) and is unmerged. After this strip lands, that branch
  must rebase onto / absorb the cleanup so 112 doesn't reintroduce the dead code.
- Reviewer evidence + the review packets are the durable record of the removed
  formats; this task only removes the code, not the history.
