# Review Request: DiskANN RaBitQ Metadata Compatibility Test

## Scope

This checkpoint adds explicit regression coverage for the Task 60 backwards-compatibility gate.

The new test decodes a byte-level V3 DiskANN metadata image representing an existing grouped-PQ / `pq_fastscan` index and asserts that the fields needed by the legacy scan path are preserved:

- `search_codec_kind = VAMANA_SEARCH_CODEC_GROUPED_PQ`
- grouped-PQ and binary-sidecar payload flags
- grouped search shape
- entry point and codebook chain pointer

The fixture is built as raw encoded metadata bytes instead of using the current encoder as the source of truth.

## Validation

Artifacts are under `reviews/task-60/007-diskann-rabitq-compat-metadata/artifacts/`.

- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18` passed.
- `cargo-test-diskann-page-compat.log`: the focused test compiled, then the local pgrx-linked lib test binary failed before executing with `undefined symbol: CacheRegisterRelcacheCallback`.

## Remaining Task 60 Gate

This strengthens the local compatibility coverage but does not close Task 60. The full 100k/1M benchmark evidence is still required to prove recall parity and the 1M storage reduction gate.
