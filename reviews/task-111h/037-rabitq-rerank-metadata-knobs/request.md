# Task 111h / Packet 037 Review Request: RaBitQ Rerank Metadata Knobs

## Summary

This packet requests review for commit
`88f10283386b55c7128da4c350e654ed901dc14c`, which addresses the packet 035
review finding that `rabitq_rerank_least_squares` and `rabitq_rerank_clip` were
build-time payload interpretation knobs but were still read from mutable live
reloptions during scan and aminsert.

The fix persists those knobs in IVF metadata:

- byte `22`: RaBitQ rerank score mode (`0 = estimator`, `1 = least_squares`)
- byte `23`: RaBitQ rerank clip (`1..=8`)
- IVF metadata format version bumps from `7` to `8`
- v7 remains as a committed legacy fixture and is rejected by version

## Code Changes

- `src/am/ec_ivf/page.rs` writes/reads the new metadata bytes, exposes helper
  methods for the internal scorer enum, and rejects invalid persisted values.
- `src/am/ec_ivf/scan.rs` overwrites the live reloption-derived scorer knobs
  with metadata-backed values immediately after reading the metadata page.
- `src/am/ec_ivf/insert.rs` uses metadata-backed scorer knobs when deriving
  bootstrap build options and resolving the compact rerank sidecar encoder.
- `fixtures/on-disk/ivf_metadata_v8.hex`, `fixtures/upgrade/matrix.csv`,
  `tests/on_disk_fixtures.rs`, `tests/size_of_assertions.rs`, and
  `tests/upgrade_matrix.rs` advance the current IVF format contract to v8.
- `docs/on-disk-format.md` documents the v8 layout and the v7 rejection reason.

## Validation

Packet-local logs are listed in `artifacts/manifest.md`.

```text
cargo test --no-default-features --features pg18 metadata_roundtrips_rabitq_rerank_score_knobs --lib
cargo test --no-default-features --features pg18 --test on_disk_fixtures ivf_metadata
cargo test --no-default-features --features pg18 --test size_of_assertions
cargo test --no-default-features --features pg18 --test upgrade_matrix
cargo check --no-default-features --features pg18
```

All passed.

## Review Ask

Please review:

- whether v8 is the right format boundary for this field addition,
- whether scan and insert now consistently ignore ALTERed live reloptions for
  RaBitQ rerank score/clip interpretation,
- whether the byte placement at 22/23 is acceptable given the existing 92-byte
  metadata width,
- whether any additional docs or ALTER guard is still needed now that the
  payload interpretation values are metadata-backed.

## Non-Claims

This packet does not claim a new performance result. Packet 036 remains the
benchmark evidence for the RaBitQ8 score/clip A/B; this packet only fixes the
durability/ALTER footgun around those knobs.
