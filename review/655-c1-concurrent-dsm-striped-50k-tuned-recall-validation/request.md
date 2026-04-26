# Review Request: Concurrent DSM striped 50k tuned recall validation

## Summary

This packet extends packet 654's tuned striped recall validation from 10k rows
to a 50k-row synthetic fixture.

Fixture:

- PostgreSQL 18.3
- 50,000 corpus rows x 64 dimensions
- 50 query rows x 64 dimensions
- `ecvector` column encoded with `encode_to_ecvector(source, 4, 42)`
- default current-format `ec_hnsw` index
- `m = 16`
- `ef_construction = 128`
- `ef_search` sweep: `128`, `200`, `400`
- same shared corpus/query tables for serial and striped concurrent DSM indexes

Artifacts:

- `artifacts/pg18_concurrent_dsm_striped_50k_tuned_recall_validation.sql`
- `artifacts/pg18_concurrent_dsm_striped_50k_tuned_recall_validation.log`
- `artifacts/manifest.md`

## Result

| Build Path | Workers | Build Wall | Graph Phase | Recall@10 ef=128 | Recall@10 ef=200 | Recall@10 ef=400 | Index Bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| Serial | 0 | 83,128 ms | 80,949 ms | 0.234 | 0.246 | 0.256 | 17,752,064 |
| Concurrent DSM striped | 4 | 30,729 ms | 28,786 ms | 0.256 | 0.256 | 0.256 | 17,752,064 |

At the main `ef_search = 200` comparison point:

- serial recall@10: `0.246`
- striped concurrent DSM recall@10: `0.256`
- recall@10 delta: `+0.010000005`
- serial recall@100: `0.6656`
- striped concurrent DSM recall@100: `0.6714`
- recall@100 delta: `+0.005800009`

## Interpretation

On this 50k tuned fixture, striped concurrent DSM preserves the build-time gain
while matching the serial `ef=400` recall@10 point and slightly exceeding the
serial `ef=200` point.

The absolute recall is still low for this synthetic fixture even with tuned
settings; exact quantized recall@10 remains `1.0`, so the gap is graph topology
and traversal rather than quantization. The relevant result for this packet is
that striped concurrent DSM does not introduce a visible serial-parity recall
regression at 50k scale.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/655-c1-concurrent-dsm-striped-50k-tuned-recall-validation/artifacts/pg18_concurrent_dsm_striped_50k_tuned_recall_validation.sql --log-output review/655-c1-concurrent-dsm-striped-50k-tuned-recall-validation/artifacts/pg18_concurrent_dsm_striped_50k_tuned_recall_validation.log`

## Review Focus

- Confirm this is a valid 50k-scale extension of packet 654.
- Confirm the shared-table setup is acceptable for serial vs striped concurrent DSM comparison.
- Confirm the next gate should move to a larger/real corpus rather than more synthetic scheduler tuning.
