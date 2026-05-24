# Review Request: Relation OID NonNull Helpers

Task: 50 unsafe burndown

Commit under review:

- `9582e11a` - `Use NonNull relation oid helpers`

## Summary

This packet extends the relation handle helpers to relation OID and tablespace reads, then rolls them into IVF/SPIRE call sites.

- Adds `relation_tablespace_handle` and reuses `relation_oid_handle` over the `RelationHandle` alias.
- Keeps raw-pointer storage helpers as checked wrappers.
- Updates IVF insert locking, SPIRE insert, SPIRE relation planning, SPIRE relation object store setup, and SPIRE snapshot relid reads to use checked handles.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1286` to `1278`.
- The updated call sites no longer wrap `relation_oid` / `relation_tablespace` in caller-side unsafe blocks.
- The broadened raw boundary-signature guard has no hits.

See `artifacts/unsafe-count.log`, `artifacts/relation-oid-helper-scan.log`, and `artifacts/raw-boundary-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1278` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm the relation OID/tablespace handle helpers preserve the previous scalar-read semantics.
- Confirm the updated SPIRE/IVF callers keep raw relation pointers available only where downstream page/store APIs still require them.
