---
agent: coder1
role: coder
model: gpt-5
date: 2026-05-14
topic: c1-spire-cost-tuning-model-coverage
code_commit: 6728742d
---

# Review Request: SPIRE Cost Tuning Model Coverage

## Summary

Added a focused pure-Rust cost-model coverage pin for all six SPIRE cost tuning
knobs:

- `cost_routing_dimension_scale`
- `cost_leaf_dimension_scale`
- `cost_index_page_scale`
- `cost_local_store_page_fanout_scale`
- `cost_storage_scoring_multiplier`
- `cost_rerank_multiplier`

The new test doubles and triples each tuning value independently and asserts the
total-cost delta scales linearly. The rerank case uses a rerank-width-zero
fixture so `cost_rerank_multiplier` is actually active.

## Scope

Changed:

- `src/am/ec_spire/cost/tests.rs`

This supports 12c.10.c by tightening the model-level baseline under the GUCs.
It does not close the full row by itself because the task also asks for an
EXPLAIN-level GUC reflection fixture.

File-size check: `src/am/ec_spire/cost/tests.rs` is 272 lines after this slice.

## Validation

Passed:

- `cargo fmt --check`
  - Stable rustfmt emitted the repository's existing warnings about nightly-only
    `imports_granularity` and `group_imports`.
- `git diff --check -- src/am/ec_spire/cost/tests.rs`
- `cargo test --no-default-features --features pg18 individual_cost_tuning_knobs_scale_modeled_costs_linearly --no-run`
  - Existing unused-import warnings in `src/am/mod.rs`.

## Review Focus

Please check whether the double/triple delta assertion is the right model-level
pin before adding the SQL EXPLAIN reflection test.
