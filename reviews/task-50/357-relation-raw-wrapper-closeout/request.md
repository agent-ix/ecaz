# Task 50 Review Request: Relation Raw Wrapper Closeout

## Summary

This slice closes out the remaining raw `storage::relation` wrapper call surface for `main_fork_block_count`, `relation_oid`, and `index_heap_relation_oid`.

Code commit: `f4556dee11bbaa95fa7db8faec7134495181efff`

Changes:

- Replaced remaining production and test calls to raw relation wrappers with `RelationHandle` APIs.
- Added guard/view handle access where an owning relation guard already proves liveness.
- Deleted the now-unused raw wrappers from `src/storage/relation.rs`.
- Kept the raw pointer dereferences centralized in the handle helpers that own the relation metadata contracts.

Unsafe count:

- Before: `1230`
- After: `1228`
- Delta: `-2`

Targeted scan result:

- No remaining calls to `crate::storage::relation::{main_fork_block_count,relation_oid,index_heap_relation_oid}(...)` under `src/am`, `src/lib.rs`, `src/tests`, or `src/storage`.

## Validation

Artifacts are under `reviews/task-50/357-relation-raw-wrapper-closeout/artifacts/`.

- `cargo-check-pg18-bench-clean.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed. It reports the pre-existing SPIRE DML re-export warning in `src/am/mod.rs`.
- `git-diff-check-clean.log`: `git diff --check` passed.
- `unsafe-count-clean.log`: `1228`.
- `raw-boundary-guard-clean.log`: no matches.
- `relation-raw-wrapper-call-scan-clean.log`: no matches.
- `unsafe-ledger-after.jsonl` and `unsafe-ledger-check-clean.log`: ledger regenerated and covers all `1228` current unsafe rows.

The first cargo rerun found two issues before the clean pass: a wrong assumption that SPIRE snapshot storage relations were `RelationGuard`s, and dead warnings for raw wrappers after the targeted scan went empty. Both were fixed before final validation.
