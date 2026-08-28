---
task: 230
packet: 002-format-and-read-path
agent: Codex
role: coder
model: gpt-5
date: 2026-08-28
seq: 04
---

# Task 230 packet 002 — Graph V2 locator trailer

Review code checkpoints `3102e28ef`, `9b13d2aca`, and test-only follow-up
`7b8edce68` against the review-closed
packet-001 contract and reviewer seq-02, which accepted the descriptor
foundation and authorized this Graph V2 slice. This remains a narrow
packet-002 checkpoint: it defines the versioned graph bytes and dispatch API,
but does not yet create hot/cold relations or switch generation callers to V2.

## Graph V2 scope

- Adds physical graph-record version 2 with `cold_tid` appended after the V1
  search code, neighbor IDs, and neighbor codes. Every existing V1 field offset
  and the complete V1 length remain unchanged.
- Adds explicit version-sized length calculation plus versioned encode,
  decode, and pooled-decode dispatch. The versioned path reads and admits the
  first two bytes before applying the selected version's length check.
- Leaves legacy tag/reserved `decode`/`decode_into` V1-sized and unchanged.
  Existing legacy and physical-V1 writers must carry `cold_tid=None`; V1 cannot
  silently discard a cold locator.
- Requires valid hot and cold owner-local TIDs for V2 and preserves canonical
  adjacency-padding validation.
- Adds `distann_graph_record_v2.hex` plus an independent fixture decoder that
  walks every field, compares the bytes after the version through the V1 end
  with the frozen V1 fixture, and reads the six-byte trailer.
- Exports the V2 format constant, trailer size, and offset helper through the
  existing benchmark/test API.

The follow-up `9b13d2aca` replaces `Option::is_none_or` with an equivalent
Rust-1.75-compatible expression after the exact clippy gate caught the MSRV
violation. No history was rewritten.

## Seq-03 disposition

- A valid `cold_tid` is now explicitly rejected by both the physical-V1 writer
  and the legacy writer. Deleting either guard makes the new test fail.
- The public pooled physical-version decoder is now exercised by decoding V2
  into a reused tuple, observing its cold locator, then decoding V1 into the
  same tuple and proving the locator is cleared rather than retained.
- Both tests share the `distann_physical_node` prefix, so the packet's focused
  command executes all five Graph V2 tests.

## Prior descriptor slice

Reviewer seq-02 accepted code `8faac4bad` as DONE for the descriptor slice. Its
seq-01 disposition below remains for history.

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

Packet-local seq-04 output and provenance are recorded in
`artifacts/manifest.md`:

- five focused PG18 physical-node version tests pass;
- two independent V1/V2 golden-fixture tests pass;
- formatting passes (`cargo fmt --all -- --check`);
- the all-target PG18 clippy command introduces no error in a touched file and
  records only the five pre-existing repository failures already identified in
  seq-02.

No PG relation or callback behavior changes in this slice, so no pgrx cluster
test is claimed. Full format/read-path PG18 coverage remains required before
this packet closes.

## Review request

Please rereview the two seq-03 test gaps at `7b8edce68`: V1 and legacy writers
reject a valid cold locator, and pooled physical-version decode populates a V2
locator then clears it on V1 reuse. If DONE, the next slice will bind descriptor
V4/layout identity and switch only version-aware generation callers to V2;
legacy tag-guarded `expand.rs`, `reader.rs`, and `insert.rs` paths remain V1.
