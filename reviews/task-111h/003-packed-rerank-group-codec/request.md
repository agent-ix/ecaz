# Task 111h / 003 - Packed Rerank Group Page Codec Checkpoint

Code commit: `0738a3a2f8e0d725e750e48f5282dcc6ff7ea8dc`

## Summary

This checkpoint adds the page-level codec for the Task 111h packed index-side
rerank group layout. It is deliberately limited to tuple encode/decode and page
capacity helpers; the build and scan paths still use the legacy `0x2A` direct
TID sidecar from checkpoint 002.

What changed:

- Added `0x2B` rerank group header tuple support. The header stores logical
  group metadata once: rerank format, list id, scorer width, valid count,
  payload length, total heap TID count, total payload byte count, header payload
  fragment byte count, continuation TID, deleted bitmap, gammas, heap TID
  counts/offsets, payload offsets, heap TIDs, and the first payload fragment.
- Added `0x2C` rerank group payload segment tuple support. Continuation
  segments store only payload bytes plus a next-segment TID.
- Added validation for invalid group metadata, including empty live heap-TID
  ranges, out-of-bounds heap/payload offsets, invalid counts, payload length
  mismatch, and nonzero reserved header bytes.
- Added page-capacity helpers for header and payload segment tuples.
- Added unit coverage for header roundtrip with a partial payload fragment,
  payload segment roundtrip, invalid metadata rejection, reserved-byte
  rejection, and capacity helper behavior.

This does **not** yet satisfy the Task 111h checklist item "Implement the packed
index-side rerank group/segment layout." That remains open until the index build,
insert, delete/vacuum, scan, and metrics paths write and read these tuples and
the durable format version/docs/fixtures are updated as required by NFR-016.

## Code Changes

- `src/am/ec_ivf/page.rs`: adds the packed rerank group header and payload
  segment tuple codecs, fit helpers, and focused unit tests.

No benchmark matrix is included in this packet. This is a narrow page-codec
checkpoint, not a performance or product-decision packet.

## Validation

Artifacts are under `reviews/task-111h/003-packed-rerank-group-codec/artifacts/`.

- `cargo test --no-default-features --features pg18 rerank_group --lib`
  passed: `3 passed; 0 failed`.
- `cargo test --no-default-features --features pg18 layout_fit_helpers_track_page_capacity --lib`
  passed: `1 passed; 0 failed`.
- `cargo check --no-default-features --features pg18` passed.

## Review Focus

- Confirm the `0x2B` header stores scorer-width logical group metadata once and
  can support payload-heavy continuation segments without repeating arrays.
- Confirm the fixed header fields and validation are sufficient before build/scan
  code starts emitting this layout.
- Confirm it is acceptable that this checkpoint does not bump the IVF format
  version because no durable writer path emits `0x2B`/`0x2C` tuples yet. The
  version bump is expected when the layout is wired into persisted build/insert
  output.
