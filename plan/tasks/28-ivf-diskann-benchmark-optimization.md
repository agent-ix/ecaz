# Task 28: IVF and DiskANN Benchmark/Optimization Lane

Status: planned follow-up
Owner: coder1 / runtime-index track

## Goal

Establish real, reproducible build/recall/latency baselines for IVF and DiskANN
on the same corpus surfaces used by HNSW, then optimize the winning structure
enough to unblock frontier research above it.

This task supersedes further HNSW parallel-build tuning as the next scale
research priority. The PG18 HNSW parallel build work remains valuable: it
delivered a clear 50k build-time win and packet 669 showed that the worker
launch path holds on the 990k x 1536 DBPedia anchor. The reason to pivot is
narrower: the current in-Postgres HNSW graph construction path is not yet
competitive enough at 990k to justify more threshold tuning before we learn
what IVF and DiskANN can do. Offline HNSW bulk build stays on the follow-up
list, but it should not block IVF/DiskANN learning.

## Hardware Baseline

Use a dedicated Graviton-class benchmark host for product claims:

- 16+ vCPU minimum, 32+ preferred.
- 128 GiB RAM minimum for 1M x 1536; higher for larger curves.
- Fast local NVMe when available; otherwise provisioned high-throughput EBS.
- No WSL, shared dev box, background compile load, or competing Postgres
  clusters.
- Record instance type, kernel, compiler, Postgres version, extension SHA, PG
  settings, storage device, and cache state in every review packet.

Local WSL numbers remain useful only for smoke tests and bottleneck discovery.

## Phase 1: Comparable Baselines

- Run the canonical DBPedia 990k/10k anchor with fixed corpus manifests.
- Measure HNSW current state only as a reference row, not as the optimization
  target.
- Add external or reference rows where practical:
  - pgvector IVFFlat and HNSW on the same host.
  - pgvectorscale DiskANN or another PostgreSQL DiskANN reference if installable.
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
- Gate on a recall/latency/build Pareto point that beats the current HNSW
  baseline for at least one important workload shape.

## Phase 3: DiskANN Candidate

- Stand up the ADR-034 `ec_diskann` candidate or a faithful reference harness
  before touching AM integration.
- Measure Vamana build cost, graph size, disk-read pattern, page-cache behavior,
  and compressed-code scoring throughput.
- Keep the first implementation batch-oriented and benchmarkable; live insert
  and vacuum semantics can follow after the build/search shape is justified.
- Gate on a repeatable result showing that disk-resident traversal has a
  credible path beyond the HNSW memory-resident ceiling.

## Phase 4: Optimization

- Pick the stronger candidate based on measured recall/latency/build curves.
- Optimize the hot path using the existing SIMD and quantizer surfaces.
- Add narrow review packets for each optimization claim with packet-local logs.
- Only then reopen larger frontier research that depends on the selected
  structure.

## Deferred Follow-Ups

- Offline or staged checkpointed HNSW bulk build from task 26.
- GPU-accelerated offline build/training from ADR-046.
- SPANN-style routing from ADR-035, after DiskANN/IVF evidence is strong enough
  to justify the extra complexity.

## Acceptance Criteria

- At least one Graviton-class review packet with complete reproducibility
  metadata.
- A table comparing HNSW, IVF, and DiskANN candidates on the same corpus and
  metric definitions.
- A clear go/no-go recommendation for the next AM implementation target.
- No product decision based only on WSL or synthetic-only measurements.
