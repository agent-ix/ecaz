# Task 80: SPIRE Row-Budgeted Routing and Subleaf Latency

Status: active (2026-06-04)
Owner: coder (to be assigned). One coder, one branch.
Priority: 0 (direct successor to Task 79 and the AWS 1M top-graph recall packet)

## Why

Tasks 73 through 79 established that SPIRE can recover high recall, but the
current high-recall path does it by scoring too many row candidates.

The strongest local 100k evidence from Task 79 showed that fixed leaf-count
routing remains too coarse:

- Task 78 nprobe96 baseline: `15,506,227` scored candidates over 200 queries,
  recall@10 `0.9975`, p50 `60.256 ms`.
- Task 79 route-time row budgeting got close to the candidate gate but still
  missed it: `5,231,408` candidates at recall@10 `0.9940`, p50 `58.153 ms`.
- Task 79 concluded that the remaining granularity problem is inside selected
  leaves: once a leaf is selected, SPIRE still scores every visible row in that
  leaf.

The AWS 1M top-graph recall packet confirmed the same shape at scale:

- Rebuilding with `top_graph_search_list_size=256` recovered recall:
  - nprobe64: recall@10 `0.9976`, p50 `554.168 ms`,
    `251,510,240` candidates over 500 queries.
  - nprobe96: recall@10 `0.9994`, p50 `779.315 ms`,
    `373,897,385` candidates.
  - nprobe128: recall@10 `1.0000`, p50 `1038.917 ms`,
    `495,000,000` candidates.
- This proves top-graph breadth is a recall ceiling lever, not a latency path.

Task 80 therefore focuses on the two candidate-surface levers that can reduce
latency without simply lowering recall:

1. row-budgeted routing, so SPIRE chooses leaves against a row/candidate budget
   instead of a fixed leaf count;
2. leaf-local subleaf or block pruning, so a selected leaf does not require
   scoring every row in that leaf.

The broader latency roadmap is recorded in
`spec/adr/ADR-075-spire-latency-roadmap.md`.

## Scope

### Track A - Row-Budgeted Routing

Make row/candidate budget a first-class scan-planning concept for SPIRE.

The implementation should:

- select routes until an estimated per-query row budget is reached, not merely
  until `nprobe` leaves have been selected;
- preserve a recall-protecting minimum route count and a deterministic tie
  order;
- expose diagnostics for requested row budget, estimated selected rows, actual
  selected leaves, actual scored candidates, overflow/underflow, and stop
  reason;
- keep fixed-nprobe behavior available as a baseline and fallback;
- avoid hot-path reads of full leaf payloads just to discover row counts.

Acceptable row-count sources, in priority order:

1. already persisted per-child or per-leaf counts if available in routing or
   leaf metadata;
2. compact routing-object metadata added with an explicit format/version plan;
3. a prototype-only estimate derived from existing diagnostics, used only to
   prove the shape before any persistent contract is accepted.

If Track A needs an on-disk change, add or update an ADR before landing the
format change.

### Track B - Leaf-Local Subleaf or Block Pruning

If Track A cannot hit the candidate and latency gates at matched recall, move
to query-aware pruning inside selected leaves.

Track B follows ADR-074 and should:

- store compact, scoreable summaries for deterministic row blocks inside each
  leaf;
- score block summaries before reading or scoring row payload blocks;
- select row blocks per query using an explicit block or row budget;
- preserve full-leaf fallback for old formats, disabled GUCs, diagnostics, and
  malformed summary metadata;
- expose selected block counts, skipped block counts, summary-score time,
  row-score time, summary bytes, and row bytes.

This track is likely a SPIRE leaf format bump. Do not land it as an implicit
storage change hidden inside a scan optimization.

## Non-Goals

- Do not pursue wider top-graph search as the primary latency solution. The AWS
  1M packet shows it recovers recall by massively increasing candidates.
- Do not optimize candidate scoring kernels until the candidate surface is
  materially smaller.
- Do not change SPIRE recursion semantics outside an explicit routing-budget
  contract.
- Do not flip product defaults or introduce a quality preset in this task. That
  belongs after a measured candidate-surface win.
- Do not write ad hoc benchmark sweep scripts. Use `ecaz bench suite` and
  extend the runner if a needed field is missing.

## Required Evidence

Every measurement packet must be packet-local under `reviews/task-80/` and use
`ecaz bench suite`.

The first packet must establish a clean baseline against the accepted Task 79
rows and the AWS 1M top-graph recall packet:

- 100k real-corpus / 200-query RaBitQ lane:
  - recall@10;
  - p50/p95/p99 latency;
  - selected leaf routes;
  - estimated selected rows;
  - actual scored candidates;
  - retained and heap-reranked candidates;
  - returned rows;
  - object bytes and object-read timing;
  - scoring timing and score share.
- AWS 1M follow-up only after a local row clears the matched-recall candidate
  and latency gates.

For Track A, the packet must compare:

- fixed nprobe baseline;
- row-budgeted routing at multiple row budgets;
- at least one high-recall control that intentionally spends more rows.

For Track B, the packet must additionally report:

- available block summaries;
- selected and skipped row blocks;
- summary-score time;
- row-score time;
- summary and row object bytes;
- full-leaf fallback parity.

## Gates

Use RaBitQ as the primary lane. TurboQuant is comparison-only after the RaBitQ
row is close enough to defend.

### Local 100k Gate

Against the Task 78/79 100k real-corpus / 200-query high-recall shape:

- recall@10 must be at or above `0.9925`, or the packet must prove a clearly
  better Pareto point;
- scored candidates must be `<=5.2M`, with `<=4.0M` as the strong target;
- p50 must be `<=45 ms` or at least 25% better than the `60.256 ms` Task 78
  baseline;
- retained and returned counts must stay comparable to the `rerank_width=25`
  baseline unless the packet explicitly changes rerank width and proves the
  tradeoff.

### AWS 1M Gate

Run AWS 1M only after the local gate has a credible candidate.

The AWS packet must compare against:

- old tg96 1M row: recall@10 `0.9832`, p50 `268.824 ms`,
  `9,213,846` candidates over 500 queries;
- tg256 recall-ceiling rows from
  `benchmarks/aws-spire-1m-topgraph-rebuild/001-run/`.

An accepted AWS 1M row should materially improve recall over the old tg96 row
without moving into the hundreds-of-millions candidate surface exposed by the
tg256 recall-ceiling run.

## Implementation Order

1. Add or tighten diagnostics needed to prove row-budget behavior.
2. Prototype Track A locally with no durable format change if possible.
3. If Track A passes the local gate, package the behavior and run AWS 1M.
4. If Track A cannot pass because selected leaves remain too coarse, write the
   Track B design packet and implement ADR-074-style block summaries.
5. Run AWS 1M only after Track B passes locally.
6. Close with a decision:
   - landed row-budgeted routing;
   - landed subleaf/block pruning;
   - shelved with evidence and split a deeper format/build task;
   - or filed a default-policy task after a measured win.

## Exit Criteria

- One of Track A or Track B lands with packet-backed candidate reduction and
  matched recall.
- If neither lands, the closeout explains which gate failed and which deeper
  format or build task owns the next attempt.
- All durable benchmark evidence is under `reviews/task-80/` with an
  `artifacts/manifest.md`.
- The AWS 1M packet is captured if and only if the local gate has a credible
  row.
- No new unsafe blocks.
- PG18-focused validation is recorded for any code slice that changes scan or
  storage behavior.
- Closeout updates this task status and cites the accepted packet.

## Coordination

- Task 79 remains the accepted evidence that candidate surface, not final heap
  retention, is the latency bottleneck.
- ADR-074 owns the leaf-local block pruning storage direction.
- ADR-075 owns the broader SPIRE latency roadmap and priority ordering.
- Task 30 phases own SPIRE recursion correctness; any semantic recursion change
  must coordinate there before landing.
