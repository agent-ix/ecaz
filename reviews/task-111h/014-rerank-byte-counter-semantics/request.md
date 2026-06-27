# Task 111h / 014 Rerank Byte Counter Semantics

## Summary

This packet requests review for commit
`17cb6f51a813f5f55b6d1448d3408d02ccedc502`
(`task111h: separate source and payload byte counters`).

The 008 feedback noted that `Rerank Payload Bytes Scored` duplicated
`Rerank Source Bytes Read` on index placement. This slice makes the counters
distinct:

- `Rerank Source Bytes Read` now means bytes read from the heap source-vector
  column. It is nonzero for source f32 rerank and remains zero for persisted
  index-side compact rerank.
- `Rerank Payload Bytes Scored` now means compact payload bytes handed to the
  scorer for the survivor frontier.
- Physical packed-group payload reads remain split across
  `Rerank Index Header Payload Bytes Read` and
  `Rerank Index Segment Payload Bytes Read`.

The focused PG18 fixtures now assert zero source-column bytes for index-side
f16/RaBitQ-4/RaBitQ-8 while still asserting nonzero compact payload bytes scored
and the existing batched slab-copy accounting.

## Non-Claims

- This is not a benchmark packet.
- This does not remove the batched compact-format slab copy; it makes the
  remaining copy cost and byte accounting clearer.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo check --no-default-features --features pg18` passed.
- `cargo pgrx test pg18 test_ec_ivf_index_placement` passed five PG18 fixtures.
