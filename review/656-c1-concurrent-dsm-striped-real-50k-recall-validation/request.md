# Review Request: Concurrent DSM striped real 50k source-scored blocker

## Summary

This packet attempted the next real-corpus gate after packets 654 and 655:
compare the striped concurrent DSM graph against the real 50k m16 baseline.

Setup:

- PostgreSQL 18.3
- real fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`
- 50,000 corpus rows x 1536 dimensions
- 1,000 query rows x 1536 dimensions
- 10-query subset for this smoke attempt
- existing serial index: `tqhnsw_real_50k_m16_idx`
- serial index reloptions: `m=16`, `ef_construction=128`, `build_source_column=source`

Artifacts:

- `artifacts/pg18_concurrent_dsm_striped_real_50k_recall_validation.sql`
- `artifacts/pg18_concurrent_dsm_striped_real_50k_recall_validation.log`
- `artifacts/manifest.md`

## Result

The existing serial source-scored real-corpus baseline ran successfully at
`ef_search=200` on the 10-query subset:

- graph recall@10: `0.91`
- graph recall@100: `0.762`
- exact quantized recall@10: `1.0`
- NDCG@10: `0.947258`
- index size: `68,280,320` bytes

The striped concurrent DSM sidecar build failed before recall measurement:

```text
ERROR: concurrent DSM graph assembly does not support source-scored builds yet
```

## Interpretation

This is a real gate blocker, not a threshold issue. The current concurrent DSM
graph path only carries quantized code bytes in its DSM graph corpus and
explicitly rejects `build_source_column=source`. The real m16 baseline uses
source-scored graph construction, so a recall-faithful real-corpus comparison
cannot run until concurrent DSM supports source-scored builds or the gate is
changed to an encoded-only comparison.

The synthetic packets remain valid for encoded-code topology behavior. They do
not prove parity for the source-scored real-corpus lane.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/656-c1-concurrent-dsm-striped-real-50k-recall-validation/artifacts/pg18_concurrent_dsm_striped_real_50k_recall_validation.sql --log-output review/656-c1-concurrent-dsm-striped-real-50k-recall-validation/artifacts/pg18_concurrent_dsm_striped_real_50k_recall_validation.log`

## Review Focus

- Confirm that `build_source_column=source` support is required before treating
  concurrent DSM as real-corpus recall-ready.
- Confirm whether an encoded-only real-corpus packet is still useful as a
  lower-fidelity topology check.
- Confirm whether the next implementation slice should add source-vector DSM
  storage/scoring or defer real-corpus gating until that larger slice.
