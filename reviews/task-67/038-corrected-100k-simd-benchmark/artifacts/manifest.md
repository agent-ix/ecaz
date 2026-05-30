# Task 67 Packet 038 Artifact Manifest

- head SHA: `d5d0a6c463affd91e67452dffcfd3c6f8a9ec9f0`
- task bucket: `reviews/task-67/038-corrected-100k-simd-benchmark/`
- timestamp: `2026-05-30T16:44:43Z`
- lane: corrected Task 67 100k AWS Intel RaBitQ8 sidecar benchmark
- fixture / storage format / rerank mode:
  - fixture: `ec_real_100k` prepared from `qdrant-dbpedia-openai3-large-1536-1m`
  - access method: `ec_ivf`
  - index storage: `storage_format=rabitq`, `quant_bits=1`, `rerank=off`
  - sidecar rerank variants: `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
  - sidecar benchmark query encoding: `bits=4`, `seed=42`
- isolated one-index-per-table or shared-table surfaces: isolated scalar and auto prefixes:
  - `task67_r8head_100k_scalar_envfix`
  - `task67_r8head_100k_auto_envfix`
- AWS profile: `10k-intel`
- final cloud state: `10k-intel` paused, `$0.00/hr` running cost

## Host Attestation

- DB instance: `i-02811174cc6ded75c`
- Instance type: `m7i.2xlarge`
- Architecture: `x86_64`
- CPU options: 4 cores, 2 threads per core
- Instance-type processor info: Intel, sustained clock 3.2 GHz
- Packet artifacts:
  - `artifacts/preflight/db-instance-attestation.log`
  - `artifacts/preflight/m7i-instance-type-attestation.log`
  - `artifacts/preflight/10k-intel-status-final.log`

## Why This Supersedes Packet 036

Packet 036 ran with `ecaz cloud bench --simd-mode`, but the cloud wrapper only set
`ECAZ_SIMD` for PostgreSQL. `bench sidecar-rerank` scores sidecar payloads in the
remote CLI process, so packet 036's scalar-vs-auto comparison was not proven.

Packet 037 fixed the runner at commit `bcd8e29c6073a3baff161cfd03e53dd238d44d04`
by exporting `ECAZ_SIMD` into the remote `ecaz bench suite` process. This packet
reruns the 100k pair using that fixed wrapper.

## Commands

### Scalar

```bash
target/debug/ecaz cloud bench \
  --profile 10k-intel \
  --simd-mode scalar \
  --config reviews/task-67/038-corrected-100k-simd-benchmark/artifacts/task67-rabitq8-100k-scalar-envfix-suite.json \
  --suite task67-rabitq8-100k-scalar-envfix \
  --database postgres \
  --ecaz-bin /usr/local/bin/ecaz \
  --log-file reviews/task-67/038-corrected-100k-simd-benchmark/artifacts/100k-scalar/cloud-bench-100k-scalar-envfix.log
```

- S3 URI: `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq8-100k-scalar-envfix/20260530T163613Z/`
- artifacts:
  - `artifacts/100k-scalar/results.jsonl`
  - `artifacts/100k-scalar/suite-manifest.json`
  - `artifacts/100k-scalar/suite-run.log`
  - `artifacts/100k-scalar/load-100k-rabitq8-headline-scalar-envfix.log`
  - `artifacts/100k-scalar/sidecar-100k-rabitq8-headline-scalar-envfix.log`

### Auto

```bash
target/debug/ecaz cloud bench \
  --profile 10k-intel \
  --simd-mode auto \
  --config reviews/task-67/038-corrected-100k-simd-benchmark/artifacts/task67-rabitq8-100k-auto-envfix-suite.json \
  --suite task67-rabitq8-100k-auto-envfix \
  --database postgres \
  --ecaz-bin /usr/local/bin/ecaz \
  --log-file reviews/task-67/038-corrected-100k-simd-benchmark/artifacts/100k-auto/cloud-bench-100k-auto-envfix.log
```

- S3 URI: `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq8-100k-auto-envfix/20260530T164005Z/`
- artifacts:
  - `artifacts/100k-auto/results.jsonl`
  - `artifacts/100k-auto/suite-manifest.json`
  - `artifacts/100k-auto/suite-run.log`
  - `artifacts/100k-auto/load-100k-rabitq8-headline-auto-envfix.log`
  - `artifacts/100k-auto/sidecar-100k-rabitq8-headline-auto-envfix.log`

## Key Result Lines

`artifacts/100k-comparison.tsv` summarizes the corrected scalar-vs-auto pair.

- sidecar score p50:
  - scalar: `0.107-0.111 ms`
  - auto: `0.019-0.022 ms`
  - speedup: `4.864-5.842x`
- total bound p50:
  - scalar: `13.433-24.287 ms`
  - auto: `11.167-19.136 ms`
  - speedup: `1.197-1.271x`
- recall@10 range across variants/nprobe: `0.9470-0.9940`
- load:
  - scalar: 100000 corpus rows, 1000 query rows, total load `67.66s`
  - auto: 100000 corpus rows, 1000 query rows, total load `76.25s`

## Bits Field Reconciliation

The packet intentionally includes three "bits" surfaces:

- `quant_bits=1` is the IVF RaBitQ index reloption under test.
- Suite/default `bits=4` is the query encoding width used by the suite runner for `corpus load` and `bench sidecar-rerank`.
- Sidecar variants `rabitq8*` identify the fixed-width sidecar rerank payload family being benchmarked after the IVF candidate frontier is collected.

## Re-run Notes

Preconditions:

- `target/debug/ecaz` must include packet 037's cloud wrapper fix.
- `10k-intel` must be resumed before running the suites and paused afterward.
- The remote `/usr/local/bin/ecaz` must be installed on the DB host.

Dry-run validation artifacts:

- `artifacts/100k-scalar/suite-dry-run.log`
- `artifacts/100k-scalar/suite-dry-run-manifest.json`
- `artifacts/100k-auto/suite-dry-run.log`
- `artifacts/100k-auto/suite-dry-run-manifest.json`

## 1m Status

This packet does not claim 1m HNSW or DiskANN results. Packet 036 documents the
`1m` profile VPC quota blocker and cleanup. The corrected 100k SIMD comparison
is the only new benchmark evidence in this packet.
