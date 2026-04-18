# Review Request: C1 Task16 TurboQuant Live Rerank

Current head at execution: `a94d98d`

## Context

Task 16's baseline packet (`423`) showed that current `50k, m=16, ef=128`
turboquant already has two behaviors the task text described as future levers:

- the generic ADR-031 binary prefilter is already active on turboquant
- the serious `1536x4` turboquant lane already uses the no-QJL / no-LUT
  `mse_no_qjl_4bit` exact scorer

What remained expensive on current head was the surviving scalar exact-score
work after the binary prefilter. The baseline packet recorded roughly:

- `~1604` binary-prefilter survivors/query
- `~1605` turboquant exact-score calls/query

So the real missing turboquant work was not more binary sidecar plumbing. It
was deferring turboquant exact comparison out of neighbor traversal and into the
existing live-rerank window that `pq_fastscan` already used.

## What Landed

This packet wires the existing live-rerank machinery into turboquant scans when
the binary query path is available.

### Scan-path changes

- Turboquant scans now enable the live-rerank buffer when the prepared binary
  query exists.
- During binary-prefilter traversal, turboquant candidates keep their binary
  approximate score in the frontier instead of exact-scoring every survivor
  immediately.
- Buffered turboquant result candidates now run their comparison pass at output
  time:
  - quantized rerank for source-less indexes
  - heap-f32 rerank for `build_source_column` indexes
- Source-backed turboquant scans now actually honor the existing heap-f32 rerank
  mode resolution; the scan-state configurator no longer force-downgrades every
  non-`pq_fastscan` index back to quantized rerank.

### Debug/profile changes

- `debug_turboquant_scan_stage_profile(...)` now reports turboquant rerank work
  from the shared quantized/heap rerank counters instead of hard-coding rerank
  to zero.
- Its traversal residual now subtracts rerank time as well as binary-prefilter
  and traversal-score buckets.

## Validation

Green on this head:

- `cargo test`
- `bash scripts/run_pgrx_pg17_test.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Additional focused coverage added in this packet:

- turboquant quantized rerank profile reports quantized-only deferred comparison
- turboquant source-backed heap rerank profile reports heap-only deferred
  comparison
- turboquant stage-profile SQL surface now expects rerank work and fewer
  traversal exact-score calls than binary-prefilter survivors

## Readout

### 1. This is the real turboquant follow-on from packet `423`

The packet does not try to re-port already-landed ADR-031 behavior. It ports the
missing runtime shape:

- binary score for traversal ordering
- deferred quantized or heap comparison in the live-rerank window

That is the part baseline `423` showed was still absent on turboquant.

### 2. Heap-f32 rerank is now genuinely available on source-backed turboquant

Before this packet, turboquant source-backed indexes resolved to `HeapF32` in
the mode decision logic but the scan-state configurator discarded that decision
for all non-`pq_fastscan` storage. This packet removes that dead-end.

### 3. Measurement is intentionally a separate next packet

This review request is the AM-logic packet only. The next task-16 packet should
capture the before/after `50k` measurement against the same isolated warm seam
used in `423`, now that turboquant has a real deferred-rerank path to measure.
