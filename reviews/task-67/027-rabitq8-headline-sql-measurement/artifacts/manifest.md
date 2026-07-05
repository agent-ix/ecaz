# Task 67 Packet 027 Manifest

- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/027-rabitq8-headline-sql-measurement/`
- Head SHA: `b0c1403a22d5c32f923143050d77d776122ade8d`
- Timestamp: 2026-05-30
- Lane: AWS Intel `10k-intel`, PG18, real 10k corpus, `ec_ivf`, RaBitQ8 sidecar rerank
- Fixture: `target/real-corpus/staged-task50/ec_real_10k_{corpus,queries,manifest}.json|tsv`, 200 queries
- Storage format: isolated one-index-per-table surfaces, `storage_format=rabitq`, `quant_bits=1`
- Rerank mode: base IVF index uses `rerank=off`; the sidecar rerank command measures `candidate_k=100`
- Variants: `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
- Nprobe sweep: 16, 32, 64
- Surface isolation:
  - Scalar prefix: `task67_r8head_10k_scalar`
  - Auto prefix: `task67_r8head_10k_auto`

## Commands

Local audit:

```sh
target/debug/ecaz bench suite audit --config reviews/task-67/027-rabitq8-headline-sql-measurement/artifacts/task67-rabitq8-headline-scalar-suite.json
target/debug/ecaz bench suite audit --config reviews/task-67/027-rabitq8-headline-sql-measurement/artifacts/task67-rabitq8-headline-auto-suite.json
```

AWS setup:

```sh
target/debug/ecaz cloud resume --profile 10k-intel
target/debug/ecaz cloud install --profile 10k-intel --git-ref b0c1403a2 --skip-extension-recreate --database postgres --timeout 3600
```

AWS suite execution:

```sh
target/debug/ecaz cloud bench --profile 10k-intel --simd-mode scalar --config reviews/task-67/027-rabitq8-headline-sql-measurement/artifacts/task67-rabitq8-headline-scalar-suite.json --suite task67-rabitq8-headline-scalar --database postgres --ecaz-bin /usr/local/bin/ecaz
target/debug/ecaz cloud bench --profile 10k-intel --simd-mode auto --config reviews/task-67/027-rabitq8-headline-sql-measurement/artifacts/task67-rabitq8-headline-auto-suite.json --suite task67-rabitq8-headline-auto --database postgres --ecaz-bin /usr/local/bin/ecaz
```

AWS shutdown:

```sh
target/debug/ecaz cloud pause --profile 10k-intel
target/debug/ecaz cloud status --profile 10k-intel
```

`artifacts/preflight/cloud-status-final.log` records the final `paused` state
with `~$0.00/hr running` cost.

## S3 Runs

- Scalar: `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq8-headline-scalar/20260530T143737Z/`
- Auto: `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq8-headline-auto/20260530T143808Z/`

## Artifact Inventory

Suite configs:

- `artifacts/task67-rabitq8-headline-scalar-suite.json`
- `artifacts/task67-rabitq8-headline-auto-suite.json`

Local audit and cloud preflight:

- `artifacts/local/suite-audit-scalar.log`
- `artifacts/local/suite-audit-auto.log`
- `artifacts/preflight/cloud-resume.log`
- `artifacts/preflight/cloud-install-b0c1403a2.log`
- `artifacts/preflight/cloud-pause.log`
- `artifacts/preflight/cloud-status-after-pause.log`
- `artifacts/preflight/cloud-status-final.log`

Scalar suite:

- `artifacts/scalar/results.jsonl`
- `artifacts/scalar/results-report.jsonl`
- `artifacts/scalar/suite-manifest.json`
- `artifacts/scalar/suite-dry-run-manifest.json`
- `artifacts/scalar/suite-run.log`
- `artifacts/scalar/cloud-bench-rabitq8-headline-scalar.log`
- `artifacts/scalar/load-10k-rabitq1-rabitq8-headline-scalar.log`
- `artifacts/scalar/sidecar-10k-rabitq8-headline-scalar.log`

Auto suite:

- `artifacts/auto/results.jsonl`
- `artifacts/auto/results-report.jsonl`
- `artifacts/auto/suite-manifest.json`
- `artifacts/auto/suite-dry-run-manifest.json`
- `artifacts/auto/suite-run.log`
- `artifacts/auto/cloud-bench-rabitq8-headline-auto.log`
- `artifacts/auto/load-10k-rabitq1-rabitq8-headline-auto.log`
- `artifacts/auto/sidecar-10k-rabitq8-headline-auto.log`

## Key Results

Headline `total_bound_p50` speedup, scalar divided by auto:

| variant | nprobe=16 | nprobe=32 | nprobe=64 |
| --- | ---: | ---: | ---: |
| `rabitq8` | 0.92x | 0.90x | 1.09x |
| `rabitq8ls` | 0.94x | 0.95x | 1.09x |
| `rabitq8c3` | 0.93x | 0.94x | 1.09x |
| `rabitq8c4` | 0.92x | 0.93x | 1.08x |

Auto `sidecar_score_p50`:

| variant | nprobe=16 | nprobe=32 | nprobe=64 |
| --- | ---: | ---: | ---: |
| `rabitq8` | 0.025 ms | 0.025 ms | 0.025 ms |
| `rabitq8ls` | 0.024 ms | 0.025 ms | 0.025 ms |
| `rabitq8c3` | 0.026 ms | 0.026 ms | 0.025 ms |
| `rabitq8c4` | 0.026 ms | 0.025 ms | 0.024 ms |

This packet does not satisfy Task 67's strict bits=8 headline SQL 4x gate.
