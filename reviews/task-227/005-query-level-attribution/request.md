---
task: 227
packet: 005-query-level-attribution
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 227 query-level attribution and stop decision

This packet requests review of the Task 227 attribution implementation through
`43a20a0d2` and the frozen 100k diagnostic-slice evidence. The disposition is
`NO RELIABLE SIGNAL`: none of the seven preregistered truth-free predicates
satisfies all four eligibility gates, so the task stops before blind evaluation,
runtime adaptive-search implementation, or packets 006/007.

The release PG18 replay used rows 201--400 of the registered 1,000-query file
on one preserved three-owner physical generation. The parent query SHA-256 is
`a7cbec6f...1782`, the exact slice SHA-256 is `a12a8111...ece9`, and all five
diagnostic arms attest generation identity `0200797e...10c210`. The installed
extension is the preserved release build at `9187e8261`, with only the
benchmark diagnostic feature enabled; the runner at `43a20a0d2` does not
replace that extension.

## Attribution result

The production BW4/RaBitQ control recalls 0.9295 (1,859/2,000 truth
neighbors), while BW8/RaBitQ recalls 0.9565. Owner-oracle BW4 recalls 0.9955
with RaBitQ and 0.9960 with exact neighbor scoring. Production-seed exact
neighbor scoring is slightly worse than the RaBitQ control, 0.9265 versus
0.9295.

All 141 production misses reconcile as `budget_frontier`; there are zero
`unknown` rows. The persisted production head reaches all 100,000 live graph
nodes. Physical and monolithic graph summaries have the same 100,000 live
nodes, 3,101,447 directed edges, degree distributions, 12 SCCs, one weak
component, 99,989 nodes in the largest SCC, and zero aggregate bridge or
articulation candidates. Their local/remote edge labels and adjacency digests
differ by construction. The physical persisted-head identities do not map
into the separately built monolithic control, so its zero mapped seed count is
reported as build-scoped identity mismatch, not as a reachability deficit.

This rejects both conditional follow-up triggers: there is no structural
physical-versus-monolithic graph deficit to assign to a rebuild task, and Task
189 remains dormant because same-seed exact-neighbor scoring does not recover
the residual. The measured residual is a traversal-budget frontier.

## Finite adaptive-rule screen

BW8 improves 29 of 200 queries, loses 3, and raises mean paired recall by
0.027. The closest activation-compliant rule, `score_gap_lte_p25`, activates
50/200 queries but captures only 8/29 improvements (27.6%, below the required
50%). `heap_saturated` is the only rule to capture at least half of the
improvements (18/29), but it activates 121/200 queries and activates all three
loss queries; both exceed their gates. Every other rule also fails at least
one gate. `artifacts/finite-rule-screen.json` records all seven predicates,
nearest-rank thresholds, activation/capture/loss counts, and the deterministic
10,000-sample paired bootstrap.

Because no rule is eligible, opening the blind rows 1--200 would violate the
frozen plan. There is no adaptive runtime candidate, no production behavior or
storage-format change, and no 10k/50k/100k candidate matrix to run.

## Implementation and validation

The implementation range adds the truth join and mutually exclusive residual
classifier, suite/runner attribution artifact plumbing, strict reused-fixture
query and generation provenance, sharded-head replay parity, monolithic
control validation, and the trace quality-bar correction that preserves the
full exact-rerank input while returning only executor-LIMIT final results.

Focused validation passed: six query-slice/suite tests; the diagnostic replay,
reuse provenance, matrix-control, and monolithic-control tests; the trace
rerank/final-result regression; the runner settings regression; and the
release CLI build. The full `ecaz bench suite` run and all cited output are
packet-local. No repository-wide formatter was invoked; functional commits
were diff-gated and the packet/status bookkeeping is separate from code.

Please review classification precedence and reconciliation, trace final-result
semantics, reused-generation provenance, physical/monolithic disposition, the
finite rule math, and the `NO RELIABLE SIGNAL` stop before blind evaluation.
