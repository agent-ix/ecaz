# Review Request: Relation Scalar Cost Handles

Task: 50 unsafe burndown

Commit under review:

- `8c4c13bf` - `Use relation handles for cost scalar reads`

## Summary

This packet applies the relation scalar handle helpers to remaining cost and insert wrapper reads.

- Updates DiskANN insert relation block/reltuples accessors to use `RelationHandle` helpers.
- Updates HNSW vacuum/shared block-count and reltuples reads to use checked handles.
- Updates IVF and SPIRE cost snapshots/estimates to use checked handles for reltuples and block counts.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1278` to `1265`.
- The updated files no longer have targeted caller-side `unsafe { relation_reltuples(...) }` or `unsafe { main_fork_block_count(...) }` wrappers.
- The broadened raw boundary-signature guard has no hits.

See `artifacts/unsafe-count.log`, `artifacts/relation-scalar-cost-handle-scan.log`, and `artifacts/raw-boundary-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1265` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm relation-handle scalar reads preserve cost model inputs.
- Confirm raw relation pointers remain available only where downstream page/store APIs still require them.
