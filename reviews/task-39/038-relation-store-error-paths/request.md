# Task 39 / 038 — Relation Store Codec Error Paths

## Goal

Direct unit coverage for the private chain meta and chain segment
codecs in `relation_store.rs`. Packet 037 round-tripped both via real
inserts; this packet pins each `Err(...)` branch in
`encode_relation_object_chain_meta`,
`decode_relation_object_chain_meta`,
`encode_relation_object_chain_segment`, and
`decode_relation_object_chain_segment`, plus the
`max_relation_object_tuple_payload_bytes` and
`max_partition_object_chain_segment_payload_bytes` helpers.

## Code Change

`hardening/careful/src/spire.rs` — 3 new tests in `storage::tests`:

- `relation_store_chain_meta_codec_round_trip_and_error_branches`
- `relation_store_chain_segment_codec_round_trip_and_error_branches`
- `relation_store_max_segment_payload_bytes_is_positive`

## Baseline Ratchet

| File | Pre-packet | This packet |
| --- | ---: | ---: |
| `am/ec_spire/storage/relation_store.rs` | 56.53 | **58.10** |

`relation_store.rs` is still short of the 80% floor. The remaining
~22% is dominated by:

- `with_single_tuple_object_bytes` length-mismatch and decode-error
  branches that can only be reached by crafting raw tuple bytes
  inconsistent with placement metadata.
- `read_large_partition_object_bytes` segment-no, byte-base, and
  length-mismatch branches (same constraint).
- `active_object_tuple_locators` chain-missing / trailing-locator /
  meta-mismatch branches for leaf V2 chains.
- `prefetch_relation_blocks_with_read_stream` PG18 implementation
  (the emulator's read-stream is a no-op so the inner loop is never
  entered).

Closing those remaining branches is mechanical but requires either a
test-only helper that writes raw tuple bytes through the page emulator
or a richer read-stream emulator. Tracked as the next slice if
reviewer asks for the full 80%.

## Validation

Artifacts under
`reviews/task-39/038-relation-store-error-paths/artifacts/`:

- `relation-store-error-paths-focused-tests.log` — **500 passed**.
- `coverage/summary.txt` + JSON files from `make coverage`.
- `coverage-delta-check.log` — green at new baseline.
- `coverage-baseline-check.log` — 40 critical paths complete.
- `changed-files.txt` — single ratcheted path.
