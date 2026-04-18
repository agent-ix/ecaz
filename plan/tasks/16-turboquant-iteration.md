# Task 16: TurboQuant Iteration with PqFastScan Learnings

Status: proposed — **depends on task 15 landing first**.

Follow-on to ADR-032.

## Scope

With PqFastScan merged as a first-class peer format (task 15), revisit the
TurboQuant format and port the three architectural wins that made
PqFastScan fast. Goal: narrow the TurboQuant vs PqFastScan latency gap on
the 50k warm real seam enough that TurboQuant remains a credible choice
for workloads that do not need PqFastScan's absolute throughput.

## Context

ADR-021 and ADR-025 established that TurboQuant scoring at 1536@4-bit is
bottlenecked by a 96 KB LUT that blows Graviton's 64 KB L1D. The scoring
kernel itself is well-optimized (AVX2/NEON) — the problem is the cache
working set.

The PqFastScan detour proved out three patterns that attack the same
bottleneck from different directions. All three already have code on
this branch — they are not new inventions, they are re-plumbings onto
the TurboQuant path.

## Levers (in expected-win order)

### Lever 1: Binary prefilter → TurboQuant rerank

Use the RaBitQ sign-bit sidecar from ADR-031
(`BinarySignNoQjl4BitQuery` in `src/quant/prod.rs`) as a cheap first
stage. Only candidates that survive the binary filter pay the TurboQuant
LUT cost. Mirrors the PqFastScan pipeline with the second-stage scorer
swapped in.

### Lever 2: Heap-f32 rerank

Port `GroupedRerankMode::HeapF32` (`src/am/scan.rs:462`) from PqFastScan
onto TurboQuant scans. With exact rerank coming from the heap, traversal
can drop to the 4+0 TurboQuant_mse fast path (no QJL), cutting LUT size
and per-edge payload. ADR-025 §Mitigation 3 sketched this but never
shipped it.

### Lever 3: Hot/cold payload split

Move QJL bits and gamma out of the hot graph tuple. Keep only
scoring-critical bytes inline per graph edge; park QJL and gamma on a
cold page read only for survivors. Mirrors the PqFastScan hot/cold
payload model. Likely requires an `INDEX_FORMAT_V3` wire bump rather
than an in-place change.

### Lever 4 (optional): Tiled LUT scoring

ADR-021 §6 open question. Process scoring in 512-dim tiles so each LUT
tile (~32 KB) fits in L1D alongside the `sq` and candidate slices.
More invasive than levers 1–3. Measure first whether levers 1–3 make
this unnecessary.

### Lever 5 (optional): int8 LUT

`Int8ApproxNoQjl4BitQuery` already exists in `src/quant/prod.rs`. Shrinks
the LUT 4x. Composes with tiling.

## Subtasks

- [ ] **Instrument TurboQuant scan path** for per-stage cost on the 50k
  warm real seam (traversal, LUT gather, QJL accumulate, rerank). Baseline
  numbers land in a measurement packet before any levers are wired.
- [ ] **Lever 1 — binary prefilter** ahead of TurboQuant scoring.
  Confirm the sidecar can be built either from existing code bytes or
  as a persisted payload (ADR-031 already enumerates both).
- [ ] **Lever 2 — heap-f32 rerank** mode for TurboQuant scans. Reuse the
  heap-fetch infrastructure added for PqFastScan (tuple slot,
  `grouped_heap_rerank_source_attnum` pattern).
- [ ] **Lever 3 — hot/cold payload split** for TurboQuant element
  tuples. Treat as `INDEX_FORMAT_V3` rather than in-place mutation of
  `INDEX_FORMAT_V1_SCALAR`.
- [ ] **Measurement packet** comparing TurboQuant-today vs
  TurboQuant-with-levers-1–3 at the same recall target on 50k warm real
  seam.
- [ ] **Decide** whether to pursue lever 4 (tiled LUT) and/or lever 5
  (int8 LUT) based on whether levers 1–3 close the gap.

## Owns

- Iteration track for ADR-006 (TurboQuant quantizer) latency.

## Dependencies

- **Task 15 must land first.** This task assumes PqFastScan is a stable
  peer format and reuses its hot/cold pagination and heap-rerank
  machinery. Starting before task 15 merges means building infrastructure
  twice.

## Unblocks

- A TurboQuant format that is fast enough to keep as the default for
  most workloads.
- A clean answer to "when should I pick TurboQuant over PqFastScan?".

## Out of scope

- Changing the TurboQuant quantizer itself (MSE codebook, QJL stage
  structure). Levers here are scoring-pipeline and payload-layout
  changes around the existing quantizer.
- OPQ transform front-end (PqFastScan-side follow-on, ADR-030).

## Notes

- Main tradeoff to watch: levers 1 and 2 reduce *how many* vectors get
  TurboQuant-scored. If Layer-0 traversal with a large frontier remains
  hot, per-candidate cost still dominates and levers 3–5 become
  load-bearing.
- Start with levers 1 + 2 — most of the wiring exists on this branch
  already for the PqFastScan path.
