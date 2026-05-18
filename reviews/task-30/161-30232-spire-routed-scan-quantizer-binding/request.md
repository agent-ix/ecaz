---
id: 30232
title: SPIRE Routed Scan Quantizer Binding
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 55d9ca84
---

# Review Request: SPIRE Routed Scan Quantizer Binding

## Summary

This checkpoint connects the helper-level routed scan path to the SPIRE
assignment payload scorer without wiring AM callbacks or relation-backed
persistence.

- Adds `collect_quantized_routed_probe_candidates(...)`.
- Prepares a `SpirePreparedAssignmentScorer` for the requested assignment
  payload format and query vector.
- Reuses the existing routed top-`nprobe` leaf selection, visible-row filtering,
  `vec_id` dedupe, score ordering, and limit handling.
- Covers TurboQuant and RaBitQ rows encoded through the real assignment payload
  helper.
- Keeps PQ-FastScan explicitly rejected until grouped-PQ model metadata is
  persisted.
- Validates payload-length errors surface through the routed scan wrapper.

## Non-Goals

- No AM scan callback execution.
- No heap exact-rerank callback integration.
- No relation-backed object-store persistence.
- No PQ-FastScan scorer implementation.

## Review Focus

- Whether `collect_quantized_routed_probe_candidates` is the right helper
  boundary, or whether the prepared scorer should be owned by a future scan
  descriptor before more helper integration.
- Whether preparing the scorer with `query_vector.len()` before routing is
  acceptable, given the root routing object still performs authoritative
  dimension validation.
- Whether this wrapper should accept only scoreable formats and exclude
  `PqFastScan` from the enum until grouped-PQ metadata lands.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 145 passed, 0 failed
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
