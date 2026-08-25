---
task: 227
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 227 residual attribution and adaptive-search plan

This packet requests review of the refined Task 227 plan at `b2efaed82`.
Task 226 now satisfies the entry gate with BW8 classified as a useful
non-default configuration; Task 219's review-closed recall-equivalence policy
keeps BW4/H100/L32 as the production control.

The plan first closes the tooling gap that made Task 188's residual impossible
to measure: benchmark-feature-gated per-query seed/frontier/rerank traces,
read-only distributed-graph topology, CLI-side truth containment, and suite
addressability. It then classifies every miss into a mutually exclusive
generation, seed/reachability, graph, budget/frontier, approximate-ordering,
rerank, exact-competition, or explicit-unknown category. A same-corpus
monolithic graph is diagnostic only and is not presented as same-generation.

Selection leakage is bounded before implementation. The 1,000-row 100k query
file is frozen into rows 201--400 for calibration and blind rows 1--200 for
evaluation, with parent and slice SHA-256s recorded in the task. The adaptive
family is narrowed to one conditional BW8/H100 replay from the shipped BW4
baseline. Seven one-predicate truth-free rules and deterministic tie-breaking
are registered; if none meets the activation, BW8-win capture, loss-query,
and simulated paired-recall gates on the calibration slice, the task stops
without runtime-policy code.

A selected rule must then pass a clean same-generation 100k A/B with positive
paired recall, zero control wins, exact result-containment semantics, <=5%
mean/p95/p99 regression, <=25% activation, and a single-replay work bound.
Only that result unlocks the required 10k/50k/100k suite matrix. Even a winner
is a supported non-default configuration under Task 219 unless an explicit
product-policy ruling later reopens the default.

Please review the trace/graph boundary, miss-classification priority, frozen
query split, finite signal-selection rule, conditional-replay semantics, and
numerical screen/full-scale gates. This is a planning-only packet; no code or
new measurement result is under review.
