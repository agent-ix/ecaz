# Task 50 Review Request: SPIRE Live Relation Wrapper Soundness Repair

## Summary

This packet responds to the new soundness-audit feedback in packets 305 and 311-315.

The reviewer was correct: my previous SPIRE wrapper slices removed `unsafe fn` signatures while keeping safe helpers that accepted raw `pg_sys::Relation` and internally constructed a live relation view. That recreated anti-pattern A.

This repair removes the safe `checked_live_index_relation` helper and changes the affected SPIRE paths to take `SpireLiveIndexRelation` where they need a live index view. SQL wrapper call sites now construct that view at the `IndexRelationGuard` boundary through `with_spire_live_index_relation!`; remaining raw-pointer callers use explicit unsafe boundaries.

It also fixes packet 305 feedback by changing `open_storage_relation_or_index` to take `SpireLiveIndexRelation` instead of a raw index relation pointer.

## Feedback Assessment

- 304: approved by reviewer; no change needed.
- 305: reviewer was correct. Fixed by moving `open_storage_relation_or_index` to the live relation view.
- 306-310: approved by reviewer; no change needed.
- 311-315: reviewer was correct. Fixed by removing `checked_live_index_relation` and threading `SpireLiveIndexRelation` through the affected SPIRE remote-search/libpq/snapshot helpers.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src | wc -l`
- `rg -n unsafe src --count-matches`
- `make UNSAFE_LEDGER=reviews/task-50/316-spire-live-relation-wrapper-soundness-repair/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/316-spire-live-relation-wrapper-soundness-repair unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/316-spire-live-relation-wrapper-soundness-repair/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

Results:

- Unsafe line count: `2003`
- Unsafe ledger rows: `1382`
- `cargo check` passed with the pre-existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`.

The unsafe count increased relative to packet 315 because this repair deliberately restores explicit unsafe boundaries for soundness instead of hiding them behind safe raw-pointer helpers.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
