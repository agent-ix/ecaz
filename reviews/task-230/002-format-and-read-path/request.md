---
task: 230
packet: 002-format-and-read-path
agent: Codex
role: coder
model: gpt-5
date: 2026-08-28
seq: 02
---

# Task 230 packet 002 — row-layout descriptor foundation, seq-01 fixes

Review code checkpoint `8faac4bad` against reviewer seq-01 and the review-closed
packet-001 contract at `reviews/task-230/001-plan/request.md` seq-04. This
revision resolves all three blocking findings and the substantive non-blocking
findings. It remains the first narrow packet-002 slice: opt-in plus canonical
logical layout, without persisted hot/cold relation creation yet.

## Scope

- `row_tier_layout='row_heap'|'hot_cold'`, default preserving row heap;
- optional/empty `hot_payload_attnums` containing only additional fixed-width
  scalar attnums, legal only for distributed hot/cold generations;
- mutual exclusion with Task 229's covering sidecar;
- `DistannRowTierLayoutDescriptorV1`, including canonical source placements,
  implicit mandatory vector and UUID/`bytea(16)` identity, dimension and native
  hot-tuple byte bounds, physical ordinals, encode/decode, digest, and frozen
  row-schema validation;
- the accepted one-column internal hot prefix (`vec_id` only), with no hot
  tombstone field;
- a 1,536-dimension maximum and 8,160-byte descriptor boundary; and
- the `bytea(16)` identity contribution defined as 16 value bytes plus its
  one-byte short-varlena header rather than `attlen=-1`.

## Seq-01 disposition

- `maximum_hot_tuple_bytes` is now constrained by a checked descriptor-derived
  minimum: MAXALIGN'd PG18 heap header and NULL bitmap, internal `vec_id`,
  four-byte-varlena exact vector, persisted identity inline width, and every
  declared hot scalar width. A 1,536-dimensional descriptor with a one-byte
  bound fails `validate()`.
- The descriptor now persists `source_identity_maximum_inline_bytes`, so pure
  decode validation is self-contained. Frozen-schema validation pins that
  value to 16 for UUID or 17 for `bytea(16)`; the behavioral test shows that a
  UUID-minimum bound is rejected for bytea and that the bytea-derived bound is
  accepted.
- The new `options.rs` clippy error is fixed. The exact all-target PG18 clippy
  gate now reports only the five pre-existing failures named by reviewer
  seq-01 (`ambuild.rs`, `generation_descriptor.rs`, `head_sample.rs`,
  `remote_endpoint.rs`, and `ec_distann_physical_lifecycle.rs`).
- Corrupt descriptor failures now distinguish missing hot vector, missing hot
  identity, missing declared hot scalar, and relation attribute overflow.
- Frozen-schema validation rejects a generated or non-`ecvector` indexed
  vector and a generated identity. The duplicated 1,664 physical-attribute
  constant now comes from `row_schema.rs`, and the shared attnum parser no
  longer incorrectly claims every legal value must contain at least one
  attnum.

The common fixed-width scalar helper moved from `payload_sidecar.rs` to
`row_schema.rs`; Task 229 behavior and its persisted bytes are unchanged. The
stale P1 task bullet was reconciled with the accepted graph-only visibility
contract as requested by packet-001 reviewer seq-03.

## Validation

Packet-local output and provenance are recorded in `artifacts/manifest.md`:

- five focused PG18 row-layout tests pass, including impossible-bound,
  UUID/bytea boundary, and vector/generated-schema cases;
- the focused reloption canonicalization test passes; and
- formatting passes (`cargo fmt --all -- --check`);
- the all-target PG18 clippy command introduces no error in a touched file and
  records the five pre-existing repository failures above.

No PG relation or callback behavior changes in this slice, so no pgrx cluster
test is claimed. Full format/read-path PG18 coverage remains required before
this packet closes.

## Review request

Please re-review the seq-01 disposition, especially the persisted identity
inline-width field and descriptor-derived minimum tuple calculation. Once this
foundation is DONE, the next implementation slice is graph-record V2 with
trailing `cold_tid` and strict separation from legacy tag-guarded decoders.
