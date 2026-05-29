# Task 66 packet 003 artifact manifest

- Head SHA: `fe1ec5ec9770988536736c32257a6841854563b6`
- Task bucket: `reviews/task-66/003-m5-sidecar-recall`
- Timestamp: `2026-05-29T12:22:20-0700`
- Lane: M5 local IVF sidecar recall
- Fixture: copied from `task57_005_10k_ivf_rabitq_n64`; 10,000 corpus rows, 200 query rows
- Storage format: `ec_ivf`, `storage_format=rabitq`, `quant_bits=1`, `rerank=off`, `nlists=64`, `nprobe=32`
- Rerank mode: sidecar `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`; free and tid-sorted reads
- Surface isolation: isolated Task 66 one-index-per-table fixture `task66_m5_10k_ivf_rabitq1_n64_sidecar_off`

## Setup and Inventory

- `db-relation-inventory.log`: local relation inventory before fixture setup.
- `task57-fixture-precheck.log`: source fixture row counts and source index definition.
- `task57-schema.log`: source table schema.
- `task66-fixture-setup.log`: copied source corpus/query tables and built the isolated `rerank=off` IVF index.

## NEON/current run

- Suite config: `suite.json`
- Suite manifest: `suite-manifest.json`
- Results: `results.jsonl`
- Raw log: `rabitq8-variants-task57-10k-k50-p32.log`
- Command:
  `/Users/peter/.cargo/bin/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-66/003-m5-sidecar-recall/suite.json`

Key `tid-sorted` results:

- `rabitq8`: recall@10 `0.9850`, sidecar_score_p50 `0.098 ms`
- `rabitq8ls`: recall@10 `0.9820`, sidecar_score_p50 `0.079 ms`
- `rabitq8c3`: recall@10 `0.9940`, sidecar_score_p50 `0.076 ms`
- `rabitq8c4`: recall@10 `0.9990`, sidecar_score_p50 `0.079 ms`

## Scalar-backend comparison

- Suite config: `suite-scalar.json`
- Suite manifest: `scalar-suite-manifest.json`
- Results: `scalar-results.jsonl`
- Raw log: `scalar-rabitq8-variants-task57-10k-k50-p32.log`
- Command:
  `/usr/bin/env ECAZ_SIMD=scalar /Users/peter/.cargo/bin/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-66/003-m5-sidecar-recall/suite-scalar.json`

Recall delta vs scalar backend:

- `rabitq8`: `0.9850 - 0.9850 = +0.0000` recall@10
- `rabitq8ls`: `0.9820 - 0.9820 = +0.0000` recall@10
- `rabitq8c3`: `0.9940 - 0.9940 = +0.0000` recall@10
- `rabitq8c4`: `0.9990 - 0.9990 = +0.0000` recall@10
