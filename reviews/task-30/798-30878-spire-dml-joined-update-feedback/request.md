# Review Request: SPIRE DML Joined UPDATE Feedback

## Scope

Code commit: `18663ced63a2429203b247c912dd70e58696960b`

This packet addresses reviewer feedback from 30876 before the transparent
UPDATE/DELETE rewrite slices.

Changes:

- Documents the operation-specific join semantics in
  `dml_frontdoor_query_has_join_shape(...)`.
- Strengthens the DML replacement-decision PG fixture so the `UPDATE ... FROM
  other` rejection case uses an `other` table that also has an `ec_spire`
  index.
- Confirms that joined UPDATE still rejects as `unsupported_join_shape` even
  when the joined table is also SPIRE-distributed.
- Updates the Phase 11 task file with packet `30878`.

No planner path generation or executor behavior changes are included.

## Validation

- `cargo test dml_frontdoor --lib`
  - `25 passed; 0 failed; 0 ignored; 1649 filtered out`
  - artifact: `artifacts/cargo-test-dml-frontdoor-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor.rs src/lib.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm the added comment accurately describes SELECT vs UPDATE/DELETE join
   semantics.
2. Confirm the PG fixture now covers the distributed-joined-table concern from
   30876 feedback.
3. Confirm no behavior changed for supported single-table UPDATE/DELETE/PK
   SELECT classification.
