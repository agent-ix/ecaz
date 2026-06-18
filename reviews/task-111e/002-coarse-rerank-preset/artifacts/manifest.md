# Task 111e Packet 002 Artifact Manifest

- head SHA: `533ace12106a599476fe107b75308f437c3670e8`
- task bucket: `reviews/task-111e/002-coarse-rerank-preset`
- purpose: explicit IVF `storage_format = 'coarse_rerank'` contract preset
- generated: 2026-06-18

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `cargo-test-coarse-rerank-pg18.log` | `script -q -c 'cargo test -q coarse_rerank --lib --no-default-features --features pg18' reviews/task-111e/002-coarse-rerank-preset/artifacts/cargo-test-coarse-rerank-pg18.log` | 4 passed, 0 failed, 2131 filtered out |
| `cargo-test-metadata-roundtrip-pg18.log` | `script -q -c 'cargo test -q metadata_roundtrip --lib --no-default-features --features pg18' reviews/task-111e/002-coarse-rerank-preset/artifacts/cargo-test-metadata-roundtrip-pg18.log` | 8 passed, 0 failed, 2127 filtered out |
| `cargo-test-scratch-soa-gate-pg18.log` | `script -q -c 'cargo test -q scratch_soa_batch_decode_gate --lib --no-default-features --features pg18' reviews/task-111e/002-coarse-rerank-preset/artifacts/cargo-test-scratch-soa-gate-pg18.log` | 1 passed, 0 failed, 2134 filtered out |

## Contract

`storage_format = 'coarse_rerank'` is an explicit gated Task 111e preset. It
uses the existing IVF RaBitQ machinery rather than introducing a second storage
engine:

- persisted storage format name/code: `coarse_rerank`
- coarse quantizer profile: RaBitQ
- coarse bits: `quant_bits = 1`
- hot posting layout: `dense_posting_blocks = true`
- dense group geometry: `dense_posting_pack_pages = 1`
- aligned dense layout: `dense_posting_typed_layout = true`
- rerank placement/format: table/heap-side f32 via `rerank = heap_f32`
- rerank width: existing `rerank_width` reloption/GUC

The preset accepts `rerank = auto` and resolves it to `heap_f32`, or accepts an
explicit `rerank = heap_f32`. It rejects `rerank = off` and
`rerank = source_column` because this mode is defined as a coarse-rerank
pipeline, not a no-rerank RaBitQ alias.
