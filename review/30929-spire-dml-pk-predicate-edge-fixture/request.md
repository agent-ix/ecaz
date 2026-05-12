---
topic: spire-dml-pk-predicate-edge-fixture
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30929
stage: phase-12.6
status: open
---

# Review Request: SPIRE DML PK Predicate Edge Fixture

## Scope

Please review commit `2c2ca55f1c8f7554a870dca5ea0ab0d2358d37a0`
(`Add SPIRE DML PK predicate edge fixture`).

This slice adds PG18 coverage for the Phase 12.6 negative DML classifier rows:

- numeric PK equality outside int8 range;
- NULL int8 PK equality;
- `WHERE id IN (...)`;
- `WHERE id = ... OR id = ...`;
- numeric equality that must not be accepted through implicit coercion;
- planner-hook fail-closed behavior for unsupported SPIRE-fronted PK SELECT
  shapes, including `feature_not_supported` SQLSTATE.

The Phase 12.6 tracker now marks the negative PK predicate fixture and
unsupported-shape fail-closed rows complete.

## Review Focus

- Confirm the fixture covers the intended Phase 12.6 predicate edge cases
  without overclaiming broader isolation/EvalPlanQual coverage.
- Confirm testing the prepared numeric parameter at `EXECUTE` time is the right
  PostgreSQL planning boundary; `PREPARE` itself does not invoke the failure
  path here.
- Confirm the tracker wording is scoped correctly to unsupported PK predicate
  shapes and SPIRE-fronted fail-closed behavior.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_dml_frontdoor_rejects_pk_predicate_edge_shapes`

Key result: the focused PG18 test passed with `1 passed; 0 failed; 1684 filtered out`.
