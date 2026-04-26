# Task 29: DiskANN Initial Tuning Lane

Status: planned follow-up after task 28
Owner: coder1 / runtime-index track

## Goal

Establish initial, reproducible build/recall/latency baselines for DiskANN on
the same corpus surfaces used by HNSW and IVF, then identify the first concrete
optimization and implementation slices for the DiskANN path.

DiskANN is separate from task 28. IVF goes first; DiskANN remains a first-class
future work stream rather than being collapsed into the IVF tuning task.

## Hardware Baseline

Use the current local development environment for initial tuning and smoke
measurements. These numbers may guide implementation work, but they are not
publishable product claims.

Record enough metadata in every packet to make local measurements interpretable:

- machine / OS shape, CPU, memory, and storage location;
- compiler profile and extension SHA;
- PostgreSQL version and relevant settings;
- corpus manifest, row count, dimensionality, query count, and cache state.

Future product-claim benchmarks should move to a dedicated Graviton-class host.

## Phase 1: Reference Baselines

- Reuse the canonical DBPedia 990k/10k anchor and any smaller local smoke
  surfaces established by task 28.
- Measure HNSW current state only as a reference row.
- Add pgvectorscale DiskANN or another PostgreSQL DiskANN reference if
  installable without distorting the local benchmark environment.
- Capture build time, load time, index size, recall@10, p50/p95/p99 latency,
  cache state, memory high-water mark, and disk-read behavior where practical.

## Phase 2: DiskANN Candidate

- Stand up the ADR-034 `ec_diskann` candidate or a faithful reference harness
  before touching AM integration.
- Measure Vamana build cost, graph size, disk-read pattern, page-cache behavior,
  and compressed-code scoring throughput.
- Keep the first implementation batch-oriented and benchmarkable; live insert
  and vacuum semantics can follow after the build/search shape is justified.
- Gate on a repeatable result showing that disk-resident traversal has a
  credible path beyond the HNSW memory-resident ceiling.

## Phase 3: Initial Optimization

- Identify the strongest immediate DiskANN optimization slices based on measured
  recall/latency/build curves.
- Optimize the hot paths using the existing SIMD and quantizer surfaces.
- Add narrow review packets for each optimization claim with packet-local logs.
- Convert the measured findings into follow-up implementation tasks for DiskANN.

## Deferred Follow-Ups

- Production `ec_diskann` AM integration after the reference harness or early
  candidate justifies the build/search shape.
- Live insert and vacuum semantics for Vamana.
- GPU-accelerated offline build/training from ADR-046.
- SPANN-style routing from ADR-035.

## Acceptance Criteria

- Local initial-tuning review packets with complete reproducibility metadata.
- A table comparing HNSW and DiskANN candidates on the same corpus and metric
  definitions.
- A clear next-slice recommendation for DiskANN.
- A note separating local tuning results from future Graviton-class product
  benchmark requirements.
