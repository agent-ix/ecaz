# Task 111h / 013 Rerank Placement Wording

## Summary

This packet requests review for commit
`a825611074c36046b939b9d76e46cb7bb91dd248`
(`task111h: clarify reserved rerank table placement`).

The slice addresses the stale wording noted in the 008 feedback:

- `rerank_placement` reloption help now marks `table` as reserved rather than
  advertising it like an implemented placement.
- `docs/on-disk-format.md` no longer describes legacy v4 `0x2A` as a Task 111h
  format. It describes it as legacy compact rerank payload bytes and keeps the
  v5 `0x2B`/`0x2C` packed-group layout as the current format.

## Non-Claims

- This is not a benchmark packet.
- This does not implement table-owned persisted rerank payload storage.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo check --no-default-features --features pg18` passed.
