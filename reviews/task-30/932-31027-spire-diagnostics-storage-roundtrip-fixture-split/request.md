# Review Request: SPIRE diagnostics storage roundtrip fixture split

## Summary

This checkpoint moves the last direct SPIRE fixture bodies out of
`src/tests/mod.rs` and into the diagnostics concern file.

Moved fixtures:

- `test_ec_spire_relation_object_tuple_roundtrip`
- `test_ec_spire_relation_leaf_v2_roundtrip`
- `test_ec_spire_empty_manifest_publish_roundtrip`

`src/tests/mod.rs` now retains shared helper functions and concern-file
includes, but no direct `test_ec_spire_*` or `test_pg18_ec_spire_*` fixture
definitions. The Phase 12b test module-tree tracker row is marked closed.

Code checkpoint: `622a106c5dc217ba8894bfba86d08978243fa16f`

## Validation

- `cargo fmt --check`
- `git diff --check`
- location check confirms the three moved fixtures now resolve in
  `src/tests/diagnostics.rs` and `src/tests/mod.rs` has only include lines
  for test fixture bodies
- focused PG18 checks:
  - `test_ec_spire_relation_object_tuple_roundtrip`: passed
  - `test_ec_spire_relation_leaf_v2_roundtrip`: passed
  - `test_ec_spire_empty_manifest_publish_roundtrip`: passed

Raw logs and command metadata are in `artifacts/manifest.md`.

## Reviewer Focus

- Confirm moving the storage roundtrip fixtures to diagnostics is the right
  concern boundary.
- Confirm fixture names and assertions are unchanged.
- Confirm `src/tests/mod.rs` is now an include/helper scaffold rather than a
  direct SPIRE fixture sink.
