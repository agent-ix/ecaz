# Task 119: HNSW RaBitQ Coarse-Rerank Profile

Status: **proposed**.
Owner: coder (to be assigned). One coder, one branch.
Priority: P1, gated by Task 118 attribution.

## Why

Task 63 made RaBitQ a first-class HNSW storage format, but the measured HNSW
RaBitQ profile is not yet the coarse-rerank design we actually want to test.

The desired profile is:

1. use cheap 1-bit RaBitQ codes to generate or traverse a candidate frontier;
2. deliberately overfetch enough candidates to protect recall;
3. rerank the frontier with exact/source-f32 or another explicitly measured
   stronger representation;
4. recover a real storage win by avoiding unnecessary cold payload bloat.

Current HNSW RaBitQ evidence is not enough to reject that design, because HNSW
recall can fail before final rerank if approximate traversal never reaches the
right nodes.

## Dependency

Do not start implementation until Task 118 has identified the dominant recall
loss stage for HNSW RaBitQ.

Acceptable Task 118 outcomes that can unblock this task:

- RaBitQ traversal has good enough containment at a wider frontier;
- exact/source rerank fixes output when applied before truncation;
- graph build quality is acceptable and the remaining issue is frontier/rerank
  policy;
- a narrow scorer or rerank-boundary bug is fixed first.

If Task 118 shows RaBitQ cannot place truth rows in any practical frontier,
close this task as not justified.

## Goal

Evaluate and, if evidence supports it, implement a true HNSW RaBitQ
coarse-rerank profile with explicit overfetch and a measured second-stage
rerank representation.

The product question is whether HNSW can use RaBitQ as a compact candidate
generator while matching or approaching stronger rerank representations at
useful latency and lower storage.

This task is always about:

```text
RaBitQ 1-bit candidate frontier + second-stage rerank representation
```

The second-stage rerank representation matrix must include:

- exact/source f32 (`source_f32` or `heap_f32`, depending on the measurable
  implementation surface);
- RaBitQ 2-bit;
- RaBitQ 4-bit;
- RaBitQ 8-bit;
- every TurboQuant bit variant exposed by the implementation/CLI at the time
  of the run.

TurboQuant and multi-bit RaBitQ in this task are rerank representations over a
RaBitQ-1 frontier, not replacements for the RaBitQ-1 candidate generator.

## Scope

### Phase 1 - Profile Contract

Define the operator-visible and internal profile knobs without flattening HNSW
formats into a vague quantizer option.

Candidate knobs:

```text
storage_format = 'rabitq'
candidate_format = 'rabitq_1bit'
rerank_format = 'source_f32' | 'heap_f32' | 'rabitq_2bit' | 'rabitq_4bit' | 'rabitq_8bit' | 'turboquant_*' | ...
rerank_width = N
traversal_rescore_budget = N
```

`storage_format` remains the physical index layout. `candidate_format` is fixed
to the cheap RaBitQ-1 candidate frontier for this task. `rerank_format` and
`rerank_width` control the second-stage ranking policy.

### Phase 2 - Rerank Representation Baselines

Implement or expose the narrowest measurable baseline:

- RaBitQ-1 approximate traversal or candidate generation;
- explicit frontier overfetch;
- exact/source-f32 rerank over the retained frontier;
- RaBitQ 2/4/8 rerank over the retained frontier;
- all exposed TurboQuant bit-variant reranks over the retained frontier;
- deterministic top-k emission after exact rerank;
- counters for visited candidates, frontier candidates, reranked rows by
  representation, heap/source reads, and emitted rows.

This phase should prioritize measurement clarity over clever lazy policies.

### Phase 3 - Frontier and Traversal Sweep

Run matched sweeps for every required second-stage rerank representation over
the same RaBitQ-1 candidate frontier:

- `ef_search` beyond current default rows, including values such as `320`,
  `500`, and `1000` when the runner can express them;
- `rerank_width` sweeps for each rerank representation;
- optional `traversal_rescore_budget` if Task 118 shows approximate traversal is
  the limiting stage;
- same-corpus recall, latency, and storage.

PqFastScan and current TurboQuant HNSW lanes may still be reported as external
reference baselines, but they do not satisfy the required RaBitQ-1 + rerank
representation matrix.

The sweep must separate:

- traversal containment;
- exact-reranked frontier recall;
- final emitted recall;
- latency cost by stage.

### Phase 4 - Storage Layout Follow-Up

If exact/source rerank is the winning path, revisit the HNSW RaBitQ payload
layout so the profile earns a real storage win.

Measure:

- current RaBitQ HNSW index bytes per row;
- graph overhead vs code payload vs cold payload;
- storage after removing or shrinking cold scalar payloads where source rerank
  is configured;
- impact on build, insert, vacuum, and old-index compatibility.

Do not change durable layout without a format/version decision packet.

### Phase 5 - Product/API Cleanup

If the profile wins, normalize HNSW profile documentation and debug counters
across TurboQuant, PqFastScan, and RaBitQ:

- physical storage format;
- traversal scorer;
- final rerank representation;
- rerank width;
- traversal rescore budget;
- source/heap read behavior;
- format-specific unsupported combinations.

## Required Evidence

- Use `ecaz bench suite` for every benchmark matrix.
- Minimum matrix: 10k, 50k, 100k x RaBitQ-1 candidate frontier x
  `{f32, RaBitQ 2-bit, RaBitQ 4-bit, RaBitQ 8-bit, all TurboQuant bit variants}`
  x relevant `ef_search` and `rerank_width` values.
- The suite config must enumerate the exact TurboQuant bit variants available
  in the implementation at the time of the run. If a listed representation is
  not implemented, the packet must state that explicitly and either add support
  before measuring or mark the task blocked/incomplete.
- PqFastScan and ordinary HNSW TurboQuant/RaBitQ storage-format baselines are
  useful context but are not substitutes for the RaBitQ-1 + second-stage rerank
  matrix.
- Required metrics: recall@10, NDCG@10 where available, p50/p95/p99 latency,
  build time, index storage, visited/frontier/reranked/emitted candidate counts,
  and heap/source read counts.
- 1M is encouraged only after the smaller scales show a credible Pareto point.

## Non-Goals

- Do not replace TurboQuant or PqFastScan defaults in this task.
- Do not count TurboQuant or PqFastScan as alternative candidate generators for
  the core matrix; the candidate frontier under evaluation is RaBitQ 1-bit.
- Do not add broad cross-AM quantizer abstractions.
- Do not treat final rerank as sufficient unless Task 118 proves the true
  neighbors reach the rerank frontier.
- Do not promote RaBitQ HNSW unless storage improves or recall/latency improves
  enough to justify the format.

## Acceptance Criteria

1. Task 118 attribution is cited and the go/no-go condition is explicit.
2. A true RaBitQ-1 coarse-rerank profile is measured with explicit overfetch
   and the required second-stage rerank matrix: f32, RaBitQ 2/4/8, and all
   TurboQuant bit variants.
3. Recall, latency, storage, and candidate-stage counters are reported at
   10k/50k/100k for every required rerank representation, or the task is left
   open with a precise missing-representation/blocker list.
4. The final packet recommends promote, keep experimental, iterate, or shelve.
5. If any durable storage layout changes land, they include a format/version
   decision and lifecycle coverage for build, insert, vacuum, and scan.

## References

- `plan/tasks/118-hnsw-quantized-recall-attribution.md`
- `plan/tasks/63-hnsw-rabitq-storage-format.md`
- `plan/tasks/15-pqfastscan-first-class.md`
- `spec/adr/ADR-018-hnsw-quantized-graph-quality.md`
- `spec/adr/ADR-030-fastscan-grouped-subvector-scoring.md`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
