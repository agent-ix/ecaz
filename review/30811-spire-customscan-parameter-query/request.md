# Review Request: SPIRE CustomScan Parameter Query

Code slice for Step 2 of the ADR-067 CustomScan pivot. This extends the
CustomScan query-vector contract from constant `real[]` ORDER BY arguments to
prepared-statement parameters, so the read-path shape can cover
`ORDER BY embedding <#> $1 LIMIT k`.

## Scope

- Allows `T_Param` real-array expressions in the CustomScan planner query-shape
  gate, in addition to constant `real[]` expressions.
- Initializes and evaluates the copied query expression during
  `BeginCustomScan` via PostgreSQL expression state when the ORDER BY argument
  is a runtime parameter.
- Keeps strict validation that the evaluated query vector is non-null,
  non-empty, `real[]`, and finite.
- Adds a PG18 fixture using `PREPARE ... ORDER BY embedding <#> $1 LIMIT 1`
  against a remote-placement table. The fixture proves parameterized execution
  reaches the production executor and fails on the expected remote transport
  gate.
- Updates the Phase 11 tracker with packet `30811`.

## Validation

- `cargo test customscan_exec --lib`
  - Covers both constant and parameterized query-vector CustomScan execution.
- `cargo test customscan_explain --lib`
- `cargo fmt --check`
- `git diff --check HEAD -- src/am/ec_spire/custom_scan.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Check the `T_Param` gate: it currently accepts only `FLOAT4ARRAYOID`, matching
  the existing AM scan query decoder's `real[]` contract.
- Check expression evaluation lifetime in `BeginCustomScan`: the copied
  `custom_exprs` expression is initialized against the CustomScan plan state and
  evaluated before the production executor stream is requested.
- Check the failure mode: the fixture still expects the production executor
  transport blocker because remote tuple-payload slot delivery is not complete.

## Artifacts

- `review/30811-spire-customscan-parameter-query/artifacts/manifest.md`
- `review/30811-spire-customscan-parameter-query/artifacts/cargo-test-customscan-exec.log`
- `review/30811-spire-customscan-parameter-query/artifacts/cargo-test-customscan-explain.log`
- `review/30811-spire-customscan-parameter-query/artifacts/cargo-fmt-check.log`
- `review/30811-spire-customscan-parameter-query/artifacts/git-diff-check.log`
