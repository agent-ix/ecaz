---
id: 30236
title: SPIRE Quantized Rerank Helper
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 0d0a1772
---

# Review Request: SPIRE Quantized Rerank Helper

## Summary

This checkpoint composes the concrete quantized routed scan helper with the
existing exact-rerank seam.

- Adds `collect_reranked_quantized_routed_probe_candidates(...)`.
- Routes to top-`nprobe` leaves, scores real encoded assignment rows through
  the selected quantized payload scorer, dedupes by `vec_id`, applies the
  requested candidate limit, then reranks the configured prefix with an
  injected exact scorer.
- Keeps heap fetch / AM callback ownership outside this helper.
- Adds coverage proving an exact rerank callback can reorder quantized routed
  candidates.

## Non-Goals

- No heap exact-score implementation.
- No AM scan descriptor or callback wiring.
- No relation-backed persistence.
- No PQ-FastScan scorer binding.

## Review Focus

- Whether the helper should apply the approximate candidate `limit` before
  exact rerank, or keep a separate candidate-budget/final-limit API before AM
  wiring.
- Whether this composition belongs in `scan.rs` now or should wait until the
  scan descriptor shape is concrete.
- Whether the rerank callback should receive more row provenance than
  `SpireScoredScanCandidate` currently stores.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 148 passed, 0 failed
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
