---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 nonnegative incremental-prune distance

Status: review-open; algorithm candidate implemented and unit-validated;
production 10k/50k/100k rerun pending.

Please review checkpoint `a001bf7e6`.

Packet 041 replaced the defective pairwise-overlap instrument with exact fp32
ground truth and measured a real 10k loss: inserted-neighborhood physical
distinct recall `0.805382` versus fresh `0.954985` (`-0.149603`), and held-out
`0.973684` versus `0.977632` (`-0.003947`).

The root cause is in the pure incremental insert planner. It passed raw
`-inner_product` to `robust_prune`. That preserves nearest-neighbor sorting but
produces negative distances for similar unit vectors. `robust_prune` explicitly
requires nonnegative distances for `alpha * d(kept, candidate) <= d(pivot,
candidate)` to be meaningful. Batch DistANN construction already uses
`max(0, 1 - inner_product)`.

This checkpoint routes incremental forward-edge selection and full backlink
re-pruning through the same shared `source_inner_product_distance` helper as
batch construction. It does not change degree, alpha, search width, beam
width, query sample, or the exact-recall threshold. Tests pin equality with
the batch metric across dimensions 1–1536, nonnegativity, ordering of positive
inner-product pairs, and the full existing insert-planning module.

The immutable rerun config is
[`artifacts/task167-distance-fix-suite.json`](artifacts/task167-distance-fix-suite.json).
No runtime win is claimed until that suite supplies 10k/50k/100k exact recall,
ordinary recall/latency, insert throughput/work, concurrency, and storage
evidence.
