# Review Request: Task 28 IVF Cost Model

Status: open
Owner: coder2
Code checkpoint: ceb8c69321b512298d33444bda18e0c7b8f0b47d
Branch: task28-ivf
Date: 2026-04-25

## Scope

This packet covers the Phase 7 planner-cost checkpoint for the first `ec_ivf`
access method baseline.

Changes in scope:

- Replace the `ec_ivf` `amcostestimate` `f64::MAX` stub with a finite
  metadata-based model.
- Model centroid scoring, selected-list posting reads, candidate scoring,
  storage-profile scoring multipliers, and rerank multipliers.
- Add `ec_ivf_index_cost_snapshot(regclass)` so reviewers can inspect modeled
  inputs and outputs without relying on EXPLAIN formatting.
- Reuse the centralized effective `nprobe` resolver from scan/admin code.
- Mark the Phase 7 cost-model checklist item complete in
  `plan/tasks/28-ivf-access-method.md`.

## Files

- `src/am/ec_ivf/cost.rs`
- `src/am/ec_ivf/routine.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

## Validation

PG18-only validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_cost_snapshot_reports_modeled_costs`
- `cargo pgrx test pg18 test_ec_ivf_admin_snapshot`
- `git diff --check`

PostgreSQL version: 18.3 via pgrx `pg18`.

No measurement claims are made in this packet.

## Review Focus

- Whether the first IVF model is conservative enough while still removing the
  planner-prohibitive `f64::MAX` stub.
- Whether planner calls should stay metadata-only rather than scanning
  directory tuples for exact distribution data.
- Whether the exposed cost snapshot fields are sufficient for follow-up EXPLAIN
  and measurement work.

## Non-Goals

- Benchmark-calibrated cost constants.
- EXPLAIN counters.
- PG18 ReadStream or shared stats wiring.
- Recall/latency/storage measurements.
