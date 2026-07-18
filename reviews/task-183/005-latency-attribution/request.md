---
task: 183
packet: 005-latency-attribution
role: coder
status: open
date: 2026-07-17
head: 97cd5a76a
---

# Review request: latency attribution result

Phase 2 produced no recall candidate and Phase 3 was conditionally skipped.
Phase 4 therefore profiles only the retained Task 182 production policy:
trained cap 4,096, exact scoring, 32 returned seeds, BW4/H100, RaBitQ neighbor
traversal, and exact final ranking on the physical three-owner path.

Before reading a new stage profile, this packet freezes benchmark-only
per-backend aggregate counters with the following non-overlapping or explicitly
nested stages:

1. query/codec preparation;
2. exact dot-product scoring of all persisted landmarks;
3. seed ranking/truncation after scoring;
4. total bounded traversal, with nested local-owner expansion and remote-owner
   expansion/transport counters;
5. remote result-payload materialization;
6. ranked-output construction/merge; and
7. total CustomScan search/materialization setup.

The client-side mean minus mean total CustomScan setup is reported as the
executor/client residual. Traversal control/merge is derived as total traversal
minus local and remote expansion. Nested counters will never be summed as if
they were independent stages.

The counters reset after warmups and snapshot after timed queries on the same
latency-worker backend. They are compiled only with the existing
`distann-head-attribution-benchmark` feature and do not alter production
defaults or persisted data. The CLI will fail closed if the requested snapshot
functions are unavailable.

The profile is one fresh 100k physical generation, 50 timed queries after 10
warmups, driven by the checked-in `ecaz bench suite` config. The retained path
reproduced 0.9625 recall (95% CI 0.9532--0.9700) and measured 40.20 ms mean,
39.20 ms p50, 51.50 ms p95, and 56.30 ms p99.

The non-overlapping wall-time attribution is:

| Stage | Mean/query | Share of wall mean |
| --- | ---: | ---: |
| remote payload materialization | 26.955 ms | 67.05% |
| bounded traversal | 7.918 ms | 19.70% |
| exact head scoring | 2.272 ms | 5.65% |
| executor/client residual | 2.170 ms | 5.40% |
| other CustomScan setup | 0.702 ms | 1.75% |
| seed selection | 0.101 ms | 0.25% |
| output merge | 0.053 ms | 0.13% |
| query preparation | 0.028 ms | 0.07% |

Traversal contains 1.310 ms local expansion, 6.540 ms remote expansion, and a
derived 0.068 ms traversal control/merge remainder. Those nested values are
not added again in the wall-time table. The executor/client residual is the
40.20 ms client mean minus the 38.030 ms CustomScan-total mean. Other
CustomScan setup is the CustomScan total less the six independent instrumented
stages.

This profile activates the pre-registered no-candidate branch. Remote payload
materialization dominates, while neither eligible Task 183 target does: head
scoring is 5.65% and seed selection is 0.25% of wall mean. A head-scoring or
selection rewrite therefore is not selected, and the task does not spend a
10k/50k/100k confirmation matrix on a nonexistent candidate. Remote
materialization is routed to Task 184 for deeper attribution and an isolated
A/B. Task 183 changes no production default or persisted format.

The pre-registered decision contract was:

- head scoring dominance permits one contiguous/vectorized exact-scoring A/B,
  with scalar equivalence and seed-identity checks;
- seed ranking dominance permits one bounded selection implementation with
  identical ordered seeds;
- a reproduced 10k-only regression without recall gain permits a small-corpus
  bypass; and
- dominance by traversal, remote transport/materialization, or executor
  residual yields no Task 183 latency candidate because those are outside the
  eligible isolated changes.

Any selected change would have been measured alone against byte-identical
index and query inputs. Because none was selected, packet 006 records the
conditional STOP decision.

Implementation checkpoint `03921f632` adds the feature-gated stage atomics and
snapshot/reset SQL functions, instruments exact landmark scoring and selection,
physical query preparation, nested local/remote expansion, traversal,
materialization, output merge, and total CustomScan setup, and teaches
`ecaz bench latency` plus `ecaz bench suite` to emit structured stage rows.
Both the measurement-feature and normal production PG18 builds compile. The
counter, merge/format, and suite expansion/parser tests pass.

The checked-in 100k suite expanded to one production-policy physical fixture
with stage counters enabled only for its latency arm. It completed one step
with zero failures, skipped steps, missing artifacts, or stale artifacts.
Please review counter nesting, timer boundaries, the derived decomposition,
feature isolation, and the no-candidate decision.
