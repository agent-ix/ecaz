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

### Phase 2 - Logical-group page-spanning prototype (Approach B)

The unit is a **logical dense group sized to the scorer width** (e.g. 32 or 64
postings), NOT a page-bound block. PostgreSQL page capacity must stop defining
SIMD batch size; physical segment tuples fragment a logical group only when the
group does not fit a page. This makes "wide scorer calls" a structural property
of the format instead of something the scan recovers with a scratch buffer (as
Approach A does).

Required shape and rules:

- logical group: `count <= target_width`, metadata arrays stored **once per
  group**, payload bytes possibly split across physical segment tuples;
- segment identity: logical group id + segment index/count, with contiguous
  segment visitation or explicit assembly checks;
- continuation segments are mostly payload bytes (no repeated metadata, no
  per-segment headers beyond what assembly needs);
- **assemble the full logical group, then score once** — never score/flush
  merely because a page or segment ended (a per-segment scorer call would
  re-introduce the small-flush regression);
- typed LE views on aligned metadata arrays (see the aligned-typed-view building
  block below) so Intel/Graviton read gammas/offsets/counts zero-copy;
- do not pad a final partial group up to `target_width` payloads unless a
  scorer fast-path requires it and it benchmarks as worthwhile;
- define the durable format-version gate and old-index compatibility contract;
  coordinate with Task 42 before landing the durable format change. Delete/repack
  lifecycle for spanning groups stays with Task 114.

### Phase 2b - Aligned LE typed-view building block (shared by A and B)

Independent of spanning, lay out the dense numeric arrays aligned + little-endian
so they can be read as `&[f32]`/`&[u16]`/`&[u32]` zero-copy on the LE targets
(Graviton 4, AWS Intel), with a `cfg!(target_endian="little")` + runtime
`is_aligned` guard and a `from_le_bytes` fallback for tooling/tests/non-LE. This
is a **component of both** the existing one-page format and the Approach B span
format — not an alternative to B. It removes per-element decode and the gather
copy; the spanning format consumes it directly (assemble → typed view → score).

### Phase 3 - Head-to-head benchmark gate

- Drive everything through `ecaz bench suite` with a committed `SuiteConfig`.
- Compare, per storage format and nprobe cell, for TurboQuant and RaBitQ:
  row, dense (current/per-block), dense+A (coalesced), dense+typed-per-block,
  dense+B (logical-group spanning), dense+B+typed. Keep Approach A as the
  already-working baseline/gate; B is the durable structural candidate.
- **Stage the scales; do not front-load 1M.** Run real **50k + 100k first**
  (local lane) as the gate. Escalate to **1M (AWS lane)** only if 50k/100k show
  promise — i.e. the TurboQuant dense regression is closed (dense ≥ row,
  width≥32 reached), the RaBitQ win is retained, and recall is unchanged. If
  50k/100k do not show promise, stop and record an iterate/abandon decision
  without paying for the 1M tier.
- 1M is the AWS lane reading the 1M corpus base; local hosts only stage
  10k/100k today and would need 50k staged.
- Use the registered `ec_ivf` default sweep `[8,16,24,32,48,64]`; do not subset
  without a manifest-stated reason.
- Report warm latency p50/p95/p99, recall@10 + NDCG@10, build time, index size,
  posting pages, candidates, scan counters, and SIMD flush-width histograms.

## Acceptance Criteria

1. Approach A (coalescing) implemented behind the `dense_posting_blocks` gate.
2. Approach B (logical-group page-spanning packing) is implemented behind the
   gate and benchmarked head-to-head against A. B is **required**, not
   closeable-by-argument: the operator directed (2026-06-17) that B not be closed
   as dominated. The A-vs-B decision must rest on measured evidence.
3. Recall and NDCG unchanged vs the legacy row path for every compared cell.
4. SIMD flush-width counters show dense TurboQuant reaching width≥32.
5. A benchmark packet reports the head-to-head matrix for TurboQuant and RaBitQ
   with latency, recall, build time, index size, pages, candidates, and
   flush-width histograms. Real 50k + 100k are required; 1M is required only if
   50k/100k show promise (and is a prerequisite for any default-promotion).
6. The packet explicitly recommends which approach to adopt (A, B, or neither),
   and an explicit promote / iterate / abandon decision for
   `dense_posting_blocks` as a default.

## Promotion Criteria (for the eventual default decision)

- Recall unchanged for the same query set and reloptions.
- TurboQuant dense p50 improves (or at least matches row) at the target
  high-recall cells with no p95/p99 regression that erases the win.
- RaBitQ dense retains its Task 111 latency win.
- Index-size and build-time deltas reported and justified.
- 1M evidence present before any default move — but 1M is run only after 50k/100k
  show promise, so a default decision can be reached at the smaller scales and
  1M confirms it rather than gating the early go/no-go.

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
