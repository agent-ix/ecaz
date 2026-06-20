# Task 111h / 011 Rerank Stage Timing Counters

## Summary

This packet requests review for commit
`7a58a882565523e13538aab526221fb104a0234f`
(`task111h: split rerank payload stage timing`).

The slice addresses the timing-counter gap called out in the 008 review:
`Exact Rerank Elapsed Us` remains as the coarse total, but IVF EXPLAIN/debug
counters now also expose:

- `Rerank Payload Decode Elapsed Us`
- `Rerank Payload Score Elapsed Us`

Source placement records heap-row fetch plus source-vector extraction as
payload decode time, and records only the exact scorer call as payload score
time. Index placement records packed group load plus payload slice lookup/batch
slab materialization as payload decode time, and records scalar or batched
sidecar scoring as payload score time.

## Notes

- This does not claim to finish the full checklist item for all admin/benchmark
  coverage. It only fills the EXPLAIN/debug counter timing split for decode and
  scoring.
- The f16 scalar index path keeps its no-owned-payload-copy shape; timing is
  recorded per candidate instead of introducing a temporary payload reference
  vector.
- No durable layout or format-version change is involved.
- No benchmark claims are made in this packet.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo test --no-default-features --features pg18 ivf_explain --lib`
  passed: 2 tests.
- `cargo check --no-default-features --features pg18` passed.
- `cargo pgrx test pg18 test_ec_ivf_index_placement_fewer_rerank_bytes`
  passed: 1 PG18 fixture.
