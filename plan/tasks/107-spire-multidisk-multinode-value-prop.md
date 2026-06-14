# Task 107: SPIRE Multi-Disk / Multi-Node Value-Proposition Benchmark

Status: proposed (2026-06-14, operator-confirmed after Task 106 AWS
closeout). Owner: coder to be assigned. One coder plus optional infra
operator support for AWS topology bring-up.
Priority: 1 for SPIRE product direction; blocks any keep/drop/product
decision about SPIRE's intended deployment model.

## Why

Task 106 exercised SPIRE only as a single-node, single-local-store benchmark
surface. That is useful regression coverage, but it does not evaluate the
reason SPIRE exists: scaling ANN search through multi-disk and multi-node
placement.

This task must prove or disprove that SPIRE has a product-relevant Pareto point
in the deployment shapes it is meant to serve. A single-node result is not
enough. The evidence must separate:

- local multi-disk / multi-store parallelism on one host;
- distributed multi-node query fanout and merge behavior;
- the operational cost and complexity required to get any win.

## Scope

### Phase 0 - Topology Design And Readiness Audit

1. Read the current SPIRE production/AWS materials before running anything:
   `task30-phase13*`, `reviews/task-30/`, `benchmarks/aws-spire-*`, and the
   current SPIRE operator scripts.
2. Define the exact benchmark packet under `reviews/task-107/001-.../` with
   checked-in `ecaz bench suite` configs. Do not use ad hoc shell sweepers.
3. Confirm that the runner can express every needed step. If not, extend
   `ecaz bench suite` first and land that as a separate prerequisite commit.
4. Record topology metadata fields that every artifact must carry:
   node count, coordinator/worker roles, local store count, disk/volume count,
   mounted paths, corpus placement, routing/fanout settings, query concurrency,
   network placement, instance types, EBS/NVMe characteristics, and software
   SHA.

### Phase 1 - Single-Node Multi-Disk / Multi-Store Baseline

Measure one physical host with multiple independent data stores so the effect
of local disk/store parallelism is isolated from network fanout.

Required cells:

- SPIRE with `local_store_count=1` as the same-host control.
- SPIRE with at least two higher store-count settings, for example 2 and 4,
  using separate mounted volumes or devices where the implementation supports
  it.
- Same-host non-SPIRE comparators on the same corpus and hardware: HNSW and
  IVF/RaBitQ at the relevant quality points.

Required scales:

- 100k as a topology smoke/control.
- 1m as the minimum product-scale decision point.
- 10k/50k are optional smoke only; they must not drive the conclusion.

Required metrics:

- recall / NDCG;
- latency p50/p95/p99 and mean;
- build time;
- storage size by relation/index/store;
- candidate counts, routing/fanout counters, and heap rerank counts;
- disk read throughput / IOPS where available;
- CPU and memory peak.

### Phase 2 - Multi-Node SPIRE Benchmark

Measure a real distributed SPIRE topology. The minimum acceptable topology is
one coordinator plus at least two worker/data nodes. Prefer a three-worker
shape if the existing infra supports it within budget.

Required cells:

- SPIRE distributed read path at 100k and 1m.
- Matched single-node SPIRE control on comparable total storage/corpus.
- Same-corpus HNSW and IVF/RaBitQ controls on one production host.

Required evidence:

- end-to-end recall and latency;
- per-node and coordinator timing split;
- remote fanout count;
- bytes/rows returned per worker;
- merge/rerank cost;
- connection/session reuse behavior;
- node placement map and corpus/store distribution;
- failure or timeout behavior if a worker is unavailable, if this can be
  exercised safely without invalidating the main run.

### Phase 3 - Product Decision Analysis

Publish a decision packet that answers:

1. Does local multi-disk/store SPIRE beat single-store SPIRE at comparable
   recall and hardware cost?
2. Does multi-node SPIRE beat the best same-budget single-node comparator at
   1m?
3. Is any win large enough to justify the implementation, operations, and
   support burden?
4. If not, what is the narrowest SPIRE surface worth retaining, if any?

The answer can be "no". A negative result is acceptable if the evidence is
complete and the task closes the product question honestly.

## Out Of Scope

- New SPIRE algorithm work before the benchmark proves where the bottleneck is.
- New quantizers or kernel families.
- A broad all-AM release sweep.
- Drawing product conclusions from single-node or single-store data alone.

## Acceptance Criteria

1. A packet under `reviews/task-107/` contains the topology design, checked-in
   suite configs, manifest, raw artifacts, and result summaries.
2. Phase 1 multi-disk/store results complete at 100k and 1m, with a
   single-store SPIRE control and HNSW/IVF controls on the same host class.
3. Phase 2 multi-node results complete at 100k and 1m, with topology metadata
   sufficient to reproduce node roles, store placement, fanout, and hardware.
4. Every benchmark matrix is driven by `ecaz bench suite`; any missing runner
   capability is added to `ecaz-cli` before use.
5. Artifacts include build, storage, recall, latency, routing/fanout, disk, CPU,
   and memory evidence. Missing observability must be explicitly called out
   and either added or treated as a blocker.
6. The final packet makes a keep/drop/narrow decision recommendation for SPIRE's
   multi-disk and multi-node product surface, grounded in the measured Pareto
   data and cost/complexity.
7. AWS resources are stopped or destroyed after artifact sync, with state
   recorded in the manifest.

## References

- Task 30 Phase 13 AWS verification materials.
- Task 73 through Task 85 SPIRE quality, latency, and product-scale packets.
- Task 106 AWS targeted benchmark packet for the single-store limitation that
  motivated this task.
- `spec/non-functional/NFR-007-benchmark-provenance.md`.
- `crates/ecaz-cli/src/commands/bench/suite.rs`.

## Estimated Size

Medium-large. Expect one design packet, one local/multi-store AWS packet, one
distributed AWS packet, and one decision packet. Runtime cost is dominated by
1m fixture/index preparation and distributed topology setup.
