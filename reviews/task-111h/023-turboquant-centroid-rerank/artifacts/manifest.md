# Task 111h Packet 023 Artifact Manifest

- head SHA: `728cc2ed9ee2b14e14b667c521390d04f3880526`
- branch: `bench-ivf-111g-115-attribution`
- task bucket: `reviews/task-111h/023-turboquant-centroid-rerank`
- captured at: `2026-06-20T13:28:00Z`
- lane: local Rust correctness tests
- fixture / storage format / rerank mode: IVF v7; index-side RaBitQ4, RaBitQ8,
  and TurboQuant rerank sidecar payloads are centroid-relative
- surface isolation: not applicable; no database benchmark or corpus load in
  this packet

## Commands

Focused centroid-relative sidecar unit test:

```sh
script -q -e -c "cargo test --no-default-features --features pg18 index_side_quantized_payloads_require_centroid_and_apply_correction" reviews/task-111h/023-turboquant-centroid-rerank/artifacts/cargo-test-centroid-relative-sidecar.log
```

Payload codec tests:

```sh
script -q -e -c "cargo test --no-default-features --features pg18 payload_codecs" reviews/task-111h/023-turboquant-centroid-rerank/artifacts/cargo-test-payload-codecs.log
```

IVF metadata fixture tests:

```sh
script -q -e -c "cargo test --no-default-features --features pg18 ivf_metadata_" reviews/task-111h/023-turboquant-centroid-rerank/artifacts/cargo-test-ivf-metadata.log
```

Upgrade matrix tests:

```sh
script -q -e -c "cargo test --no-default-features --features pg18 --test upgrade_matrix" reviews/task-111h/023-turboquant-centroid-rerank/artifacts/cargo-test-upgrade-matrix.log
```

Whitespace check:

```sh
script -q -e -c "git diff --check" reviews/task-111h/023-turboquant-centroid-rerank/artifacts/git-diff-check.log
```

## Artifact Index

- `artifacts/cargo-test-centroid-relative-sidecar.log`: focused unit test for
  index-side RaBitQ4, RaBitQ8, and TurboQuant sidecars requiring centroid
  context and applying centroid score correction.
- `artifacts/cargo-test-payload-codecs.log`: source-diagnostic payload codec and
  compact batch/scalar consistency tests.
- `artifacts/cargo-test-ivf-metadata.log`: IVF metadata fixture tests covering
  current v7 and rejected legacy v6/v5/v4/v3.
- `artifacts/cargo-test-upgrade-matrix.log`: upgrade matrix tests.
- `artifacts/git-diff-check.log`: `git diff --check` output.

## Key Result Lines

```text
cargo-test-centroid-relative-sidecar.log:
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2205 filtered out; finished in 0.08s

cargo-test-payload-codecs.log:
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2204 filtered out; finished in 0.08s

cargo-test-ivf-metadata.log:
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 45 filtered out; finished in 0.00s

cargo-test-upgrade-matrix.log:
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

git-diff-check.log:
exit 0, no output
```

## Notes

This packet does not contain new latency, recall, or storage benchmark results.
The pre-existing Task 111h TurboQuant benchmark packets were produced before
this centroid-relative sidecar fix, so they should not be used as post-fix
TurboQuant evidence. Together with packet 022, the post-fix benchmark rerun now
needs to cover RaBitQ4, RaBitQ8, and TurboQuant.
