# Task 50 Bench Baseline Plan

Task 50 packets need before/after evidence whenever a structural change touches
hot scoring, traversal, cache, or build paths. Capture the shared baseline once
before slice 1 so later packets can compare against a stable HEAD.

Baseline packet:

```text
benchmarks/task-50-baseline/
  manifest.md
  artifacts/
    unsafe-block-count-baseline.log
    ivf-rabitq-recall-baseline.log
    ivf-rabitq-latency-baseline.log
    spire-read-efficiency-baseline.log
    quant-rabitq-kernel-baseline.log
    hnsw-reference-recall-qps-baseline.log
    diskann-low-l-latency-baseline.log
```

## Required Baselines Before Code Slices

### IVF / RaBitQ

Purpose:

- primary prerequisite for IVF/RaBitQ optimization profiling;
- required for callback, page visitor, scorer, reloption, and vector-datum
  slices that touch IVF.

Capture:

- recall and QPS on the standard corpus profile;
- storage format explicitly recorded as `rabitq`;
- rerank mode and isolated/shared-table choice recorded.

Tolerance:

- use Task 31 M5 tolerance for IVF scan/QPS changes;
- any recall movement below the Task 47 floor blocks the packet.

### SPIRE Read Efficiency

Purpose:

- SPIRE is the product-differentiating target and should get baseline evidence
  before structural work starts.

Capture:

- Task 30 phase 13d read-efficiency lane;
- `ec_spire_remote_search_production_read_profile` output where available;
- candidate counts, heap session reuse, remote/local split, and final latency.

Tolerance:

- use the Task 30 phase 13d / M5 tolerance for read-efficiency changes;
- no new remote candidate loss, identity mismatch, or degraded-mode behavior.

### RaBitQ / Quant Kernel

Purpose:

- required before SIMD load/store newtypes or vector-datum wrappers touch
  RaBitQ scoring/build inputs.

Capture:

- quant scoring kernel benchmark or the closest existing criterion/IAI lane;
- include CPU target features in manifest.

Tolerance:

- no statistically meaningful instruction-count or latency regression beyond
  the kernel benchmark noise envelope.

### HNSW Reference

Purpose:

- needed only when a shared helper is rolled into HNSW or when top-15 density
  work touches HNSW hot scan/build paths.

Capture:

- standard HNSW recall + QPS lane;
- parallel build slot latency if touching DSM/build parallel state.

Tolerance:

- Task 33 M5 tolerance.

### DiskANN Low-L

Purpose:

- needed only when shared helpers roll into DiskANN or DiskANN-specific vector
  Datum / WAL / page work starts.

Capture:

- low-L latency curves from Task 32 / Task 29d lineage.

Tolerance:

- Task 32 M5 tolerance.

## Per-Packet Artifact Rule

Each implementation packet should store:

- `artifacts/block-count-before.log`;
- `artifacts/block-count-after.log`;
- relevant bench logs only for touched hot paths;
- `artifacts/manifest.md` with HEAD SHA, command, timestamp, lane, fixture,
  storage format, rerank mode, and isolated-vs-shared table choice.

For doc-only or callback-only packets that do not touch hot path behavior, the
request may explicitly say that runtime benches were skipped and cite this
baseline packet as the evidence source.
