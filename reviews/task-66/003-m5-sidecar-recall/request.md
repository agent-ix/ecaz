# Task 66 packet 003: M5 sidecar recall delta

## Summary

This packet closes reviewer flag B2 for packet 001. It adds a local M5 IVF
sidecar recall run for all four active bits=8 variants and a scalar-backend
comparison on the same isolated fixture.

The current NEON path and `ECAZ_SIMD=scalar` path produce identical recall@10
for `rabitq8`, `rabitq8ls`, `rabitq8c3`, and `rabitq8c4` on this fixture:
delta is `+0.0000` for every variant, within the Task 66 0.5 pp gate.

## Fixture

- Prefix: `task66_m5_10k_ivf_rabitq1_n64_sidecar_off`
- Source: copied from `task57_005_10k_ivf_rabitq_n64`
- Corpus/query rows: 10,000 / 200
- Index: `ec_ivf`, `storage_format=rabitq`, `quant_bits=1`, `rerank=off`,
  `nlists=64`, `nprobe=32`
- Query limit: 100
- Candidate frontier: `candidate_k=50`, `k=10`

## Key Results

`tid-sorted` recall@10:

- `rabitq8`: NEON `0.9850`, scalar `0.9850`, delta `+0.0000`
- `rabitq8ls`: NEON `0.9820`, scalar `0.9820`, delta `+0.0000`
- `rabitq8c3`: NEON `0.9940`, scalar `0.9940`, delta `+0.0000`
- `rabitq8c4`: NEON `0.9990`, scalar `0.9990`, delta `+0.0000`

Artifacts:

- `artifacts/results.jsonl`
- `artifacts/scalar-results.jsonl`
- `artifacts/rabitq8-variants-task57-10k-k50-p32.log`
- `artifacts/scalar-rabitq8-variants-task57-10k-k50-p32.log`
- `artifacts/manifest.md`
