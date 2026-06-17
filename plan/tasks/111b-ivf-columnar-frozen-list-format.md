# Task 111b: IVF Columnar Frozen-List Format

Status: **complete** (2026-06-17; closeout
`reviews/task-111b/009-closeout-status/`). Format, writer, decode, scan,
vacuum, old-index compatibility, counters, and the 50k/100k benchmark baseline
landed. The format is **not promoted**; packet 008/009 defer storage-density and
score-in-place decisions to 111c/111d.
Priority: P0 latency (Task 111 line; score-in-place foundation).
Parent: `111-ivf-scan-dense-posting-block-layout.md`,
follows `111a-ivf-dense-block-scan-batch-width.md`.
Evidence anchor: `reviews/task-111a/{004,007,008}` (A wins vs the page-spanning
packed format because A avoids fragmentation; B lost on copy + per-segment
overhead).

## Goal

Replace the page-bound dense posting *tuple* (block/group) with a **columnar
frozen-list on-disk format**: store each frozen IVF list as parallel columns
packed across raw pages, so the scan can eventually feed the SIMD scorer wide
batches straight from the page cache with no assembly copy and no per-tuple
fragmentation.

This task lands the **format and its correctness** (writer, decode, vacuum,
mixed scan, old-index compatibility) behind a gate, validated with a
copy-based scan. It deliberately does **not** yet add the zero-copy scatter
scorer — that is Task 111c, which turns this format into the latency win. 111b
on its own is expected to roughly match Approach A (it still copies to score);
its purpose is to de-risk the durable on-disk change and the Task 42
coordination independently before the kernel work.

## Why

Task 111a measured the full matrix (TQ + RaBitQ {1,2,4,8}, 50k/100k) and showed:

- Approach A (scan-side coalescing on compact one-page blocks) wins because it
  gets wide batches with only a scratch copy and no fragmentation.
- Approach B (page-spanning packed *tuples*) lost on latency, storage, page
  reads, and copy bytes — the per-segment headers + line pointers + assembly
  copy dominate (16.6/32.8/65.2 MB copied per 100k scan at rb2/4/8).

The conclusion was not "stop" but "the tuple abstraction on the hot path is the
problem." Both A and B pay to escape the 8 KB page boundary — A with a copy, B
with fragmentation. A columnar layout removes the abstraction: payloads are
packed as a column across raw pages with only the unavoidable page header
between runs, no per-posting/per-segment tuple overhead. This is the storage
foundation for a scan that reads each payload byte exactly once (Task 111c).

## Scope

- A durable columnar frozen-list format for build-time (frozen) IVF lists:
  - **hot columns**: `gamma[]` (f32), `payload[]` (quantized codes), packed
    contiguously across raw pages, **page-aligned to whole postings** (pad each
    page's payload run to a `payload_len` boundary; waste ≤ payload_len−1/page);
  - **cold columns**: `heap_tid[]`, `rerank_tid[]` (touched only for survivors);
  - per-list **deleted bitmap** (metadata-once for the whole list);
  - a per-list header: count, payload_len, format tag/version, column offsets.
- Build-time writer emitting the columnar layout deterministically.
- Decode / readers for each column with the aligned typed-view fast path from
  111a packet 003 reused for the `gamma`/offset columns (LE + alignment guard +
  byte-decode fallback).
- A **page-aware reader** that walks a column across page boundaries (the
  reader the 111c scatter scorer will later consume); for 111b it may copy
  page-aligned posting runs into scratch to feed the existing kernels
  (correctness, not yet the no-copy win).
- Hot/cold separation: the scan reads only `gamma`+`payload` for scoring;
  `heap_tid`/`rerank_tid` fetched lazily for the ~k survivors.
- Vacuum: mark deletes in the per-list bitmap in place (size-preserving).
- Mixed scan: frozen columnar region (this format) + mutable delta row-postings
  (existing append area, scored via Approach A coalescing) in the same list.
- Old-index compatibility: existing dense (`0x25`) / aligned (`0x28`) and row
  indexes remain readable; the columnar format is a new gated tag/version.
- Behind the existing `dense_posting_blocks` gate (or a new explicit reloption);
  default off.

## Non-Goals

- The zero-copy page-aware **scatter scorer** (Task 111c).
- **Pre-transposed** canonical block geometry (Task 111d).
- Host-pinned compaction escape hatch (future Task 111e; deferred).
- Changing scoring math, quantization, recall, nprobe, or routing.
- SPIRE / HNSW / DiskANN.
- Promoting the format to default (separate decision after 111c evidence).

## Phases

1. **Format + writer + decode.** Define the columnar tag/version and column
   layout; deterministic build writer; decode with typed-view reuse; static
   encode/decode invariants + focused page-format tests.
2. **Page-aware reader + correctness scan.** Read columns across page
   boundaries; score via the existing kernels (copy page-aligned runs to scratch
   if needed); preserve candidate dedup, deleted filtering, live-tid budget,
   heap-tid expansion. PG18 fixtures for a multi-page columnar list.
3. **Vacuum + mixed scan.** Per-list deleted bitmap vacuum; mixed frozen-column
   + delta-row-posting scan; deletes across both; PG18 fixtures incl. churn.
4. **Old-index compatibility + correctness benchmark.** Confirm old dense/row
   indexes still read; run the standard matrix to confirm **recall unchanged**
   and quantify storage/page-reads vs A and vs the 111a dense formats (latency
   parity-with-A is acceptable here — the win lands in 111c).

## Acceptance Criteria

1. Columnar frozen-list format implemented behind a gate; deterministic build.
2. Existing row / dense (`0x25`) / aligned (`0x28`) indexes remain readable.
3. Recall and NDCG unchanged vs the legacy path for all compared cells.
4. Mixed frozen-column + delta-row scan and vacuum return the same candidates as
   the legacy path under controlled fixtures, including after deletes.
5. A benchmark packet reports storage and posting-pages-read vs Approach A and
   the 111a dense formats across TQ + RaBitQ {1,2,4,8} at 50k/100k, plus
   recall/latency. Storage/page-read reduction (or parity) is the headline; this
   task is not required to beat A on latency.
6. The packet records the on-disk tag/version set and the Task 42 reconciliation
   plan (keep simple/typed dense tags; the columnar tag is new; retire the
   abandoned 111a packed tags `0x26`/`0x27`).

## Dependencies and Coordination

- Reuses the 111a aligned typed-view accessors (packet 003) for `gamma`/offset
  columns.
- Task 42 (on-disk-format invariants): a new durable format version — coordinate
  before landing; enumerate the final dense tag set.
- Task 114 owns dense-block lifecycle/repack under churn; 111b includes only the
  minimum delete/vacuum needed for correctness.
- Enables Task 111c (scatter scorer) and Task 111d (pre-transpose).
