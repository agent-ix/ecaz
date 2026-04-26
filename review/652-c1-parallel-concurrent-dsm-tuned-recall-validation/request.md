# Review Request: Parallel Concurrent DSM Tuned Recall Validation

## Summary

This packet answers the concern raised after packets 650 and 651: the previous
recall values were low because those packets reused the cheap build-speed
settings (`m = 6`, `ef_construction = 40`). This packet reruns the serial versus
concurrent-DSM topology comparison with more recall-oriented settings:

- `m = 16`
- `ef_construction = 128`
- `ef_search` sweep: `128`, `200`, `400`

Fixture:

- PostgreSQL 18.3
- 10,000 corpus rows x 64 dimensions
- 100 query rows x 64 dimensions
- `ecvector` column encoded with `encode_to_ecvector(source, 4, 42)`
- default TurboQuant current-format index
- same shared corpus/query tables for serial and concurrent DSM indexes

Artifacts:

- `artifacts/pg18_parallel_concurrent_dsm_tuned_recall_validation.sql`
- `artifacts/pg18_parallel_concurrent_dsm_tuned_recall_validation.log`
- `artifacts/manifest.md`

## Result

| Build Path | Workers | Build Wall | Graph Phase | Recall@10 ef=128 | Recall@10 ef=200 | Recall@10 ef=400 | Index Bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| Serial | 0 | 14,224 ms | 13,767 ms | 0.505 | 0.534 | 0.599 | 3,563,520 |
| Concurrent DSM | 4 | 5,367 ms | 4,988 ms | 0.528 | 0.538 | 0.538 | 3,563,520 |

At the main `ef_search = 200` comparison point:

- serial recall@10: `0.534`
- concurrent DSM recall@10: `0.538`
- recall@10 delta: `+0.004000008`
- serial recall@100: `0.7189`
- concurrent DSM recall@100: `0.714`
- recall@100 delta: `-0.0049000382`

## Interpretation

The tuned settings substantially improve recall over the cheap `m=6`,
`ef_construction=40` packets. At `ef_search=200`, concurrent DSM is effectively
at recall parity with serial on this fixture while preserving the build-time
gain: `5.367s` concurrent DSM versus `14.224s` serial.

The `ef_search=400` serial row improves further while concurrent DSM is flat.
That means the concurrent topology may have less benefit from very high
`ef_search` on this synthetic fixture, and larger/real-corpus validation should
include a high-ef point before considering the GUC default.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/652-c1-parallel-concurrent-dsm-tuned-recall-validation/artifacts/pg18_parallel_concurrent_dsm_tuned_recall_validation.sql --log-output review/652-c1-parallel-concurrent-dsm-tuned-recall-validation/artifacts/pg18_parallel_concurrent_dsm_tuned_recall_validation.log`

## Review Focus

- Confirm the low recall in packets 650/651 is attributable to cheap benchmark
  settings rather than an obvious concurrent DSM correctness bug.
- Confirm tuned serial and concurrent DSM recall are close at `ef_search=200`.
- Confirm the high-ef flattening on concurrent DSM should remain a follow-up
  measurement question before default promotion.
