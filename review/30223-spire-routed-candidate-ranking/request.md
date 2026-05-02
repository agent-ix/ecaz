---
id: 30223
title: SPIRE Routed Candidate Ranking
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: a0b4e3a0
---

# Review Request: SPIRE Routed Candidate Ranking

## Summary

This checkpoint adds the first candidate-ranking seam on top of SPIRE routed
leaf reads.

- Adds `SpireScoredScanCandidate` as the scan-local candidate shape carrying
  PID, object version, leaf row index, `vec_id`, heap TID, and score.
- Adds `collect_ranked_routed_probe_candidates`, which routes through the root
  object, reads the top-`nprobe` leaf objects, scores visible primary rows, and
  applies an optional limit.
- Converts scorer-provided inner products to PostgreSQL AM distance ordering by
  storing `score = -ip`.
- Deduplicates by `vec_id` before final sorting so boundary replicas or future
  overlapping routed reads do not emit duplicate candidates.
- Keeps deterministic tie breaks on heap TID, PID, and row index after score.
- Rejects non-finite scorer outputs instead of letting NaN ordering leak into
  scan result selection.

## Non-Goals

- No AM callback execution or scan opaque state.
- No real quantizer binding; the scorer remains injected by the caller.
- No heap rerank or visibility callback.
- No remote placement, replica read, or adaptive probe-width behavior.

## Review Focus

- Whether `score = -ip` is the right contract for this helper, or whether the
  helper should expose both raw inner product and AM order-by score.
- Whether `vec_id` dedupe should keep the best candidate by score as implemented
  here, or preserve deterministic first-route provenance for diagnostics.
- Whether tie breaks should include `vec_id` bytes before PID/row index once
  global IDs are in use.
- Whether non-finite scorer rejection belongs here or in the eventual scorer
  adapter.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 129 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
