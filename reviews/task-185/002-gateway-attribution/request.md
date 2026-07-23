---
task: 185
packet: 002-gateway-attribution
role: coder
date: 2026-07-23
head: 23154d722eee818df1ef4b086b1e76d1d7ceb58e
status: review_requested
---

# Review request: gateway attribution and frozen fixed-cap policies

Task 185 packet 001 already accepted the program split and fixed-cap scope.
This packet implements the measurement-only surfaces required before the 100k
screen. Production defaults, generation formats, graph construction,
neighbor codec, BW/H, materialization, and production head policy are
unchanged.

## What changed

- `training_gateway_set_cover` uses the real `distann_orchestrated_search`
  core to test each bounded training candidate as a single seed on the full
  build graph. It selects cap-4,096 membership by marginal exact-truth
  coverage and emits compact gateway/basin/build-work aggregates.
- `head_basin_diverse` exact-scores the same persisted head and applies one
  deterministic, bounded overlap penalty to the returned-seed set.
- The physical fixture emits structured
  `physical_benchmark_gateway` and `physical_benchmark_basin` rows, and
  `ecaz bench suite` parses them into `results.jsonl`.
- Training (rows 201-400), validation (401-600), and evaluation (1-200) are
  explicitly disjoint. The builder cannot read evaluation outcomes.
- Unit tests pin set-cover tie breaking, marginal behavior, basin-diversity
  behavior, suite policy expansion, and structured result parsing.

The exact four-cell A/B is frozen in
[`artifacts/manifest.md`](artifacts/manifest.md). Membership and returned-seed
selection are isolated before their combined cell, as required by Task 185.

## Review questions

1. Does the single-seed BW4/H100 attribution legitimately measure marginal
   gateway success under the production traversal semantics?
2. Is the validation/evaluation separation sufficient to prevent policy
   selection leakage?
3. Is the query-conditioned head-graph basin definition and fixed
   `rank + 32 * max_jaccard` objective narrow and deterministic enough for the
   pre-registered diversity arm?
4. Are the emitted aggregates sufficient to explain a negative screen without
   retaining per-query or corpus-sized exhaust?
5. Is this checkpoint ready to drive packet 003's suite-only 100k screen?

## Validation

PG18 benchmark-feature Clippy passes with warnings denied. Five focused
extension/CLI checks pass; the commands and durable logs are listed in the
artifact manifest.

No recall, latency, storage, or candidate outcome is claimed yet. Packet 003
will contain the checked-in `ecaz bench suite` config and physical 100k
evidence after this policy checkpoint is reviewed.
