# Task 227: ec_distann Recall Residual and Adaptive Search

Status: **plan refined; packet 001 review-open** (2026-08-24). Priority:
P1 recall.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidates
GRAPH-10, GRAPH-17, GRAPH-18, and TRAV-16 through TRAV-19.

## Entry disposition

Task 226 satisfies the entry gate. On the current fixed-4,096 sharded head,
BW8/H100 is a useful non-default configuration: the registered rule passes at
10k, 50k, and 100k, but Task 219's review-closed recall-equivalence policy
retains BW4/H100/L32 as the shipped interactive default. Task 227 therefore
uses BW4 as its production control and BW8 as a measured diagnostic/escalation
ceiling. It does not stack an adaptive policy on BW8 or reopen the default.

Task 188's accepted closeout is also binding: head-vs-owner-oracle and isolated
BW/H were measured, but candidate-frontier/exact-rerank containment,
components/indegree/bridge/hard-query reachability, and
monolithic-versus-sharded graph quality were explicitly unrun, not refuted.

## Goal

Classify the remaining current-head recall misses at query and truth-neighbor
granularity. Only if a truth-free runtime signal reliably identifies queries
that benefit from more search, screen one bounded conditional BW8 replay. Do
not select a graph rebuild and an adaptive runtime policy in the same A/B.

## Frozen measurement contract

1. Use one fresh 100k three-owner physical PG18 release generation with the
   current conforming fixed-4,096 persisted sharded head, L32, H100, RaBitQ,
   lazy-10 materialization, and current pushdown/gateway behavior. All runtime
   diagnostic variants share that immutable generation, and all evidence is
   driven by checked-in `ecaz bench suite` configuration.
2. The source query file has 1,000 rows and SHA-256
   `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
   Rows 201--400 are the diagnostic/calibration slice (slice SHA-256
   `a12a81111d586e78165a950962aa8667e2f95e700159fe86d83bba2b49a5ece9`);
   rows 1--200 are the blind evaluation slice (slice SHA-256
   `966fcdfb55bd8c36b05ca308e871407237e9bc22ad4355d42dd51db0b54e42c3`).
   Truth labels from rows 1--200 cannot tune the rule or its threshold.
3. A monolithic same-corpus control may be built in the same fixture for graph
   diagnosis. It is a distinct build, not a same-generation latency A/B; its
   build identity, parameters, bytes, and adjacency digest must be reported.
4. Instrumented query traces and clean production latency are separate suite
   steps on separate fresh fixtures. Stage counters or trace capture never
   supply the candidate's production latency row.
5. No corpus/query/truth files or cluster directories enter git. Manifests
   record the parent corpus/query prefixes, hashes, exact row slices, commands,
   execution SHA, and cleanup.

## P0 — Diagnostic tooling checkpoint

The current tree has owner-scan seed variants and aggregate stage counters,
but no ec_distann per-query containment trace or distributed persisted-graph
summary. Land those read-only surfaces before collecting attribution:

1. Add a benchmark-feature-gated ec_distann query trace that records, per
   query and variant, seed locators, requested/returned/expanded locators by
   round, retained approximate candidates in rank order, exact-rerank input,
   final returned ids, round count, heap saturation, frontier stability, score
   gaps, owner fanout, and request/response work. It records stable identities,
   not vector payloads, and is inert when the benchmark feature is disabled.
2. Add a read-only distributed graph diagnostic that reports, per owner and in
   aggregate, live nodes, directed/weak components, reachability from each
   registered seed set, zero/min/percentile/max in/out degree, local/remote and
   stitch edge counts, invalid/duplicate/self edges, bridge/articulation
   candidates, and adjacency digests. Report the same shape for the monolithic
   control where fields are meaningful.
3. Join traces to exact truth in `ecaz-cli`, not inside the production AM. Emit
   packet-local JSONL with explicit query id, query slice/hash, generation
   identity, variant, truth rank/id, stage membership, and classification.
4. Extend `ecaz bench suite` and the existing distann multinode runner with
   explicit diagnostic/evaluation query slices and expected artifact paths.
   Do not add a packet-local sweeper or one-off SQL script.
5. Focused tests must prove trace reset/isolation, bounded capture, disabled
   fast-path equivalence, deterministic graph digests, suite expansion, and
   unknown/missing-stage handling. PG18 callback coverage is required because
   the trace spans scan/traversal/rerank state.

## P1 — Query-level residual attribution

Run these diagnostic variants on the same physical generation and diagnostic
slice; only the named field changes:

| Variant | Seeds | BW/H | Neighbor score | Purpose |
| --- | --- | --- | --- | --- |
| `prod-bw4-rabitq` | persisted head | 4/100 | RaBitQ | shipped control |
| `task226-bw8-rabitq` | persisted head | 8/100 | RaBitQ | measured wider-search ceiling |
| `prod-bw4-exact-neighbor` | persisted head | 4/100 | exact | same-seed ordering diagnostic only |
| `owner-bw4-rabitq` | owner scan | 4/100 | RaBitQ | seed/oracle residual |
| `owner-bw4-exact-neighbor` | owner scan | 4/100 | exact | seed-independent ordering diagnostic |

The unchanged full exact-neighbor arm is not a candidate and cannot reopen
Task 189 by aggregate recall alone. It is retained only to classify individual
frontier comparisons under identical seeds.

Classify every missed truth neighbor in this priority order, so categories are
mutually exclusive and totals reconcile:

1. `generation_missing`: absent/tombstoned or identity cannot be mapped.
2. `seed_reachability`: unreachable from the production seed set but reachable
   from an owner-oracle seed set.
3. `graph_unreachable`: unreachable from both registered seed sets in the
   persisted directed graph.
4. `budget_frontier`: reachable, but never requested/expanded before H100.
5. `approximate_ordering`: reached/scored but excluded by RaBitQ where the
   same-seed exact-neighbor trace retains it.
6. `rerank_containment`: retained by approximate traversal but absent from the
   exact-rerank input.
7. `exact_competition`: present in exact-rerank input but outside final top-k.
8. `unknown`: missing/inconsistent evidence; report each unknown and fail the
   aggregate reconciliation rather than inferring a category.

After the diagnostic-slice report is committed, freeze the taxonomy, runtime
feature set, rule, and threshold. Then run the same trace on blind rows 1--200
without revising them. Report both slices separately and combined; evaluation
results may validate or reject the frozen rule, never tune it.

Graph/stitch and codec dispositions are independent outputs. A structural
physical-versus-monolithic deficit receives a separately numbered graph task,
not an in-task rebuild. Task 189 stays dormant unless same-seed per-query
evidence shows reachable correct candidates are lost specifically at RaBitQ
ordering margins; an unchanged exact-neighbor aggregate is not sufficient.

## P2 — At most one adaptive candidate

The only in-task candidate family is a bounded conditional BW8 replay from the
shipped BW4 baseline:

1. The first traversal is exactly BW4/H100/L32. A frozen truth-free rule may
   trigger one BW8/H100 replay from the same persisted seeds.
2. No query triggers more than one replay. The union of control and replay
   candidates is exact-reranked; untriggered queries must be byte-identical to
   BW4, and the candidate pool must contain the control pool.
3. Candidate eligibility on rows 201--400 requires activation on at most 25%
   of queries, capture of at least 50% of queries where BW8 improves paired
   recall, activation of no more BW8-loss queries than
   `ceil(loss_queries * all_query_activation_fraction)`, and a simulated
   paired-recall bootstrap lower bound >= 0. If no frozen rule satisfies all
   four conditions, record `NO RELIABLE SIGNAL` and STOP before runtime
   implementation.
4. The rule may use only values available before truth and final correctness
   are known: round-cap use, heap saturation, frontier churn/stability, score
   gap, repeated-node rate, and owner/transport work. Query ids, truth labels,
   source ids, and corpus-specific lookup tables are forbidden inputs.
5. Rule selection is finite and deterministic. Before joining diagnostic
   traces to labels, construct exactly seven one-predicate rules: round cap
   reached; heap saturated; score gap <= its diagnostic p25; frontier churn,
   repeated-node rate, remote-owner requests, or response bytes >= that
   feature's diagnostic p75. Among rules satisfying item 3, choose highest
   simulated paired-recall delta, then lowest activation, then lexical
   predicate name. Do not search arbitrary thresholds or predicate
   combinations. Freeze the winner before opening rows 1--200.

Screen the candidate on blind 100k rows 1--200 with clean production metrics,
same-generation A/A and A/B predictions, and separate full-metrics
attribution. Advance only if all are true:

- paired recall delta > 0 and its bootstrap 95% lower bound is >= 0;
- zero control wins, byte-identical untriggered results, and control-pool
  containment on every triggered query;
- warm mean, p95, and p99 regress by no more than 5%;
- activation is <= 25%, per-query work obeys the single-replay bound, storage
  is unchanged, and topology/lifecycle semantics conform.

Otherwise STOP and retain the attribution findings without a runtime policy.
Because Task 219 retains recall-equivalence, even a useful recall-changing
candidate is a supported non-default configuration unless an explicit product
policy ruling later reopens the shipped default.

## P3 — Full-scale decision

Only a 100k candidate passing P2 proceeds to fresh 10k/50k/100k release A/B via
`ecaz bench suite`. Apply the same recall, result-containment, latency-tail,
activation/work, storage, and topology gates at every scale. Close with one of:

- `NO RELIABLE SIGNAL`;
- `STOP — ATTRIBUTED, NO USEFUL ADAPTIVE CANDIDATE`;
- `USEFUL NON-DEFAULT ADAPTIVE CONFIGURATION`; or
- `FOLLOW-UP REQUIRED — STRUCTURAL GRAPH OR CODEC TRIGGER`.

Do not label a benchmark winner a shipped default or persisted-format change.

## Non-goals

- Reopening fixed head-capacity/head-selection experiments, BW64/H8, or the
  unchanged exact-neighbor candidate.
- Truth-aware production decisions or evaluation-slice threshold tuning.
- Stacking Task 222--225 materialization candidates with search work.
- Combining a graph rebuild, codec change, and adaptive runtime policy.
- Ad hoc shell/SQL benchmark sweepers or committing corpus/truth caches.

## Acceptance

1. Every registered diagnostic and evaluation miss reconciles at the stated
   boundaries, with unknowns explicit.
2. Physical/monolithic graph and stitch quality receive evidence-backed
   dispositions, and Task 189's trigger is explicitly pass/fail.
3. At most one truth-free conditional-BW8 rule is frozen on the diagnostic
   slice and blindly evaluated, or the task stops for no reliable signal.
4. A useful candidate has clean 10k/50k/100k recall, latency, storage,
   activation/work, topology, and result-containment evidence; otherwise it is
   stopped at 100k.
5. Task status and the task index cite the final reviewed outcome and any
   separately numbered graph/codec follow-up.

## Required review packets

1. `reviews/task-227/001-plan/`
2. `reviews/task-227/002-diagnostic-tooling/`
3. `reviews/task-227/003-query-level-attribution/`
4. `reviews/task-227/004-adaptive-candidate/` (only after a reliable signal)
5. `reviews/task-227/005-full-scale-decision/` (only after a useful 100k screen)

## References

- Task 188 packet 008 and reviewer feedback
- Task 226 packets 002/003 and Task 219's review-closed default policy
- Tasks 185, 189, 207, and 215
- Roadmap GRAPH-10 / GRAPH-17 / GRAPH-18 and TRAV-16..19
