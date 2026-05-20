# Task 39 leaf-V2 meta validate() coverage

## Summary

Adds direct coverage for the nine `SpireLeafPartitionObjectV2Meta::
validate()` error branches in
`src/am/ec_spire/storage/leaf_v2_parts.rs` that previously only had
indirect (success-path) coverage.

The pre-existing `miri_leaf_v2_empty_meta_rejects_segment_locator`
test exercised one error path (empty meta + non-INVALID locator).
This slice closes the remaining nine error branches in `validate()`:

- `published_epoch_backref == 0` (line 121)
- `object_bytes_total == 0` (line 124)
- LocalU64 with wrong `vec_id_stride` (line 129)
- GlobalBytes with stride below the 2-byte minimum (line 138)
- GlobalBytes with stride above `SPIRE_VEC_ID_MAX_BYTES` (line 138)
- non-empty meta with `segment_count == 0` (line 154)
- non-empty meta with INVALID `first_segment_locator` (line 157)
- non-empty meta with `payload_format == NONE` (line 160)
- non-empty meta with `payload_stride == 0` (line 165)

All sub-asserts isolate exactly one twisted field on top of a
known-good baseline (a sanity-check `is_ok()` on the unmodified
args pins that the baseline itself is valid).

This is a narrow follow-up slice in the Task 39 coverage burndown
(handoff identifies `am/ec_spire/storage/leaf_v2_parts.rs below 80%`).

## Code under review

- Commit: `4c2f13b11ce094fb211bc7d08edfeb6d7cbf9d4a`
- Changed file: `src/am/ec_spire/storage/tests/leaf.rs`

## Validation

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib
  careful_spire::storage::tests::miri`: 21 passed (up from 20 in
  packet 028). Artifact:
  `artifacts/leaf-v2-focused-tests.log`.

## Notes

- Test is MIRI-friendly (no `unsafe`, no pgrx interaction) so it
  flows into the hardening MIRI lane.
- Two `validate()` branches remain uncovered by this test because
  they require integer-overflow inputs (`segment_count u32::try_from`
  in `SpireLeafPartitionObjectV2::validate`, `assignment_count
  checked_add` in the same loop). Those are deferred — building a
  Vec of `u32::MAX + 1` segments is impractical in a unit test.
