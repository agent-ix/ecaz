# Task 188 finding disposition

Date: 2026-07-27
Branch head: `30febc1694a174b73d8809e87b0af75112ae1522`

## Decision-critical evidence

The corrected packet-005 suite is the acceptance source of truth. It used the
PG18 local three-owner physical lane, isolated one-index-per-table surfaces,
explicit `materialization_batch_size=10`, 200 shared queries, and 2,000 paired
trials per scale:

| scale | BW4 recall | BW8 recall | paired BW8 wins/losses/ties | paired recall delta | bootstrap 95% | warm mean delta | p95 delta | storage delta |
| --- | ---: | ---: | --- | ---: | --- | ---: | ---: | ---: |
| 10k | 1.0000 | 1.0000 | 0 / 0 / 200 | +0.0000 | — | -1.00 ms | -1.20 ms | 0 |
| 50k | 0.9840 | 0.9865 | 5 / 0 / 195 | +0.0025 | [+0.0005,+0.0050] | -1.30 ms | -0.90 ms | 0 |
| 100k | 0.9740 | 0.9805 | 7 / 0 / 193 | +0.0065 | [+0.0020,+0.0125] | -2.30 ms | -3.10 ms | 0 |

Both arms share each scale’s build and passed topology/remote-owner
engagement. The final rule application is therefore based on the corrected
batch-10 regime. The earlier eager-0 50k cost (+19.8% mean, +44% p95) is
retained as historical diagnostic context only, not mixed into the decision.

## Scope and ownership

The original Phase 1 requirement listed six attribution families. The run
completed head-vs-owner-oracle, isolated BW, and isolated H controls. It did
not run candidate-frontier/exact-rerank containment, components/indegree/
bridge/hard-query reachability, or monolithic-versus-sharded graph-quality
audits. Those omissions are now explicit scope, not hidden assumptions:

- Task 188 selects only the measured BW8 search-budget candidate.
- The exact-scored 16,384 training-landmark head is the candidate’s research
  reference surface; no production head or default is promoted.
- A future Task 189 trigger may own ordering/neighbor-estimation evidence.
- Task 190 owns the separately measured sequential transport/architecture
  residual.
- The unrun graph-quality audits are not assigned to either task and are not
  claimed to be refuted. Reopening them requires a separately scoped task and
  its own `ecaz bench suite` evidence.

The entry evidence from Task 186 is also qualified: its cited packets lived on
the unmerged `task-186-bounded-hierarchical-head` branch, and the hierarchy
latency result was a query-time/arbitrary-representative prototype that
re-bucketed the stored head per query. It was not treated as a merged
production routing conclusion.

## Measurement provenance labels

- Packet 002 stage rows: instrumented, eager-0 historical attribution.
- Packet 005 latency rows: explicit batch-10, no stage counters, accepted A/B
  decision source.
- Packet 006 efficient diagnostic: explicit batch-10, reconnecting
  `worker_batch_size=5`; p50 and direct stage values are cited, while mean/p95/
  p99 and contaminated `custom_scan_total` are excluded from latency claims.
- Packet 007: pre-refactor/current default-worker equivalence plus the harness
  fixes and suite reachability changes.

No new benchmark values are introduced by this audit.
