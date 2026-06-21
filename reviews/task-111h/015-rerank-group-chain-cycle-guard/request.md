# Task 111h / 015 Rerank Group Chain Cycle Guard

## Summary

This packet requests review for commit
`36633b08dd49c9ead82508bf9448a6a79d777b5b`
(`task111h: guard rerank group chain walks`).

The 008 feedback noted that packed rerank group-chain walkers had no cycle
guard. This slice adds a shared `remember_rerank_group_chain_tid` helper and
uses it in both live walkers:

- scan full-chain fallback loader for postings without direct group TIDs;
- vacuum packed-group tombstoning walker.

A repeated group TID now returns an error instead of looping indefinitely. The
unit test pins the helper behavior; the focused PG18 fixtures cover the live
fallback and vacuum paths on valid chains.

## Non-Claims

- This does not add a synthetic corrupted-index SQL fixture.
- This is not a benchmark packet.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo test --no-default-features --features pg18 rerank_group_chain_visit_rejects_cycle --lib`
  passed.
- `cargo check --no-default-features --features pg18` passed.
- `cargo pgrx test pg18 test_ec_ivf_index_placement` passed five PG18 fixtures.
