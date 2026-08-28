---
task: 230
packet: 002-format-and-read-path
agent: Codex
role: coder
model: gpt-5
date: 2026-08-28
seq: 01
---

# Task 230 packet 002 — row-layout descriptor foundation

Review code checkpoint `ef558a669` against the review-closed packet-001
contract at `reviews/task-230/001-plan/request.md` seq-04. This is the first
narrow packet-002 slice; it defines the opt-in and canonical logical layout but
does not yet create or write persisted hot/cold relations.

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

The common fixed-width scalar helper moved from `payload_sidecar.rs` to
`row_schema.rs`; Task 229 behavior and its persisted bytes are unchanged. The
stale P1 task bullet was reconciled with the accepted graph-only visibility
contract as requested by packet-001 reviewer seq-03.

## Validation

Packet-local output and provenance are recorded in `artifacts/manifest.md`:

- five focused PG18 row-layout tests pass;
- the focused reloption canonicalization test passes; and
- formatting passes (`cargo fmt --all -- --check`).

No PG relation or callback behavior changes in this slice, so no pgrx cluster
test is claimed. Full format/read-path PG18 coverage remains required before
this packet closes.

## Review request

Please review the implicit-identity contract, empty additional-hot set,
canonical partition/ordinal validation, dimension and tuple bounds, bytea short-
varlena accounting, reloption mutual exclusion, and Task 229 byte compatibility.
The next implementation slice is graph-record V2 with trailing `cold_tid` and
strict separation from legacy tag-guarded decoders.
