# Task 62: HNSW Graviton Full Optimization Follow-Through

Status: **proposed**

Follow-on to Task 61. Task 61 established the first low-cost AWS Graviton
`ec_hnsw` lane, but the measured optimization used the default TurboQuant
storage format. This task completes the HNSW Graviton work by measuring both
supported HNSW storage formats, deciding whether the next wins are general HNSW
scan/build-path wins or PqFastScan-specific wins, and landing the justified
follow-up slices.

## Goal

Produce a complete Graviton HNSW optimization answer for `ec_hnsw` across
10k, 50k, and 100k rows, with 1M attempted only after the selected profile has
enough storage headroom. The task must clearly answer:

- whether generic HNSW graph traversal/build tuning is still the highest-ROI
  path;
- whether `storage_format = 'pq_fastscan'` changes the bottleneck enough to
  justify format-specific work;
- where HNSW sits against pgvector HNSW, pgvectorscale DiskANN, vchord RaBitQ,
  and the current ecaz IVF/RaBitQ rows; and
- what Graviton-specific work remains after portable HNSW tuning.

## Scope

- Access method: `ec_hnsw`.
- Hardware lane: AWS Graviton only for acceptance evidence.
- Required scales: DBpedia/OpenAI3 `ec_real_10k`, `ec_real_50k`,
  `ec_real_100k`.
- Required storage formats:
  - `storage_format = 'turboquant'`;
  - `storage_format = 'pq_fastscan'`.
- Required sweep: use the same `ef_search` values for both formats so rows are
  recall-comparable. Start with `40,64,100,128,160,200`; widen only if the
  recall/latency curve is under-sampled.
- Required runner: all benchmark matrices and retries must use checked-in
  `ecaz bench suite` configs.

## Non-Goals

- Do not add RaBitQ to HNSW in this task. That is a separate storage-format
  design task.
- Do not benchmark Intel in this task.
- Do not flip the HNSW default storage format.
- Do not make broad native/offline builder changes unless this task first
  produces evidence that build time dominates and a separate design task is
  accepted.

## Required Baseline Matrix

Create a benchmark packet under `benchmarks/task62-hnsw-graviton-full/` with
checked-in suite configs and packet-local logs.

For each of `10k`, `50k`, and `100k`, capture:

- load/build timing split;
- recall@10;
- latency mean/p50/p95/p99 at `concurrency = 1`;
- storage for heap, table, and index;
- EXPLAIN/cost rows;
- host precheck with CPU, memory, storage, PostgreSQL settings, extension SHA,
  and suite config hash.

The matrix must include both HNSW storage formats on isolated surfaces so the
planner cannot accidentally choose a sibling index:

| Scale | Format | Required |
| --- | --- | --- |
| 10k | TurboQuant | yes |
| 10k | PqFastScan | yes |
| 50k | TurboQuant | yes |
| 50k | PqFastScan | yes |
| 100k | TurboQuant | yes |
| 100k | PqFastScan | yes |

## Decision Gates

### Gate 1: General HNSW vs PqFastScan-Specific

After the baseline matrix, classify the dominant cost at matched recall.

Choose **general HNSW tuning** if both formats are dominated by shared work:

- graph tuple reads;
- neighbor fetch and prefetch behavior;
- candidate/frontier heap churn;
- duplicate expansion;
- allocation/reset overhead;
- build graph construction or page flush overhead.

Choose **PqFastScan-specific tuning** only if the PqFastScan rows are clearly
format-bound:

- grouped-PQ scoring;
- grouped codebook/LUT setup;
- hot/cold payload fetches;
- heap-f32 rerank;
- grouped traversal score mode;
- NEON grouped-score kernel behavior.

### Gate 2: Portable vs Graviton-Specific

Treat the first implementation slice as portable unless profiling shows the
winning path is aarch64-specific.

Portable HNSW work includes traversal, prefetch, allocation, layout, and build
pipeline improvements that should also help x86.

Graviton-specific work is limited to measured aarch64 concerns:

- NEON scoring kernels;
- aarch64 prefetch/cache-line behavior;
- CPU-specific dispatch or build flags.

## Candidate Optimization Slices

Promote at most one slice at a time, and only after packet-local evidence
identifies it as the active bottleneck.

1. **General scan-path tuning.** Reduce graph page reads, duplicate expansion,
   frontier churn, and allocation/reset overhead.
2. **PqFastScan HNSW scan tuning.** Optimize grouped-PQ scoring, rerank, and
   hot/cold payload access when the PqFastScan matrix proves the format is the
   limiter.
3. **Build-path tuning.** Improve graph construction, tuple packing, WAL/page
   flush, or training/setup only if build time dominates the acceptance rows.
4. **Host/profile tuning.** Change Graviton profile, memory settings, or EBS
   layout only when the evidence shows host shape is masking the implementation
   signal.

## Acceptance Criteria

- Baseline matrix completed for 10k, 50k, and 100k across TurboQuant and
  PqFastScan.
- At least one justified optimization/config slice landed, or a packet explains
  why no code/config slice is warranted.
- Post-change matrix rerun at the affected scales and formats.
- No recall regression outside documented tolerance at matched settings.
- Final packet compares HNSW rows against available comparator evidence:
  pgvector HNSW, pgvectorscale DiskANN, vchord RaBitQ, and ecaz IVF/RaBitQ.
- Final packet states whether the next HNSW work is general tuning,
  PqFastScan-specific tuning, Graviton-specific SIMD/cache work, build-path
  redesign, or no further HNSW work.

## Stop Conditions

- Stop before 1M if storage/build feasibility is not proven on the selected
  Graviton profile; record the exact blocker and required host shape.
- Stop before Graviton-specific code if portable HNSW work is still the
  measured bottleneck.
- Stop before PqFastScan-specific code if TurboQuant and PqFastScan share the
  same dominant bottleneck.
