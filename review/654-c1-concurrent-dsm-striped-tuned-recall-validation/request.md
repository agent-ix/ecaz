# Review Request: Concurrent DSM striped tuned recall validation

## Summary

This packet reruns the packet 652 tuned recall fixture after code checkpoint
`654ebb1` changed concurrent DSM graph insertion from one contiguous node range
per participant to 64-node striped global-order chunks.

Fixture:

- PostgreSQL 18.3
- 10,000 corpus rows x 64 dimensions
- 100 query rows x 64 dimensions
- `ecvector` column encoded with `encode_to_ecvector(source, 4, 42)`
- default current-format `ec_hnsw` index
- `m = 16`
- `ef_construction = 128`
- `ef_search` sweep: `128`, `200`, `400`
- same shared corpus/query tables for serial and concurrent DSM indexes

Artifacts:

- `artifacts/pg18_concurrent_dsm_striped_tuned_recall_validation.sql`
- `artifacts/pg18_concurrent_dsm_striped_tuned_recall_validation.log`
- `artifacts/manifest.md`

## Result

| Build Path | Workers | Build Wall | Graph Phase | Recall@10 ef=128 | Recall@10 ef=200 | Recall@10 ef=400 | Index Bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| Serial | 0 | 13,627 ms | 13,147 ms | 0.505 | 0.534 | 0.599 | 3,563,520 |
| Concurrent DSM striped | 4 | 5,282 ms | 4,905 ms | 0.552 | 0.558 | 0.582 | 3,563,520 |

At the main `ef_search = 200` comparison point:

- serial recall@10: `0.534`
- striped concurrent DSM recall@10: `0.558`
- recall@10 delta: `+0.024000049`
- serial recall@100: `0.7189`
- striped concurrent DSM recall@100: `0.7256`
- recall@100 delta: `+0.0066999793`

Compared to packet 652, the concurrent DSM high-ef row no longer stays flat:

- packet 652 contiguous-range concurrent DSM: `0.528`, `0.538`, `0.538`
- packet 654 striped concurrent DSM: `0.552`, `0.558`, `0.582`

## Interpretation

The striped scheduler is a credible improvement over contiguous participant
ranges on this fixture. It preserves the build-time gain (`5.282s` concurrent
DSM versus `13.627s` serial) while improving recall at every tested ef point
relative to packet 652's contiguous-range concurrent DSM graph.

It still does not fully match serial at `ef_search=400` (`0.582` versus
`0.599`), but the previous flatline is gone. That supports the diagnosis that
range-local insertion skew was contributing to the high-ef behavior.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/654-c1-concurrent-dsm-striped-tuned-recall-validation/artifacts/pg18_concurrent_dsm_striped_tuned_recall_validation.sql --log-output review/654-c1-concurrent-dsm-striped-tuned-recall-validation/artifacts/pg18_concurrent_dsm_striped_tuned_recall_validation.log`

## Review Focus

- Confirm the comparison against packet 652 is valid and the fixture is unchanged apart from the striped insertion scheduler.
- Confirm the remaining `ef=400` serial gap is acceptable as a follow-up tuning question rather than a blocker for the striped scheduler checkpoint.
- Confirm whether the next measurement should move to a larger or real-corpus fixture before further scheduler changes.
