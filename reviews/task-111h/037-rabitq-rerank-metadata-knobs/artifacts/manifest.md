# Artifact Manifest

Task bucket: `reviews/task-111h/`
Packet: `reviews/task-111h/037-rabitq-rerank-metadata-knobs/`
Head SHA: `88f10283386b55c7128da4c350e654ed901dc14c`
Code commit under review: `88f10283386b55c7128da4c350e654ed901dc14c`
Timestamp: `2026-06-20T14:55:14-07:00`
Branch: `bench-ivf-111g-115-attribution`

## Scope

This packet covers the follow-up to packet 035 reviewer feedback: RaBitQ compact
rerank score knobs (`rabitq_rerank_least_squares` and `rabitq_rerank_clip`) are
now persisted in IVF metadata instead of being read from mutable live reloptions
during scan/insert. The persisted layout bumps IVF metadata format version from
7 to 8 while keeping the 92-byte metadata width.

No benchmark matrix was run for this packet. This is a correctness and format
contract change over the already measured packet 036 score/clip A/B.

## Commands

All commands ran from `/tmp/ecaz-111h-clean` against head
`88f10283386b55c7128da4c350e654ed901dc14c`.

```text
cargo test --no-default-features --features pg18 metadata_roundtrips_rabitq_rerank_score_knobs --lib
cargo test --no-default-features --features pg18 --test on_disk_fixtures ivf_metadata
cargo test --no-default-features --features pg18 --test size_of_assertions
cargo test --no-default-features --features pg18 --test upgrade_matrix
cargo check --no-default-features --features pg18
```

## Artifact Inventory

- `artifacts/cargo-test-metadata-knobs.log`
- `artifacts/cargo-test-on-disk-ivf-metadata.log`
- `artifacts/cargo-test-size-of-assertions.log`
- `artifacts/cargo-test-upgrade-matrix.log`
- `artifacts/cargo-check-pg18.log`

## Key Result Lines

- `cargo-test-metadata-knobs.log`: `test am::ec_ivf::page::tests::metadata_roundtrips_rabitq_rerank_score_knobs ... ok`; `1 passed; 0 failed`.
- `cargo-test-on-disk-ivf-metadata.log`: `test ivf_metadata_v8_fixture_decodes ... ok`; `test ivf_metadata_v7_is_rejected_by_version ... ok`; `7 passed; 0 failed`.
- `cargo-test-size-of-assertions.log`: `13 passed; 0 failed`.
- `cargo-test-upgrade-matrix.log`: `2 passed; 0 failed`.
- `cargo-check-pg18.log`: `Finished dev profile`.

## Layout Summary

- `EC_IVF_INDEX_FORMAT_VERSION = 8`
- `EC_IVF_METADATA_BYTES = 92`
- `EC_IVF_METADATA_RABITQ_RERANK_SCORE_MODE_OFFSET = 22`
- `EC_IVF_METADATA_RABITQ_RERANK_CLIP_OFFSET = 23`
- `fixtures/on-disk/ivf_metadata_v8.hex` is the current readable/writable IVF
  metadata fixture.
- `fixtures/on-disk/ivf_metadata_v7.hex` remains committed as a rejected legacy
  fixture.
