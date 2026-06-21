# Task 111h / 012 Rerank Insert Chain Prepend

## Summary

This packet requests review for commit
`db6b6397794ee718aae58e45e0ec822c9829471a`
(`task111h: relink rerank group inserts under metadata lock`).

The 008 review carried forward a concurrent insert risk: inserted packed rerank
groups were appended with `next_group_tid` set from a stale metadata-head read,
then the metadata page later published the new head unconditionally. Two
concurrent insert backends could both prepend against the same old head, and the
last metadata writer could orphan the other group from full-chain walks.

This slice changes insert to:

- append the new packed rerank group unpublished, with `next_group_tid = INVALID`;
- keep the appended header tuple in insert state;
- during the final metadata update, hold the metadata page exclusive lock,
  observe the current `rerank_sidecar_head`, rewrite the new group header so
  `next_group_tid` points at that current head, and publish the new head in the
  same WAL transaction.

Hot-path posting-carried direct TIDs are unchanged.

## Non-Claims

- This is not a benchmark packet.
- This does not add a synthetic concurrent pgrx test. The focused PG18 fixture
  covers that inserted index-side packed groups remain visible and scorable
  after the relink path change.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo check --no-default-features --features pg18` passed.
- `cargo pgrx test pg18 test_ec_ivf_index_placement_insert_maintains_packed_group`
  passed.
