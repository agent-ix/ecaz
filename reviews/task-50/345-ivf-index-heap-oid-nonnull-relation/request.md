# Review Request: IVF Index Heap OID NonNull Relation

Task: 50 unsafe burndown

Commit under review:

- `ba24c978` - `Use NonNull relation for IVF heap OID lookup`

## Summary

This packet removes raw-Relation caller-side unsafe around IVF heap OID resolution.

- Adds `IvfScanDescView::index_relation_nonnull(...)` so scan descriptor users validate the index relation before relation metadata access.
- Changes `ivf_index_heap_oid` to take `NonNull<RelationData>` instead of raw `pg_sys::Relation`.
- Updates IVF debug heap OID lookup to perform the same null check before calling the relation helper.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1308` to `1306`.
- No remaining `unsafe { ivf_index_heap_oid(...) }` callers.
- The broadened raw boundary-signature guard has no hits.

See `artifacts/unsafe-count.log`, `artifacts/ivf-index-heap-oid-scan.log`, and `artifacts/raw-boundary-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1306` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm the checked `NonNull<RelationData>` boundary is appropriate for this IVF relation metadata read.
- Confirm null-index-relation handling remains equivalent for scan and debug paths.
