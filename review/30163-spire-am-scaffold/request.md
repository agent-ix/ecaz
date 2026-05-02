---
id: 30163
title: SPIRE Access Method Scaffold
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 7ae2cd4e
---
# Review Request: SPIRE Access Method Scaffold

## Summary

This checkpoint starts Task 30 Phase 1 after the Phase 0 storage decision.

The checkpoint:

- adds `src/am/ec_spire/` following the ADR-041 AM module shape
- registers `ec_spire` in the AM module tree
- wires an `ec_spire_handler` and `IndexAmRoutine`
- registers `ec_spire` plus `ecvector_spire_ip_ops` and
  `tqvector_spire_ip_ops` in `sql/bootstrap.sql`
- keeps build, scan, insert, vacuum, and cleanup callbacks as explicit
  unsupported stubs until partition-object persistence lands
- gates planner selection with the shared high-cost placeholder model
- adds pg catalog tests for `ec_spire` AM and opclass registration
- updates the Task 30, FR-029, top-level spec, StR-005, tests matrix, and ADR
  index references to call this a scaffold rather than a working persistence
  path

No partition-object persistence code is included in this checkpoint.

## Files To Review

- `src/am/ec_spire/mod.rs`
- `src/am/ec_spire/routine.rs`
- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/insert.rs`
- `src/am/ec_spire/scan.rs`
- `src/am/ec_spire/vacuum.rs`
- `src/am/ec_spire/cost.rs`
- `src/am/ec_spire/{assign,storage,meta,update}.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `sql/bootstrap.sql`
- `plan/tasks/30-spire-ivf-foundation.md`
- `spec/functional/FR-029-multi-am-sql-bootstrap.md`
- `spec/spec.md`
- `spec/stakeholder/StR-005-multi-am-vector-search.md`
- `spec/tests.md`
- `spec/adr/index.md`

## Validation

- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`

`cargo fmt` emits the repository's existing stable-toolchain warnings for
unstable rustfmt options (`imports_granularity`, `group_imports`), but the
format check passed after formatting.

## Reviewer Focus

1. Is it acceptable to register `ec_spire` as an opt-in scaffold while all
   persistence callbacks fail explicitly?
2. Does the AM routine match the existing `ec_ivf`/`ec_hnsw` callback posture
   closely enough for the next persistence slice?
3. Are the SPIRE opclass names and catalog tests consistent with the Phase 0
   surface decision?
4. Does the docs/spec wording avoid implying that SPIRE build/scan persistence
   already works?
