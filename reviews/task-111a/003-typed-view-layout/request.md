# Review Request: Task 111a Typed Dense Layout

## Summary

This checkpoint implements the aligned little-endian dense posting typed-view
building block requested in the latest Task 111a feedback.

Code commit under review:

- `8f3979f8e Task 111a: add dense typed view layout`

The slice keeps existing dense formats gated/default-off and adds:

- a new one-page aligned dense block tag (`0x28`) that stores typed numeric
  arrays before variable byte fields;
- aligned/native LE accessors for dense block and packed header gammas,
  heap-tid counts, and heap-tid offsets, with runtime alignment and endian
  guards plus existing byte-decoding fallback;
- a `dense_posting_typed_layout` reloption for one-page aligned layout writes;
- a diagnostic `ec_ivf.dense_posting_typed_views` GUC to disable runtime typed
  reads while leaving the durable layout unchanged;
- scan wiring so per-block scoring can pass a typed gamma view directly to the
  scorer, while coalesced gather and packed group assembly use native typed
  reads when available;
- packed logical-group headers reordered to the aligned metadata-once layout,
  with continuation tuples still carrying payload bytes only.

## Scope Notes

This is not the final Task 111a benchmark gate. Approach A remains the baseline,
Approach B remains required, and the next benchmark packet still needs the full
matrix from the updated task: row, dense/current, dense+A, dense+typed,
dense+B, and dense+B+typed for TurboQuant and RaBitQ at real 50k/100k.

The current checkpoint is intended to make the typed-view format component
reviewable before that larger measurement pass.

## Validation

See `artifacts/manifest.md` for commands and outputs.

Passed locally:

- `cargo check -q --lib`
- `cargo test -q dense_posting_aligned_block_roundtrip_exposes_native_views --lib`
- `cargo test -q dense_posting_packed --lib`
- `cargo test -q build_state_splits_packed_dense_payloads_into_continuations --lib`
- `cargo test -q dense_posting_block_roundtrip_preserves_scan_arrays --lib`

No PG18 callback test or benchmark suite was run for this code checkpoint.
