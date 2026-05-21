# Review Request: SPIRE DML Predicate Expression Boundary

## Scope

This packet reviews commit `6d06e0b03a0714aa647c8ae748ca60dc7f044205` (`Consolidate SPIRE DML predicate expression unsafe`).

The slice consolidates repeated unsafe blocks in the SPIRE DML frontdoor predicate-expression reader. It keeps the existing planner-owned `Query` / `Expr` / `List` contract and groups reads by traversal boundary rather than by individual node/list operation.

## Unsafe Burndown

- Consolidates baserestrictinfo list walking under one planner-list contract.
- Consolidates `OpExpr` clause reads and child expression value extraction under the active-query expression contract.
- Consolidates predicate column wrapper traversal and column-reference recursion.
- Consolidates integer predicate value coercion recursion for `Const`, `Param`, `FuncExpr`, `RelabelType`, and `CoerceViaIO` wrappers.

Unsafe ledger movement:

- previous packet 181 ledger: `1827`
- packet 182 ledger: `1816`
- net reduction: `11`

High-signal file counts from `make unsafe-block-count`:

- `src/am/ec_spire/dml_frontdoor/mod.rs`: `59 -> 48`

## Validation

Packet-local artifacts are under `reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/`.

Passed:

- `cargo-check-pg18-bench.log`
- `cargo-check-pg18-pg-test.log`
- `cargo-test-dml-frontdoor-predicate-pg18-no-run.log`
- `git-diff-check.log`
- `rustfmt-dml-frontdoor-check.log`
- `unsafe-block-count.log`
- `unsafe-ledger-generate.log`
- `unsafe-ledger-check.log`

Blocked:

- `cargo-pgrx-test-dml-predicate-edge-shapes-pg18-blocked.log`: focused pgrx test built, then failed before running tests with unresolved PostgreSQL symbol `BufferBlocks`.
- `cargo-pgrx-test-dml-const-coercion-pg18-blocked.log`: same local symbol-resolution blocker.

## Reviewer Focus

Please check that the consolidated unsafe blocks each keep one coherent planner-owned expression/list contract and that predicate classification semantics are unchanged.
