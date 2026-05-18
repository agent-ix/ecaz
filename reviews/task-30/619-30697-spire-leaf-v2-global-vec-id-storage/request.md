# Review Request: SPIRE Leaf V2 Global Vector ID Storage

Status: open
Owner: coder1
Head SHA: `597c8f998542b49964433f2bf98967311c1dbbf1`

## Summary

This Phase 11.2 slice removes the Leaf V2 base-object blocker for global
`0x02` vector IDs by activating the existing `vec_id_kind` and
`vec_id_stride` meta fields.

Key changes:

- Leaf V2 now supports `GlobalBytes` vector-ID columns when every row in the
  object uses global `0x02` bytes with the same encoded length.
- Existing local `0x01` rows keep the 16-byte padded `LocalU64` format.
- Mixed local/global rows and variable-width global rows inside one Leaf V2
  object are rejected with explicit errors.
- Leaf V2 assignment reconstruction and quantized scan candidate collection
  now decode vector IDs through the column meta instead of rebuilding local
  IDs from `local_vec_seq`.
- The SQL-visible vector identity contract now reports
  `writer_global_base_storage = phase11_2_landed`.
- Added `plan/design/spire-leaf-v2-vector-id-layout.md` to pin the storage
  decision and limitations.

## Deliberate Limits

This does not yet make live build/insert writers emit global IDs. The remaining
Phase 11.2 blocker is the stable source-identity input contract, which must
produce fixed-width global payloads when targeting Leaf V2 base objects.

This also does not support mixed namespaces or variable-width global payloads
inside one Leaf V2 object. That needs a future length-prefixed format if we
decide it is necessary.

## Validation

- `cargo test leaf_v2 --lib`
  - `test tests::pg_test_ec_spire_relation_leaf_v2_roundtrip ... ok`
- `cargo test leaf_partition_object_v2 --lib`
  - 4 passed, 0 failed
- `cargo test global --lib`
  - 14 passed, 0 failed
- `cargo test collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer --lib`
  - 1 passed, 0 failed
- `cargo test remote_search_final_contract --lib`
  - `test tests::pg_test_ec_spire_remote_search_final_contract ... ok`
- `cargo fmt --check`
  - passed; rustfmt still prints the existing stable-toolchain warnings for
    unstable import-grouping settings
- `git diff --check`
  - passed

## Review Focus

- Is the fixed-width `GlobalBytes` use of Leaf V2 acceptable for Phase 11.2, or
  should this have been a new storage version immediately?
- Did any read path still reconstruct local IDs instead of preserving global
  bytes?
- Are the rejected mixed/variable layouts the right production constraint for
  the next writer source-identity slice?
