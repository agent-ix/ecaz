# Task 111e: IVF Coarse-Rerank Candidate Pipeline

Status: **implemented / iterate, not default-promoted** (2026-06-18; packets
`reviews/task-111e/001` through `006`; reviewer requested bounded
matched-recall and rerank-representation follow-up before final closeout).
Priority: P0 latency / recall-at-latency.

## Goal

Build and measure an IVF `coarse_rerank` path that uses page-local dense
RaBitQ-1 postings as the hot candidate generator, then reranks a bounded
survivor set with a configurable stronger representation and configurable
placement.

The objective is to keep the proven storage and scan-locality wins of dense
RaBitQ-1 while recovering high final recall through rerank, without making
large page-spanning posting payloads the hot scan format.

The product goal is flexibility in both dimensions:

- rerank representation: RaBitQ-2 / RaBitQ-4 / RaBitQ-8, TurboQuant variants,
  and additional formats through a common rerank scoring interface;
- rerank placement: index-side payloads, or table/heap-side payloads.

## Why

Task 111 and 111a showed that page-local dense RaBitQ-1 is the only dense
posting shape that clearly works as a hot scan payload:

- 100k, nprobe 32, RaBitQ-1 row: p50 about `14.5 ms`, index about
  `311 B/row`.
- 100k, nprobe 32, page-local dense RaBitQ-1: p50 about `11.6-12.3 ms`,
  index about `236 B/row`.
- Recall is unchanged by the layout change.

The same packet line also showed what not to pursue as the primary path:

- Dense larger-payload scans are sensitive to small SIMD batch widths unless
  copied/coalesced at scan time.
- Page-spanning dense groups were dominated in latency, storage, page reads,
  and assembly work once payloads exceeded the 1-bit case.
- True zero-copy page-scatter paths were correct but slower than copy fallback
  after locality improvements.

This points to a staged design: keep the hot posting list compact and
page-local, then spend stronger scoring only on a bounded survivor set.

## Scope

- IVF only.
- New explicitly gated storage/query mode, tentatively named
  `storage_format = 'coarse_rerank'`.
- Hot coarse stage starts with dense RaBitQ-1 postings.
- Rerank representation variants to measure:
  - f32,
  - RaBitQ-2,
  - RaBitQ-4,
  - RaBitQ-8,
  - TurboQuant variants,
  - other formats if they fit the common rerank representation interface
    without AM-specific special casing.
- Rerank placement variants to measure:
  - index-side payloads,
  - table/heap-side payloads.
- Candidate-frontier diagnostics before any default promotion.
- Works with the Task 111/111a accepted page-local dense layout and scan-side
  coalescing decisions.
- Coordinates with Task 112 for lazy heap-f32 rerank, Task 113 for safe bound
  pruning, and Task 115 for residual RaBitQ coarse scoring.

## Non-Goals

- Do not reintroduce page-spanning dense posting groups as the primary hot
  format.
- Do not make one rerank representation or placement a hard-coded design
  assumption.
- Do not change centroid training in this task.
- Do not require residual RaBitQ before proving the plain dense RaBitQ-1
  candidate frontier.
- Do not promote a default based only on final RaBitQ-1 no-rerank recall.
- Do not add recall-risky heuristic pruning.

## Phases

### Phase 1 - Candidate Frontier Measurement

Measure whether dense RaBitQ-1 contains the exact top-k inside a practical
candidate frontier.

Run oracle rerank measurements over the existing real 50k / 100k IVF fixtures:

- candidate_k: `25`, `50`, `100`, `256`, `512`, `1000`,
- nprobe sweep matching the Task 111a cells,
- final recall@10 and NDCG@10 after exact f32 oracle rerank,
- coarse scan latency,
- candidate materialization count,
- heap or sidecar bytes that a real rerank would need to touch.

Stop condition: if dense RaBitQ-1 needs thousands of candidates to recover the
target recall, do not implement a full pipeline until Phase 1b evaluates
RaBitQ-2 or residual RaBitQ as the coarse stage.

### Phase 1b - Coarse Stage Alternatives If Needed

Only if plain dense RaBitQ-1 containment is too weak:

- evaluate dense RaBitQ-2 as the coarse stage,
- evaluate residual RaBitQ-1 if Task 115 has enough scoring design landed,
- compare containment, storage, and scan latency against plain dense RaBitQ-1.

Do not proceed to a larger rerank implementation until the chosen coarse stage
has a credible candidate_k range.

### Phase 2 - Pipeline Contract and Reloptions

Define the durable user-visible mode and internal plan shape.

Tentative reloptions:

```text
storage_format = 'coarse_rerank'
coarse_format = 'rabitq'
coarse_bits = 1
rerank_placement = 'index' | 'table'
rerank_format = 'f32' | 'rabitq2' | 'rabitq4' | 'rabitq8' | 'turboquant' | ...
rerank_width = N
```

Clarify whether `rerank_width` is a hard fixed-width cap, a minimum target for
a lazy reranker, or a diagnostic fallback depending on the chosen placement.

Define a common rerank representation interface before adding format-specific
variants. `QuantCodec` is the first candidate seam, but it is not a foregone
conclusion: if rerank needs different access, metadata, or scoring semantics,
define a smaller rerank-specific interface and document why it is separate.
Avoid per-format AM special cases except behind that shared interface.

### Phase 3 - Heap-F32 Baseline Pipeline

Implement the narrowest end-to-end pipeline first:

- dense RaBitQ-1 coarse scan,
- bounded candidate frontier,
- table/heap-side f32 exact rerank,
- duplicate handling and snapshot semantics matching current IVF rerank,
- EXPLAIN counters for coarse candidates, reranked rows, skipped rows, heap
  rows, heap blocks, and elapsed time by stage.

This phase should reuse or align with Task 112 rather than duplicate lazy
rerank machinery.

### Phase 4 - Flexible Rerank Representations and Placement

Add quantized rerank formats and index-side placement only after the table/heap
f32 baseline proves the candidate frontier is useful.

Initial representation variants:

- RaBitQ-2 sidecar,
- RaBitQ-4 sidecar,
- RaBitQ-8 sidecar,
- TurboQuant sidecar if the existing codec/page APIs support a narrow
  implementation without broad format churn,
- any other format that can be scored through the common rerank interface.

Before implementing these variants, decide whether the common interface is:

- the existing `QuantCodec` surface,
- a thin adapter over `QuantCodec`,
- or a rerank-specific trait for payload access, query preparation, and
  candidate scoring.

For each representation, measure both feasible placements:

- index-side payload,
- table/heap-side payload.

Measure index size, table/storage overhead where applicable, build time, scan
latency, rerank latency, final recall, and read amplification for each variant.
Keep rerank payload placement separate from the hot coarse posting payload.

### Phase 5 - Bound and Lazy Integration

Integrate safe pruning and lazy stop only after the fixed-width pipeline is
correct and measurable.

- Use Task 113's bound contract where available.
- Use Task 112's lazy heap-f32 policy where available.
- Preserve an explicit fixed-width fallback for A/B measurements.
- Report whether each latency win comes from less coarse scoring, fewer
  retained candidates, fewer heap reads, or cheaper index-side rerank.

### Phase 6 - Benchmark Gate

Run matched measurements against the current accepted IVF surfaces.

Required cells:

- 50k and 100k local first,
- 1M AWS only after smaller cells show a credible Pareto point,
- nprobe and candidate_k sweeps,
- table/heap f32 and at least one compact quantized rerank variant,
- index-side and table/heap-side placement where both are feasible,
- final recall@10 and NDCG@10,
- p50/p95/p99 and mean,
- index size and build time,
- coarse scan, candidate retention, and rerank stage counters.

Promotion criteria:

- Final recall reaches the target band at materially lower latency than the
  current high-recall IVF path, or reaches equivalent latency with materially
  lower index storage.
- Candidate frontier width is stable enough to set a conservative default or
  documented preset.
- Tail latency does not regress from heap scatter or sidecar reads.
- Storage overhead is explicitly accounted for and justified by latency or
  recall-at-latency gains.

## Acceptance Criteria

1. Candidate-frontier containment evidence exists for dense RaBitQ-1.
2. The `coarse_rerank` contract and reloptions are documented.
3. Heap-f32 rerank baseline works end to end behind an explicit gate.
4. At least one compact quantized rerank representation is implemented or
   rejected with packet-local evidence.
5. Index-side and table/heap-side placement are either implemented for a
   representative format or explicitly separated with evidence and follow-up
   scope.
6. EXPLAIN and bench output expose coarse candidates, retained candidates,
   reranked rows, heap/sidecar reads, and per-stage elapsed time.
7. Bench evidence recommends promote, iterate, or abandon.

## Evidence Requirements

Benchmark packets must include:

- suite config,
- reloptions,
- corpus and query count,
- nlists, nprobe, candidate_k, and rerank_width,
- coarse format,
- rerank placement,
- rerank format,
- which common rerank interface was used,
- recall@10 and NDCG@10,
- p50/p95/p99 and mean,
- index size and build time,
- coarse candidate count,
- exact/oracle containment count,
- reranked row count,
- heap rows and heap blocks if heap rerank is used,
- sidecar/index bytes read if index-side rerank is used,
- per-stage elapsed time.

## Dependencies and Coordination

- Builds on Task 111 and Task 111a page-local dense block evidence.
- Coordinates with Task 112 for lazy heap-f32 rerank.
- Coordinates with Task 113 for bound-aware pruning.
- Coordinates with Task 115 if residual RaBitQ becomes necessary for candidate
  frontier quality.
- Coordinates with Task 42 for any durable format-version change.
