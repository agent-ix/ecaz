# Task 111h Packet 022 Artifact Manifest

- head SHA: `7112caeae6ac5bfc659253e823d13a3f31f64b2e`
- branch: `bench-ivf-111g-115-attribution`
- task bucket: `reviews/task-111h/022-rabitq-residual-rerank`
- captured at: `2026-06-20T13:13:54Z`
- lane: local Rust correctness tests
- fixture / storage format / rerank mode: IVF v6; index-side RaBitQ4/RaBitQ8 rerank sidecar residual payloads
- surface isolation: not applicable; no database benchmark or corpus load in this packet

## Commands

Focused residual RaBitQ sidecar unit test:

```sh
script -q -e -c "cargo test --no-default-features --features pg18 index_side_rabitq_payloads_require_centroid_and_apply_correction" reviews/task-111h/022-rabitq-residual-rerank/artifacts/cargo-test-rabitq-residual-sidecar.log
```

Payload codec tests:

```sh
script -q -e -c "cargo test --no-default-features --features pg18 payload_codecs" reviews/task-111h/022-rabitq-residual-rerank/artifacts/cargo-test-payload-codecs.log
```

IVF metadata fixture tests:

```sh
script -q -e -c "cargo test --no-default-features --features pg18 ivf_metadata_" reviews/task-111h/022-rabitq-residual-rerank/artifacts/cargo-test-ivf-metadata.log
```

Upgrade matrix tests:

```sh
script -q -e -c "cargo test --no-default-features --features pg18 --test upgrade_matrix" reviews/task-111h/022-rabitq-residual-rerank/artifacts/cargo-test-upgrade-matrix.log
```

Rerank group lookup tests:

```sh
script -q -e -c "cargo test --no-default-features --features pg18 rerank_group_payload_lookup" reviews/task-111h/022-rabitq-residual-rerank/artifacts/cargo-test-rerank-group-lookup.log
```

## Artifact Index

- `artifacts/cargo-test-rabitq-residual-sidecar.log`: focused test for
  index-side RaBitQ sidecars requiring centroid context and applying centroid
  score correction at scan time.
- `artifacts/cargo-test-payload-codecs.log`: payload codec unit tests.
- `artifacts/cargo-test-ivf-metadata.log`: IVF metadata fixture tests covering
  current v6 and rejected legacy versions.
- `artifacts/cargo-test-upgrade-matrix.log`: upgrade matrix tests.
- `artifacts/cargo-test-rerank-group-lookup.log`: rerank group payload lookup
  tests.

## Key Result Lines

```text
cargo-test-rabitq-residual-sidecar.log:
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2205 filtered out; finished in 0.00s

cargo-test-payload-codecs.log:
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2204 filtered out; finished in 0.08s

cargo-test-ivf-metadata.log:
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 45 filtered out; finished in 0.00s

cargo-test-upgrade-matrix.log:
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo-test-rerank-group-lookup.log:
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2204 filtered out; finished in 0.00s
```

## Notes

This packet does not contain new latency, recall, or storage benchmark results.
The pre-existing Task 111h compressed rerank benchmark packets were produced
before this residual RaBitQ sidecar fix, so they should not be used as
post-fix RaBitQ4/RaBitQ8 evidence.
