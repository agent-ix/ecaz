# Task 79: SPIRE Candidate Surface Reduction

Status: active (2026-06-01)
Owner: coder (to be assigned). One coder, one branch.
Priority: 0 (direct successor to Tasks 75-78)

## Why

Tasks 75 through 78 identified the same root problem from several angles:
high-recall SPIRE scans far too many leaf candidates before retaining the tiny
set that can reach heap rerank.

The current Task 78 RaBitQ evidence is the baseline this task must beat:

- nprobe64 scores `10,420,357` candidates over 200 queries, retains `5,000`,
  returns `2,000`, and reaches recall@10 `0.9825` at p50 `41.757 ms`.
- nprobe96 scores `15,506,227` candidates over 200 queries, retains `5,000`,
  returns `2,000`, and reaches recall@10 `0.9975` at p50 `60.256 ms`.
- nprobe128 scores `20,000,000` candidates over 200 queries, retains `5,000`,
  returns `2,000`, and reaches recall@10 `1.0000` at p50 `74.951 ms`.

The nprobe96 point is the canonical high-recall target for this task. It scans
about `77,531` candidates per query to return 10 rows. Under RaBitQ, scoring is
still roughly `88%` of measured candidate-path CPU time, so another heap cutoff,
materialization tweak, or scorer-only micro-optimization does not directly
address the failure mode.

This task is successful only if it reduces the number of scored candidates at a
matched recall floor. Latency improvement without a candidate-surface reduction
is not a completion path for this task.

## Current Root Cause

The current 100k SPIRE high-recall fixture uses `nlists=128`,
`recursive_fanout=8`, top-graph routing, `boundary_replica_count=0`,
`rerank_width=25`, and session nprobe/top-graph search sizes of 64, 96, and
128.

That geometry makes the high-recall candidate surface too large:

- With 100k rows and 128 leaves, the mean leaf has about 781 rows.
- nprobe96 routes 96 leaves per query, or about 75% of the whole corpus.
- The Task 78 nprobe96 pipeline confirms `19,200` leaf routes and `15,506,227`
  candidates over 200 queries, about 807 candidates per selected leaf route.
- `max_candidate_rows` and `rerank_width` bound retained candidates only. They
  do not prevent the scan from reading and scoring every visible row in the
  selected leaves.
- The top graph is built over root routing-object children. With
  `recursive_fanout=8`, the root has only 8 children, so a top-graph
  search/route target of 64, 96, or 128 exhausts the root. It cannot by itself
  reduce the final leaf surface in the measured configuration.

The primary defect is therefore candidate selection geometry and routing budget,
not final candidate heap sizing.

## Paper Alignment

This task follows the SPIRE paper's central design guidance rather than treating
the current implementation shape as fixed. The paper, "Scalable Distributed
Vector Search via Accuracy Preserving Index Construction" (arXiv:2512.17264),
describes SPIRE around two decisions: choose a balanced partition granularity
that avoids read-cost explosion, and build a recursive hierarchy that preserves
accuracy with predictable search cost.

Task 78's evidence is a local read-cost explosion: the nprobe96 point reads and
scores about 75% of the 100k corpus to recover recall. Task 79 therefore treats
partition density and hierarchy route budgeting as the primary levers. The first
measurement packet must look for the inflection point where increasing leaf
density reduces vector reads without losing the high-recall floor.

## Non-Negotiable Gates

Use RaBitQ as the primary/default lane. TurboQuant is required only as a
comparison and regression guard.

The first accepted slice must report the current Task 78 RaBitQ nprobe96 row as
the baseline and meet all of these gates on the same 100k real-corpus / 200
query shape:

- recall@10 remains within `0.5 pp` of the Task 78 high-recall point
  (`0.9975`), or the packet explicitly shows an equal-or-better Pareto point at
  a different nprobe;
- scored candidates drop by at least 3x versus `15,506,227` candidates
  (`<=5.2M` over 200 queries);
- strong-pass candidate target is `<=4.0M` over 200 queries;
- stretch candidate target is `<=2.0M` over 200 queries;
- p50 latency improves by at least `25%` versus `60.256 ms`, or reaches
  `<=45 ms`, while preserving the recall gate;
- retained and returned counts remain comparable (`5,000` retained and
  `2,000` returned over 200 queries for rerank_width 25), unless the packet
  intentionally changes rerank width and proves the full recall/latency/candidate
  tradeoff.

Candidate count is the first gate. A slice that leaves `10.4M` / `15.5M` /
`20.0M` candidates unchanged must be shelved, even if p50 moves within noise.

## Required Measurement Packet

Before landing behavior changes, create a Task 79 packet under
`reviews/task-79/001-candidate-surface-baseline/` using `ecaz bench suite`.
The suite config must be checked into the packet and the artifact manifest must
record:

- head SHA and, when relevant, comparison SHA;
- storage format (`rabitq` primary, `turboquant` comparison);
- `nlists`, `recursive_fanout`, `top_graph_enabled`,
  `top_graph_search_list_size`, session nprobe, `nprobe_per_level`,
  `rerank_width`, `max_candidate_rows`, and `boundary_replica_count`;
- recall@10, p50/p95/p99 latency, selected leaf route count, scored candidate
  count, candidates/query, retained count, returned count, truncated count,
  object bytes, object-read timing, scoring timing, and score share;
- leaf density distribution for selected routes: mean, p50, p95, p99, max
  candidates per selected leaf route;
- top-graph/root routing diagnostics showing whether the root is exhausted and
  whether truncation happens at `beam_width`, `max_leaf_routes`, or neither.

Do not write a new shell sweeper. Extend `ecaz bench suite` only if the runner
is missing a necessary matrix dimension or report field.

## Phase 1 - Geometry-First Candidate Reduction

This is the required first solution attempt because it directly attacks the
observed row surface with the least scan-code risk.

Run a RaBitQ-primary matrix over leaf density and routing budget:

- `nlists`: 128 baseline, 256, 512, 1024, 2048;
- `recursive_fanout`: 8 baseline, 16, 32, and 64 where build cost and top-graph
  degree make sense;
- top-graph search/session nprobe: 32, 48, 64, 96, 128;
- `rerank_width=25`, `boundary_replica_count=0`, adaptive nprobe off for the
  first controlled matrix;
- TurboQuant comparison only for the best RaBitQ rows and the 128-leaf baseline.

Expected geometry math:

- 128 leaves at nprobe96 scans about 75% of the corpus, matching Task 78.
- 512 leaves at nprobe96 should reduce the raw row surface to roughly 25% of
  the corpus before distribution skew.
- 1024 leaves at nprobe96 should reduce the raw row surface to roughly 9-10% of
  the corpus before distribution skew.
- 2048 leaves at nprobe96 should reduce the raw row surface to roughly 5% of
  the corpus before distribution skew, but may need a larger nprobe to preserve
  recall.

The Phase 1 packet must select one of:

- a validated SPIRE high-recall recipe that meets the candidate and recall
  gates with only build/option changes;
- a narrow code task to improve routing budget/top-graph semantics because the
  geometry sweep exposes an otherwise-good row surface that current routing
  cannot use;
- a storage-format or subleaf design task because global leaf density alone
  cannot reach the candidate gate.

If Phase 1 passes with only a tuned recipe, do not silently change product
defaults. Land the evidence and split default-policy work separately.

## Phase 2 - Top-Graph and Per-Level Route Budget

If Phase 1 shows that leaf density can reduce candidates but current recursive
routing wastes the surface, land a narrow routing-budget slice.

Candidate implementation directions:

1. Add an explicit top-graph route budget that is separate from the leaf-level
   `scan_plan.nprobe`. The current top-graph path passes
   `top_graph_search_list_size.unwrap_or(scan_plan.nprobe)` and
   `scan_plan.nprobe`, which makes the top level too broad when root fanout is
   small or when leaf nprobe is high.
2. Make top-graph routing honor a documented `nprobe_per_level` interpretation,
   if that is the right existing API, rather than treating top-graph route count
   as the leaf nprobe.
3. Add diagnostics that report root child count, top-graph frontier size,
   selected root routes, internal parents expanded, final unique leaves, and
   per-query leaf row count.

Acceptance for this phase:

- candidate count falls versus the same `nlists`/fanout geometry without the
  route-budget change;
- recall remains within the high-recall gate;
- tests cover root-level top-graph budget behavior and recursive routing
  diagnostics in `src/am/ec_spire/scan/tests/routing.rs` or the closest existing
  scan test module;
- no SPIRE ownership/recursion semantic change outside the budget contract.

## Phase 3 - Row-Budgeted Routing

If fixed leaf counts remain too blunt because selected leaves have skewed row
counts, add candidate-row budgeting at routing time.

Design options:

1. Persist per-child `source_count` or estimated row count in routing children.
   The recursive build path already carries `source_count` while constructing
   the hierarchy, but `SpireRoutingPartitionObject` currently stores only child
   pids, centroid ordinals, and centroids. Persisting this value is likely an
   on-disk format change and needs an ADR or explicit format-version plan.
2. Prototype row-budget routing with existing leaf/object headers if it can be
   done without reading full leaf payloads or adding a hot object-read path that
   defeats the purpose.
3. Route leaves by approximate centroid quality until a per-query row budget is
   reached, with a minimum leaf count and a recall-preserving overflow rule.

Acceptance for this phase:

- report selected leaf count and selected row estimate separately;
- prove object/header reads do not consume the candidate-count win;
- show the row-budgeted path beats fixed nprobe at matched recall and matched
  `nlists`.

## Phase 4 - Leaf-Local Subpartitioning

Use this only if global leaf density and route budgeting cannot reach the gate.
The idea is to avoid scoring every row in a selected leaf by adding a second,
leaf-local pruning layer.

Candidate implementation directions:

- store compact subleaf centroids or block summaries in the leaf V2 object;
- select the best subleaf blocks within each routed leaf before row scoring;
- preserve a fallback path that scans the full selected leaf when summaries are
  missing or when recall diagnostics require it.

This is likely a format and build-cost task. Do not start it without a design
packet that compares it to simply increasing `nlists`.

## Phase 5 - Adaptive Candidate-Budgeted Nprobe

The existing `ec_spire.adaptive_nprobe` only halves requested nprobe when a
score-gap threshold passes, and it was disabled in the Task 78 evidence. It is
not enough as-is.

A real adaptive path may be useful after the static geometry and routing-budget
work:

- use a per-query candidate-row budget rather than a fixed half-nprobe step;
- keep a recall-preserving minimum route count;
- record why each query stopped: row budget, score margin, exhausted frontier,
  or configured maximum;
- compare against the static best RaBitQ recipe, not against the 128-leaf
  baseline only.

This phase must not be used to trade away recall invisibly.

## Non-Solutions

The following are not accepted completion paths for Task 79:

- another bounded-heap cutoff that leaves scored candidate counts unchanged;
- lowering `rerank_width` or `max_candidate_rows` without reducing scored rows;
- a RaBitQ or TurboQuant scoring-kernel speedup that makes each candidate
  cheaper but still scores `15.5M` candidates at the high-recall point;
- a product default flip to RaBitQ without a candidate-selection win;
- AWS-only exploration before a local candidate-count gate passes.

## Implementation Surfaces

Likely files and components:

- `src/am/ec_spire/scan/candidates.rs` for candidate collection and scored-row
  accounting;
- `src/am/ec_spire/scan/routing.rs` for top-graph, recursive routing, and
  diagnostics;
- `src/am/ec_spire/options/mod.rs` for scan-plan, `nprobe_per_level`, and route
  budget contracts;
- `src/am/ec_spire/build/recursive.rs` for hierarchy fanout and any persisted
  source-count design;
- `src/am/ec_spire/build/top_graph.rs` for root top-graph construction;
- `crates/ecaz-cli/src/commands/bench/suite.rs` only if the standard suite
  runner lacks the required matrix/report fields.

## Validation

For measurement-only packets:

- `ecaz bench suite audit`;
- `ecaz bench suite run`;
- `ecaz bench suite status`;
- `ecaz bench suite report`;
- packet-local artifact manifest with the required metrics above.

For code changes:

- focused Rust tests covering the touched routing/scan contract;
- PG18-focused validation for PostgreSQL callback behavior when relevant;
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`;
- no PG17 requirement unless the slice is explicitly PG17-facing.

Run AWS only after the local RaBitQ candidate-count and recall gates pass. AWS
evidence must record cloud status before and after the run and leave no running
SPIRE infrastructure afterward.

## Exit Criteria

- A Task 79 packet proves a RaBitQ-primary candidate-surface reduction at the
  100k high-recall point, with TurboQuant comparison rows.
- The accepted slice reduces scored candidates by at least 3x versus the Task
  78 RaBitQ nprobe96 baseline while preserving the recall gate.
- The packet reports whether the winning route is geometry-only, top-graph
  budget, row-budgeted routing, or a deferred subleaf/storage-format design.
- If no slice passes, closeout must state precisely which candidate surface
  remains too large and split the next task around that surface. Do not close
  this task as a generic latency optimization.

## Coordination

- Task 75 provides the corrected top-graph routing-funnel baseline.
- Task 76 keeps SPIRE defaults unchanged until a candidate-surface fix changes
  the Pareto curve.
- Task 77 proves materialization and heap maintenance are not the first-order
  problem.
- Task 78 proves RaBitQ is the correct primary/default storage-format direction
  for this lane, but also proves the current cutoff slice did not reduce
  candidates.
- Task 79 owns candidate-surface reduction directly. It must not be satisfied
  by scorer-only or default-policy work.
