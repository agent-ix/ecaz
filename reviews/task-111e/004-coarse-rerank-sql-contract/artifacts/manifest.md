# Task 111e Packet 004 Artifact Manifest

- head SHA: `1f969c91bc945651e1f820373d780f93388e6ee6`
- task bucket: `reviews/task-111e/004-coarse-rerank-sql-contract`
- purpose: SQL-level validation for the explicit `coarse_rerank` contract
- generated: `2026-06-18T09:56:55-07:00`

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `cargo-test-coarse-rerank-pg18.log` | `script -q -c 'cargo test -q coarse_rerank --lib --no-default-features --features pg18' reviews/task-111e/004-coarse-rerank-sql-contract/artifacts/cargo-test-coarse-rerank-pg18.log` | 6 passed, 0 failed, 2131 filtered out |
| `cargo-test-metadata-roundtrip-pg18.log` | `script -q -c 'cargo test -q metadata_roundtrip --lib --no-default-features --features pg18' reviews/task-111e/004-coarse-rerank-sql-contract/artifacts/cargo-test-metadata-roundtrip-pg18.log` | 8 passed, 0 failed, 2129 filtered out |

## SQL Contract Cell

The PG18 `coarse_rerank` test creates a four-row `ecvector` table and an IVF
index with:

- `nlists = 2`
- `nprobe = 2`
- `training_sample_rows = 4`
- `storage_format = 'coarse_rerank'`
- `coarse_format = 'rabitq'`
- `coarse_bits = 1`
- `rerank = 'heap_f32'`
- `rerank_placement = 'heap'`
- `rerank_format = 'heap_f32'`
- `rerank_width = 3`

The admin snapshot reports the normalized contract as:

```text
coarse_rerank/rabitq/1/heap_f32/table/f32/3
```
