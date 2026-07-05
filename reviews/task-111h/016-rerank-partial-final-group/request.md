# Task 111h / 016 Rerank Partial Final Group

## Summary

This packet requests review for commit
`8c9c9ad5f9acc70eede9fd4c6b22812e2c73010f`
(`task111h: cover partial final rerank groups`).

The 008 feedback noted that padding for a final group with
`valid_count < scorer_width` was only indirectly covered. This slice adds a
PG18 fixture that builds an index-side f16 coarse-rerank index with three rows
and `rerank_width = 8`, forcing a partial final packed rerank group.

The fixture asserts:

- exactly three rows are reranked;
- exactly three rows are emitted, so padded slots do not leak;
- f16 scored payload bytes equal three valid payloads only;
- index placement still reads zero heap source-vector bytes;
- f16 scalar scoring performs no batch slab copy.

## Non-Claims

- This is not a benchmark packet.
- This does not change the packed rerank group layout.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo check --no-default-features --features pg18` passed.
- `cargo pgrx test pg18 test_ec_ivf_index_placement_partial_final_group`
  passed.
