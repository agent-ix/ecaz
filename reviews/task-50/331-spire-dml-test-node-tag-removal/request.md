# Review Request: SPIRE DML Test Node Tag Unsafe Removal

Task: 50 unsafe burndown

Commit under review:

- `28e8b780` - `Remove redundant DML test node tag unsafe`

## Summary

This packet removes the DML frontdoor test-only `expr_node_tag()` helper, which directly dereferenced `pg_sys::Expr` pointers to assert `NodeTag::T_Const`.

The same test already verifies the primitive-plan mode, non-null handoff expression, primitive plan construction, and const PK byte output. Keeping a raw node-tag deref only for the redundant assertion was not worth retaining another direct unsafe block.

## Unsafe Count Impact

- `src/tests/dml_frontdoor.rs`: `3 -> 2`
- Current `src/` total: `1347`

See `artifacts/unsafe-counts-before-after.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1347` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Runtime DML pg tests remain blocked by the PostgreSQL symbol-linkage issue captured in packet 329.

## Reviewer Focus

- Confirm the removed raw `NodeTag::T_Const` assertion was redundant with the remaining primitive-plan and const-byte assertions.
