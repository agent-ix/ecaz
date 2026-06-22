# Task 120: SPIRE Coarse-Rerank Measurement Program

Status: **proposed**.
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 for any renewed SPIRE algorithm work.

## Why

Tasks 79 through 85 established that SPIRE's product-scale problem is not a
generic lack of scoring throughput. The strongest retained AWS 1M/q500 surface
kept recall around `0.9832` with about `9.21M` candidates, and the attribution
work narrowed most misses to selected-leaf block or candidate-surface loss
rather than pure top-graph routing.

The IVF Task 111e-111h coarse-rerank work suggests a better pattern:

- use a cheap compact representation to generate candidates;
- keep the hot scan path storage-light;
- exact/source rerank only a bounded survivor set;
- measure containment before promoting any policy.

This task transfers that pattern to SPIRE, but only as a phased
measurement-first program. It must not assume coarse-rerank is a win just
because it worked in IVF.

## Goal

Determine where coarse-rerank belongs in SPIRE:

- inside selected leaves and row blocks;
- around route/leaf/block summary selection;
- on remote worker nodes before distributed merge;
- or nowhere, if containment or shipping costs do not support it.

The task should produce evidence-backed SPIRE decisions, not a new default by
intuition.

## Scope

This is a phased SPIRE research task. Each phase should produce a review packet
with its own result and go/no-go recommendation before the next phase changes
durable behavior.

### Phase 1 - Stage-by-Stage Containment and Budget Diagnostic

Build or extend SPIRE diagnostics so each query can report containment and
candidate budget at every stage:

- topology route set;
- selected leaves;
- selected leaf blocks or partition objects;
- local candidate frontier;
- exact/source rerank frontier;
- final emitted top-k;
- distributed worker stream, when applicable.

Report, per stage:

- candidate/object count;
- bytes read or shipped;
- exact truth top-k containment;
- score rank of truth rows when present;
- budget/cap that dropped the row when absent;
- latency by stage.

This phase should start from the Task 81-84 finding that selected-leaf block
choice and candidate-surface limits dominate the retained 1M recall gap.

### Phase 2 - Local Leaf Coarse-Rerank

Within already-selected SPIRE leaves, test cheap candidate generation followed
by source-f32 or exact rerank.

Candidate coarse representations:

- existing leaf/block summaries;
- 1-bit RaBitQ row or block codes;
- PQ/PqFastScan-style compact codes if they fit the leaf layout;
- centroid-residual or route-relative summaries;
- other compact summaries only if Phase 1 shows a specific miss pattern they
  can address.

Required measurements:

- containment before exact rerank;
- final recall after exact/source rerank;
- row payload reads avoided;
- summary bytes and sidecar bytes;
- latency split between summary scoring, candidate materialization, and rerank;
- storage delta.

### Phase 3 - Candidate Budget and Rerank Policy

Define explicit candidate budgets instead of relying on accidental cap
interactions.

Budgets to model:

- route overfetch;
- leaf fanout;
- block/object cap;
- per-node local frontier;
- final rerank width;
- coordinator merge width.

This phase should produce conservative defaults or presets only if curves show
a stable recall/latency tradeoff. Otherwise it should leave the knobs
diagnostic-only.

### Phase 4 - Topology Route-Set Refinement

Test coarse-rerank around topology routing without replacing topology routing.

Allowed experiments:

- overfetch topology routes;
- rerank route, leaf, or block summaries;
- use exact/source containment diagnostics to detect routing loss;
- reduce fanout only after containment remains safe.

Non-negotiable constraint: centroid/top-graph routing remains the primary route
generator. This phase is about conservative route-set refinement, not replacing
SPIRE topology.

### Phase 5 - Distributed Near-Data Rerank

For distributed SPIRE, evaluate the contract where workers do local candidate
generation and local exact/source rerank before returning compact exact-scored
streams to the coordinator.

Phase 5 MUST be validated on a local multi-node setup before any AWS run. This
local gate must be a **local multi-node distributed test**: one local
coordinator plus one or more local worker PostgreSQL nodes/databases, with
remote placements and remote dispatch enabled. The coordinator and worker may
run on the same physical machine, but they must be distinct SPIRE node
identities and the evidence must show that the worker path was used. A
single-node local scan, even on the same corpus and host, does not satisfy this
gate. The local multi-node test must exercise the same distributed
production-read path, static remote placements, worker/fanout controls, shipping
counters, and merge/dedupe counters. Start with a tiny local fixture, then scale
to the required local 10k/50k/100k matrices only after the production-read suite
path is known to complete and emit the required counters.

Do not run Phase 5 in AWS until the local multi-node gate passes and the user
explicitly approves the specific AWS run. Approval must be affirmative and tied
to the intended AWS matrix; a prior general task request is not approval to
provision or benchmark in AWS.

Required measurements:

- worker-local candidates generated;
- worker-local candidates exact-reranked;
- rows/bytes shipped to coordinator;
- coordinator merge/dedupe cost;
- end-to-end recall and latency;
- effect of worker count and route fanout.

The coordinator should merge exact-scored streams where possible, not fetch or
rerank a large unresolved candidate set after excessive shipping.

### Phase 6 - Maintenance, Staleness, and Fallback Invariants

If any coarse summaries or rerank sidecars become durable, define correctness
and fallback behavior for:

- insert;
- delete;
- vacuum;
- leaf split or movement;
- summary rebuild;
- mixed-version indexes;
- stale, missing, or malformed summaries;
- remote worker version skew.

Fallback must be conservative: stale or missing summaries may overfetch or use
exact/full-leaf behavior, but must not silently drop candidates.

## Required Evidence

- Use `ecaz bench suite` for every benchmark matrix.
- Minimum local matrix before product claims: 10k, 50k, and 100k.
- Distributed Phase 5 evidence must first run locally as a multi-node test
  against a coordinator plus at least one worker PostgreSQL node/database.
  Running those PostgreSQL nodes/databases on the same machine is acceptable;
  replacing the gate with a single-node scan is not. The packet manifest must
  identify the local coordinator node and local worker node(s), and cite
  evidence that remote placements/dispatch were active. Local distributed
  evidence must prove the production-read path completes and reports
  recall/latency, shipped rows/bytes, merge/dedupe counters, rowcap behavior,
  and worker/fanout effects before any AWS benchmark is considered.
- AWS is opt-in only. Do not provision, resume, or run AWS benchmarks for this
  task without explicit user approval for that specific AWS run.
- AWS 1M runs are required before any SPIRE product-default or product-claim
  decision, but only after the local distributed gate above passes and explicit
  AWS approval is given.
- Required metrics: recall@10, NDCG@10 where available, p50/p95/p99 latency,
  storage, build time, per-stage candidate counts, row payload reads,
  shipped bytes/rows for distributed paths, and exact truth containment.
- All durable artifacts must live under `reviews/task-120/` or an immutable
  `benchmarks/` packet cited by `reviews/task-120/`.

## Non-Goals

- Do not replace topology routing.
- Do not reopen local multi-disk SPIRE as a product surface; Task 107 dropped
  that surface.
- Do not make distributed SPIRE a product claim without distributed shipping
  evidence.
- Do not promote a summary or rerank format from final recall alone; candidate
  containment, payload reads, latency, and storage must all be measured.
- Do not write new one-off benchmark sweepers; extend `ecaz bench suite` if a
  required diagnostic step is missing.

## Acceptance Criteria

1. Phase 1 produces stage-by-stage containment and budget evidence that
   identifies where SPIRE loses or retains exact truth rows.
2. Phase 2 produces local leaf coarse-rerank A/B evidence against the current
   SPIRE surface at 10k/50k/100k.
3. Phase 3 records explicit budget policy curves and either recommends
   conservative defaults/presets or keeps the knobs diagnostic-only.
4. Phase 4 records whether route-set refinement safely reduces fanout, or
   rejects it with containment evidence.
5. Phase 5 records distributed near-data rerank shipping cost and merge behavior
   before any distributed SPIRE claim.
6. Phase 6 records maintenance and fallback invariants before any durable format
   or default is promoted.
7. The final packet recommends promote, iterate, or shelve each SPIRE
   coarse-rerank location separately: local leaf, topology refinement, and
   distributed near-data rerank.

## References

- `plan/tasks/75-spire-latency-routing-envelope.md`
- `plan/tasks/79-spire-candidate-surface-reduction.md`
- `plan/tasks/81-spire-leaf-block-summary-format.md`
- `plan/tasks/82-spire-1m-recall-attribution.md`
- `plan/tasks/83-spire-selected-block-containment-recovery.md`
- `plan/tasks/84-spire-1m-recall-recovery-without-candidate-inflation.md`
- `plan/tasks/85-spire-product-scale-pareto-program.md`
- `plan/tasks/107-spire-multidisk-multinode-value-prop.md`
- `plan/tasks/111e-ivf-coarse-rerank-candidate-pipeline.md`
- `spec/functional/spire/local/FR-053-spire-local-search.md`
- `spec/functional/spire/distributed/FR-058-spire-customscan-distributed-read.md`
- `spec/adr/ADR-074-spire-leaf-local-block-pruning.md`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
