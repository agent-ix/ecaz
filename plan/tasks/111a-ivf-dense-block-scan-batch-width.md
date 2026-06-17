# Task 111a: IVF Dense Block Scan Batch Width

Status: **proposed**.
Priority: P0 latency (Task 111 follow-on).
Parent: `111-ivf-scan-dense-posting-block-layout.md`.
Evidence anchor: `reviews/task-111/006-benchmark-gate/` (Phase 5 benchmark gate).

## Goal

Eliminate the TurboQuant dense-posting-block latency regression measured in
Task 111 Phase 5 by restoring SIMD-efficient batch widths (≥32) on the dense
scan path, without losing the RaBitQ dense win, without changing recall, and
without giving up the dense-layout index-size reduction.

The promotion decision for dense posting blocks (`dense_posting_blocks`)
remains gated on this work: Task 111 closed with an "iterate, do not promote"
verdict precisely because TurboQuant dense regressed p50/p95 at every measured
cell.

## Why

Task 111 packet 006 established, against verified raw artifacts, that:

- Recall is identical row vs dense for TurboQuant and RaBitQ.
- Dense reduces index size (TQ ≈ −8.7 MiB, RaBitQ ≈ −7.2 MiB at real 100k) and
  eliminates the row→scratch SoA copy.
- TurboQuant dense regressed p50/p95 (e.g. nprobe=32: 31.7→39.2 ms p50), while
  RaBitQ dense improved (14.4→12.3 ms p50).

Root cause (confirmed from `block-kernel-counters` + EXPLAIN logs): dense blocks
are capped at one page and `process_dense_posting_block` scores exactly one
block's postings per batch call, so achievable SIMD width is bounded by
postings-per-block. TurboQuant's larger payload yields ≈10 postings/block
(4233 blocks / 42171 postings), so dense TQ flushes almost entirely at
`width_8_15` and never reaches `width_ge32`. RaBitQ yields ≈35 postings/block
(1189 blocks / 42171 postings), so it clears 32 and wins. The legacy row path
avoids this because its scratch-SoA accumulator batches *across* pages up to
`IVF_POSTING_SCRATCH_SOA_BATCH_POSTINGS`, staying at width≥32 for both formats.

The dense layout removed the scratch copy (the Task 111 goal) but, as a side
effect, also removed the cross-page batching that copy was providing.

## Scope

Implement and benchmark **both** candidate fixes head-to-head, then recommend a
winner with evidence:

- **Approach A — scan-side cross-block coalescing.** Keep one-page dense blocks
  on disk. In the scan, accumulate postings from consecutive dense blocks into a
  reusable buffer and flush to the batch scorer only at a target width (≥32 /
  `IVF_POSTING_SCRATCH_SOA_BATCH_POSTINGS`). Because dense blocks are already
  structure-of-arrays, accumulation is a bulk contiguous copy of payload bytes
  (plus gammas/heap-tids), not the row path's per-field scatter-gather reshape.
  No durable on-disk format change.
- **Approach B — on-disk dense block packing.** Make a logical dense block span
  more than one page (chained / overflow representation) so a single
  `process_dense_posting_block` call already holds ≥32 postings and can score
  zero-copy without per-batch concatenation. This is a durable page-format
  change and requires Task 42 coordination.

Both approaches stay behind the existing `dense_posting_blocks` gate and remain
default-off until a promotion decision.

## Non-Goals

- Do not change scoring math, quantization, or recall behavior.
- Do not implement residual quantization (Task 115).
- Do not implement dense-block lifecycle / vacuum repack / delta-posting density
  policy (Task 114); include only the minimum correctness handling already
  present from Task 111.
- Do not change nprobe, centroid routing, or adaptive probing.
- Do not change SPIRE, HNSW, or DiskANN layout or scan.
- Do not promote `dense_posting_blocks` to default in this task; promotion is a
  separate explicit decision after the benchmark gate.

## Phases

### Phase 1 - Coalescing prototype (Approach A)

- Add a scan-opaque-owned coalescing buffer (mirror the existing
  `IvfDensePostingBlockScratch` / SoA scratch lifecycle).
- Accumulate dense-block postings across consecutive blocks in a selected list
  range; flush to the batch scorer at the target width.
- Preserve candidate dedup, deleted-bitmap filtering, live-tid budget, and
  heap-tid expansion semantics exactly as the current dense path.
- Add EXPLAIN/batch counters for coalesced flush count and width histogram so
  the width shift is observable.

### Phase 2 - Packing prototype (Approach B)

- Add a build-time writer for multi-page (chained/overflow) dense blocks behind
  the gate, with encode/decode invariants and focused page-format tests.
- Add scan support that scores a packed block zero-copy at width≥32.
- Define the durable format-version gate and old-index compatibility contract;
  coordinate with Task 42 before landing any durable format change.

### Phase 3 - Head-to-head benchmark gate

- Drive everything through `ecaz bench suite` with a committed `SuiteConfig`.
- Compare, per storage format and nprobe cell: row, dense (current), dense+A,
  dense+B for TurboQuant and RaBitQ.
- Cells: real 100k **and 1M** (1M is required for the promotion question; it is
  an AWS lane reading the 1M corpus base — local hosts only stage 10k/100k).
- Use the registered `ec_ivf` default sweep `[8,16,24,32,48,64]`; do not subset
  without a manifest-stated reason.
- Report warm latency p50/p95/p99, recall@10 + NDCG@10, build time, index size,
  posting pages, candidates, scan counters, and SIMD flush-width histograms.

## Acceptance Criteria

1. Approach A (coalescing) implemented behind the `dense_posting_blocks` gate.
2. Approach B (packing) implemented behind the gate (or, if a phase establishes
   one approach is strictly dominated, that approach may be closed early with
   evidence rather than fully built — state the reason in the packet).
3. Recall and NDCG unchanged vs the legacy row path for every compared cell.
4. SIMD flush-width counters show dense TurboQuant reaching width≥32.
5. A benchmark packet reports the head-to-head matrix at real 100k and 1M for
   TurboQuant and RaBitQ with latency, recall, build time, index size, pages,
   candidates, and flush-width histograms.
6. The packet explicitly recommends which approach to adopt (A, B, or neither),
   and an explicit promote / iterate / abandon decision for
   `dense_posting_blocks` as a default.

## Promotion Criteria (for the eventual default decision)

- Recall unchanged for the same query set and reloptions.
- TurboQuant dense p50 improves (or at least matches row) at the target
  high-recall cells with no p95/p99 regression that erases the win.
- RaBitQ dense retains its Task 111 latency win.
- Index-size and build-time deltas reported and justified.
- 1M evidence present (not 100k-only) before any default move.

## Dependencies and Coordination

- Builds directly on Task 111 (gated dense blocks, dense scan, vacuum, Phase 5
  evidence). Reuses the Task 111 dense codec, deleted bitmap, and scan path.
- Approach B requires Task 42 (on-disk-format invariants) coordination if it
  introduces a durable page-format version; reconcile the existing experimental
  `0x25` dense tag at the same time.
- Task 114 owns dense-block lifecycle/repack; keep that out of scope here.
- 1M benchmark cells are an AWS lane: confirm the real DBpedia corpus base
  snapshot (`snap-0e9c7743263e61d70`, last-known base) via `describe-snapshots`
  before relying on it; never recreate the corpus.
