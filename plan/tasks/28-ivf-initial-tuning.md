# Task 28: IVF Initial Tuning Lane

Status: planned follow-up
Owner: coder1 / runtime-index track

## Goal

Establish initial, reproducible build/recall/latency baselines for IVF on the
same corpus surfaces used by HNSW, then start the first tuning passes needed to
unblock follow-on research above the IVF path.

This task supersedes further HNSW parallel-build tuning as the next scale
research priority. The PG18 HNSW parallel build work remains valuable: it
delivered a clear 50k build-time win and packet 669 showed that the worker
launch path holds on the 990k x 1536 DBPedia anchor. The reason to pivot is
narrower: the current in-Postgres HNSW graph construction path is not yet
competitive enough at 990k to justify more threshold tuning before we learn
what IVF can do. Offline HNSW bulk build stays on the follow-up list, but it
should not block IVF learning.

IVF goes first. DiskANN remains a separate future work stream in task 29 rather
than sharing ownership with this task.

## Hardware Baseline

Use the current local development environment for initial tuning and smoke
measurements. These numbers may guide implementation work, but they are not
publishable product claims.

Record enough metadata in every packet to make local measurements interpretable:

- machine / OS shape, CPU, memory, and storage location;
- compiler profile and extension SHA;
- PostgreSQL version and relevant settings;
- corpus manifest, row count, dimensionality, query count, and cache state.

Future product-claim benchmarks should move to a dedicated Graviton-class host:

- 16+ vCPU minimum, 32+ preferred.
- 128 GiB RAM minimum for 1M x 1536; higher for larger curves.
- Fast local NVMe when available; otherwise provisioned high-throughput EBS.
- No WSL, shared dev box, background compile load, or competing Postgres
  clusters.

## Phase 1: Comparable Baselines

- Run the canonical DBPedia 990k/10k anchor with fixed corpus manifests.
- Measure HNSW current state only as a reference row, not as the optimization
  target.
- Add external or reference rows where practical:
  - pgvector IVFFlat and HNSW on the same host.
  - Qdrant or Milvus only if the run can be documented without changing the
    Postgres-centered acceptance criteria.
- Capture build time, load time, index size, recall@10, p50/p95/p99 latency,
  cache state, and memory high-water mark.

## Phase 2: IVF Candidate

- Prototype or adapt an IVF candidate that reuses the existing quantized scoring
  kernels rather than introducing a second scoring stack.
- Sweep centroid count, `nprobe`, posting-list layout, and rerank width.
- Optimize for sequential posting-list access, predictable memory use, and
  low-WAL bulk loading.
- Gate on a recall/latency/build Pareto point that improves on the current HNSW
  baseline for at least one important workload shape or clearly documents why
  IVF should remain narrower than expected.

## Phase 3: Initial Optimization

- Identify the strongest immediate IVF optimization slices based on measured
  recall/latency/build curves.
- Optimize the hot paths using the existing SIMD and quantizer surfaces.
- Add narrow review packets for each optimization claim with packet-local logs.
- Convert the measured findings into follow-up implementation tasks for IVF.

## Deferred Follow-Ups

- Offline or staged checkpointed HNSW bulk build from task 26.
- DiskANN initial tuning and access-method work from task 29.
- GPU-accelerated offline build/training from ADR-046.
- SPANN-style routing from ADR-035, after DiskANN/IVF evidence is strong enough
  to justify the extra complexity.

## Acceptance Criteria

- Local initial-tuning review packets with complete reproducibility metadata.
- A table comparing HNSW and IVF candidates on the same corpus and metric
  definitions.
- A clear next-slice recommendation for IVF.
- A note separating local tuning results from future Graviton-class product
  benchmark requirements.
