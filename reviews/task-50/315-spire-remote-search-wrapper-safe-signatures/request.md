# Task 50 Review Request: SPIRE Remote Search Wrapper Safe Signatures

## Summary

This packet covers a two-commit SPIRE remote-search unsafe burndown slice:

- `905f9ba2 Make SPIRE remote search wrappers safe`
- `4dd64664 Centralize SPIRE live relation wrapper construction`

The slice removes unsafe public/helper signatures from remote-search SQL wrapper paths and then consolidates the remaining live-relation proof obligation behind `checked_live_index_relation`. That avoids replacing unsafe function signatures with scattered local `unsafe` blocks.

Primary changes:

- Made remote search candidate, local coordinator, local heap resolution, result summary, endpoint identity, and scan-output wrapper helpers safe where they only require the live relation contract supplied by their SQL/AM entry point.
- Updated `src/lib.rs` call sites to use `with_live_index_relation_safe!` for the newly safe SPIRE remote-search wrappers.
- Replaced repeated `unsafe { live_index_relation(index_relation) }` construction in remote-search helper paths with the checked live-relation helper.

## Validation

- `git diff --check HEAD~2..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src | wc -l`
- `rg -n unsafe src --count-matches`
- `make UNSAFE_LEDGER=reviews/task-50/315-spire-remote-search-wrapper-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/315-spire-remote-search-wrapper-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/315-spire-remote-search-wrapper-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

Results:

- Unsafe line count: `2001`
- Unsafe ledger rows: `1367`
- `cargo check` passed with the pre-existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
