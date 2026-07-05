# SPIRE SQL VACUUM Rerank Visibility

## Checkpoint

- Code commit: `c87699c8`
  (`Handle SPIRE vacuumed heap rerank candidates`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Real SQL `VACUUM` coverage and heap-rerank visibility after vacuum

## Summary

This checkpoint adds end-to-end PG18 coverage for normal SQL `VACUUM` against a
SPIRE index after:

- populated one-leaf build
- post-build insert that creates an insert delta
- committed heap delete
- real SQL `VACUUM` from an external `psql` session

The test exposed and fixes a scan-side bug: heap rerank treated a vacuumed or
otherwise invisible heap TID as an index error. The reranker now models that
case as an invisible candidate and drops it before tuple emission, while still
hard-failing malformed vectors, NULL indexed values, and non-finite exact
scores.

## Changed Files

- `src/am/ec_spire/scan.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test --lib test_pg18_ec_spire_sql_vacuum_mixed_delta --no-default-features --features pg18 -- --nocapture`
  - Result: `1 passed; 0 failed; 1117 filtered out`
- `cargo test --lib rerank_scored_candidates_by_ip_drops_invisible_candidates --no-default-features --features pg18`
  - Result: `1 passed; 0 failed; 1118 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `238 passed; 0 failed; 881 filtered out`
- `git diff --check`

## Notes

- SQL `VACUUM` coverage now proves insert-delta compaction and deleted-row scan
  invisibility through PostgreSQL's normal callback path.
- Physical page reclamation and old-epoch cleanup remain open follow-ups.
