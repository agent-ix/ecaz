# Task 111e Packet 003 Artifact Manifest

- head SHA: `69d75b318b08d56ef107a275c06f3873f3d9efac`
- task bucket: `reviews/task-111e/003-coarse-rerank-contract`
- purpose: explicit `coarse_rerank` contract reloptions and admin diagnostics
- generated: `2026-06-18T09:47:42-07:00`

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `cargo-test-coarse-rerank-pg18.log` | `script -q -c 'cargo test -q coarse_rerank --lib --no-default-features --features pg18' reviews/task-111e/003-coarse-rerank-contract/artifacts/cargo-test-coarse-rerank-pg18.log` | 5 passed, 0 failed, 2131 filtered out |
| `cargo-test-metadata-roundtrip-pg18.log` | `script -q -c 'cargo test -q metadata_roundtrip --lib --no-default-features --features pg18' reviews/task-111e/003-coarse-rerank-contract/artifacts/cargo-test-metadata-roundtrip-pg18.log` | 8 passed, 0 failed, 2128 filtered out |
| `cargo-test-scratch-soa-gate-pg18.log` | `script -q -c 'cargo test -q scratch_soa_batch_decode_gate --lib --no-default-features --features pg18' reviews/task-111e/003-coarse-rerank-contract/artifacts/cargo-test-scratch-soa-gate-pg18.log` | 1 passed, 0 failed, 2135 filtered out |

## Contract

`storage_format = 'coarse_rerank'` now has explicit normalized contract fields:

- coarse format: `rabitq`
- coarse bits: `1`
- rerank placement: `table` (the `heap` alias is accepted and normalized)
- rerank format: `f32` (the `heap_f32` alias is accepted and normalized)

Unsupported future variants are parsed as named reloption values but rejected
for `coarse_rerank` until their implementation packets:

- `rerank_placement = 'index'`
- `rerank_format = 'rabitq2'`
- `rerank_format = 'rabitq4'`
- `rerank_format = 'rabitq8'`
- `rerank_format = 'turboquant'`
