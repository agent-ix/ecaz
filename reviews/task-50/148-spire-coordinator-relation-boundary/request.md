# Review Request: SPIRE Coordinator Relation Boundary

## Summary

This checkpoint addresses part of the soundness-audit feedback around SPIRE
coordinator helpers that accepted a raw `pg_sys::Relation` behind safe
interfaces.

Changes:

- Made `SpireLiveIndexRelation::new`, `live_index_relation`, and
  `live_index_relid` unsafe because callers must guarantee that the PostgreSQL
  relation descriptor is live for the view.
- Propagated the unsafe boundary through remote-node and remote-search helper
  functions that dereference or derive state from the live index relation.
- Switched validated SQL-facing call sites in `lib.rs` to
  `with_live_index_relation!`, keeping the explicit safety acknowledgment at
  the relation-guard boundary.
- Added explicit unsafe call acknowledgments for two internal remote candidate
  users that pass through a live coordinator relation.

## Validation

- Pass: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Artifact: `artifacts/cargo-check-pg18-bench.log`
  - Note: pre-existing unused-import warning in `src/am/mod.rs`.
- Pass: `git diff --check`
  - Artifact: `artifacts/git-diff-check.log`
- Pass: `make unsafe-block-count`
  - Artifact: `artifacts/unsafe-block-count.log`
  - Summed unsafe count: 1617.
- Fail: `make unsafe-ledger-check`
  - Artifact: `artifacts/unsafe-ledger-check.log`
  - The ledger check reports broad stale/unledgered rows (`1615` unledgered,
    `2444` stale) and is not isolated to this checkpoint.

## Reviewer Focus

Please check whether the unsafe boundary propagation is at the correct layer:
raw `pg_sys::Relation` validity is now required at the helper boundary, while
SQL entry points keep the guard-backed safety proof.
