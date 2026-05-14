# Artifact Manifest: 31027 SPIRE diagnostics storage roundtrip fixture split

Head SHA: `622a106c5dc217ba8894bfba86d08978243fa16f`

Packet/topic: `31027-spire-diagnostics-storage-roundtrip-fixture-split`

Timestamp: `2026-05-14T02:36:45Z`

Surface note: this packet moves Rust test fixtures only. No benchmark lane,
fixture corpus, storage format, rerank mode, or isolated/shared index surface
applies.

## Artifacts

### `cargo-fmt-check.log`

- Command: `script -q -e -c 'cargo fmt --check' review/31027-spire-diagnostics-storage-roundtrip-fixture-split/artifacts/cargo-fmt-check.log`
- Key result: command exited `0`.

### `git-diff-check.log`

- Command: `script -q -e -c 'git diff --check -- plan/tasks/task30-phase12b-spire-cleanup.md src/tests/mod.rs src/tests/diagnostics.rs' review/31027-spire-diagnostics-storage-roundtrip-fixture-split/artifacts/git-diff-check.log`
- Key result: command exited `0`.

### `line-counts.log`

- Command: `script -q -e -c 'wc -l src/tests/mod.rs src/tests/diagnostics.rs' review/31027-spire-diagnostics-storage-roundtrip-fixture-split/artifacts/line-counts.log`
- Key result lines:
  - `24517 src/tests/mod.rs`
  - `1658 src/tests/diagnostics.rs`

### `location-check.log`

- Command: `script -q -e -c 'rg -n "fn test_ec_spire_relation_object_tuple_roundtrip|fn test_ec_spire_relation_leaf_v2_roundtrip|fn test_ec_spire_empty_manifest_publish_roundtrip|fn test_ec_spire_|fn test_pg18_ec_spire|include!\\(" src/tests/mod.rs src/tests/diagnostics.rs' review/31027-spire-diagnostics-storage-roundtrip-fixture-split/artifacts/location-check.log`
- Key result lines:
  - `src/tests/diagnostics.rs:1579: fn test_ec_spire_relation_object_tuple_roundtrip()`
  - `src/tests/diagnostics.rs:1603: fn test_ec_spire_relation_leaf_v2_roundtrip()`
  - `src/tests/diagnostics.rs:1625: fn test_ec_spire_empty_manifest_publish_roundtrip()`
  - `src/tests/mod.rs` reports only concern-file `include!(...)` lines in this check.

### `pg18-test-relation-object-tuple-roundtrip.log`

- Command: `script -q -e -c 'cargo pgrx test pg18 test_ec_spire_relation_object_tuple_roundtrip' review/31027-spire-diagnostics-storage-roundtrip-fixture-split/artifacts/pg18-test-relation-object-tuple-roundtrip.log`
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 39.17s`

### `pg18-test-relation-leaf-v2-roundtrip.log`

- Command: `script -q -e -c 'cargo pgrx test pg18 test_ec_spire_relation_leaf_v2_roundtrip' review/31027-spire-diagnostics-storage-roundtrip-fixture-split/artifacts/pg18-test-relation-leaf-v2-roundtrip.log`
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 33.58s`

### `pg18-test-empty-manifest-publish-roundtrip.log`

- Command: `script -q -e -c 'cargo pgrx test pg18 test_ec_spire_empty_manifest_publish_roundtrip' review/31027-spire-diagnostics-storage-roundtrip-fixture-split/artifacts/pg18-test-empty-manifest-publish-roundtrip.log`
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 32.54s`
