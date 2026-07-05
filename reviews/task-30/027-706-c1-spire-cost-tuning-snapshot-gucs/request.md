---
agent: coder1
role: coder
model: gpt-5
date: 2026-05-14
topic: c1-spire-cost-tuning-snapshot-gucs
code_commit: 05cf28a5
---

# Review Request: SPIRE Cost Tuning Snapshot GUCs

## Summary

Added a SQL-level cost tuning fixture in `src/tests/spire_cost_tuning.rs`:

- Builds a small `ec_spire` RaBitQ index with `rerank_width = 0`.
- Captures baseline `ec_spire_index_cost_snapshot(...).modeled_total_cost`.
- Sets all six SPIRE cost tuning GUCs to `2.0`.
- Asserts `modeled_total_cost` increases through the live SQL snapshot path.
- Asserts `ec_spire_index_cost_tuning_snapshot(...)` reflects all six session
  GUC values.
- Pins derived effective values:
  - RaBitQ storage scoring baseline `0.45 * 2.0 = 0.90`.
  - `rerank_width = 0` keeps `effective_rerank_multiplier = 2.0`.

The new test lives in its own file rather than growing existing cost/planner
test files.

## Scope

Changed:

- `src/tests/spire_cost_tuning.rs`
- `src/tests/mod.rs`

This is a 12c.10.c-adjacent SQL/operator-surface pin. It does not fully close
the requested EXPLAIN-level reflection check; that still needs a fixture that
parses planner-visible EXPLAIN costs.

File-size check:

- `src/tests/spire_cost_tuning.rs`: 93 lines.
- `src/tests/spire_recall.rs`: 79 lines.
- `src/tests/mod.rs`: 2806 lines. This shared harness was already above the
  2500-line target; the slice keeps new test logic out of it, but a later
  structural split should move more includes/helpers out of `mod.rs`.

## Validation

Passed:

- `cargo fmt --check`
  - Stable rustfmt emitted the repository's existing warnings about nightly-only
    `imports_granularity` and `group_imports`.
- `git diff --check -- src/tests/mod.rs src/tests/spire_cost_tuning.rs`
- `cargo test --no-default-features --features pg18 test_ec_spire_cost_tuning_snapshot_reflects_session_gucs_sql --no-run`
  - Existing unused-import warning in `src/am/mod.rs`.

## Review Focus

Please check whether this is a useful intermediate SQL-level guard for cost GUC
plumbing, and whether the remaining 12c.10.c work should parse EXPLAIN JSON or
plain-text EXPLAIN costs.
