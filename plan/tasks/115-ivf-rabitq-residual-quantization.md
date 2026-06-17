# Task 115: IVF RaBitQ Residual Quantization

Status: **proposed**.
Priority: P2 recall-per-probe after latency-first layout work.

## Goal

Improve IVF RaBitQ recall per probed list by encoding posting payloads as
residuals relative to their assigned IVF centroid, with the correction metadata
needed to score candidates against the original vector.

The expected product value is lower latency at a fixed recall target if the
same recall can be reached with fewer probes. This task is intentionally after
the latency-first page-layout and rerank work so benchmark attribution stays
clean.

## Why

Plain RaBitQ encodes each vector directly. IVF already assigns every posting to
a centroid. Encoding the residual from centroid to vector can reduce quantized
error within each list, which may improve recall at the same nprobe or allow a
lower nprobe at the same recall.

This is a scoring-semantics change, not a page-layout optimization. It should
therefore be measured separately from dense posting blocks.

## Scope

- IVF only.
- RaBitQ only.
- Build-time residual encoding against the assigned centroid.
- Insert-time residual encoding for new postings.
- Scan-time scoring with centroid correction.
- Minimal correction metadata only; no full-vector index sidecar.
- Reloption-gated or quantizer-mode-gated behavior.

## Non-Goals

- Do not change TurboQuant in this task.
- Do not change posting page layout.
- Do not change heap-f32 rerank behavior.
- Do not change centroid training.
- Do not change default RaBitQ behavior without benchmark-backed promotion.

## Phases

### Phase 1 - Scoring Design

- Derive the residual scoring formula for IVF inner-product search.
- Define the correction metadata needed per posting.
- Define how centroid score, residual score, and correction combine.
- Add scalar reference tests comparing residual scoring against exact source
  vectors on small fixtures.

Stop condition: if the correction metadata is large enough to undermine the
compact-index goal, stop and document the tradeoff.

### Phase 2 - Build and Insert Encoding

- Add residual RaBitQ encoding for build-time postings.
- Add residual RaBitQ encoding for insert-time postings.
- Keep plain RaBitQ indexes readable and buildable.
- Add metadata/version gating so scan can distinguish plain and residual
  payloads.

### Phase 3 - Scan Integration

- Add residual scoring to IVF scan.
- Preserve batch scoring where possible.
- Preserve heap-f32 rerank correctness.
- Add counters or diagnostics showing residual mode is active.

### Phase 4 - Recall-Per-Probe Sweep

Run recall sweeps comparing plain RaBitQ and residual RaBitQ:

- same corpus,
- same nlists,
- same q-set,
- same nprobe sweep,
- same rerank setting,
- index size and build time reported.

Promotion criteria:

- Residual RaBitQ improves recall materially at the same nprobe, or reaches
  the same recall at lower nprobe.
- Any latency win is shown at matched recall, not just matched nprobe.
- Index-size increase is small and explicitly reported.

### Phase 5 - Latency Confirmation

Only after recall-per-probe improves, run latency benchmarks at matched recall
targets to see whether fewer probes convert into lower p50/p95/p99.

## Acceptance Criteria

1. Residual RaBitQ scoring has scalar reference tests.
2. Build and insert can encode residual postings behind a gate.
3. Scan can read both plain and residual RaBitQ indexes.
4. Recall-per-probe benchmark evidence exists.
5. Latency-at-matched-recall benchmark evidence exists if recall improves.
6. The final packet recommends promote, iterate, or abandon.

## Evidence Requirements

Benchmark packets must include:

- suite config,
- reloptions and residual gate,
- q-count,
- nlists and nprobe sweep,
- recall@10 and NDCG@10,
- p50/p95/p99 and mean,
- index size and build time,
- posting/candidate counters where available,
- matched-recall comparison table.

## Dependencies and Coordination

- Should follow Task 111/112 if latency remains the top product priority.
- Coordinates with Task 113 because residual scoring may change bound behavior.
- Coordinates with Task 42 if a durable format-version change is required.
