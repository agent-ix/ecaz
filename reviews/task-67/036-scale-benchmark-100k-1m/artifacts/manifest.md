# Task 67 Packet 036 Artifact Manifest

- head SHA: `c24426eff12817824afb36409715970445f40053`
- task bucket: `reviews/task-67/036-scale-benchmark-100k-1m/`
- timestamp: `2026-05-30T16:25:52Z`
- lane: Task 67 scale follow-up AWS benchmark evidence
- fixture / storage format / rerank mode:
  - 100k RaBitQ8 IVF sidecar: `ec_real_100k`, `ec_ivf`, `storage_format=rabitq`, `rerank=off`, variants `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
  - 1m HNSW context: `ec_real_ann_benchmarks_anchor`, `ec_hnsw`, `m=16`, `ef_construction=128`
  - 1m DiskANN context: `ec_real_ann_benchmarks_anchor`, `ec_diskann`, default build options
- isolated one-index-per-table or shared-table surfaces: isolated prefixes per suite
- AWS profiles:
  - `10k-intel`: used for the completed 100k scalar/auto runs; final status `paused`
  - `1m`: attempted for 1m HNSW/DiskANN context; final status `down`

## Suite Configs

- `artifacts/task67-rabitq8-100k-scalar-suite.json`
- `artifacts/task67-rabitq8-100k-auto-suite.json`
- `artifacts/task67-hnsw-1m-suite.json`
- `artifacts/task67-diskann-1m-suite.json`
- `artifacts/task67-hnsw-1m-min-suite.json`
- `artifacts/task67-diskann-1m-min-suite.json`

## Completed 100k Runs

### Scalar

- command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode scalar --config reviews/task-67/036-scale-benchmark-100k-1m/artifacts/task67-rabitq8-100k-scalar-suite.json --suite task67-rabitq8-100k-scalar --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/036-scale-benchmark-100k-1m/artifacts/100k-scalar/cloud-bench-100k-scalar.log`
- remote artifact URI: `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq8-100k-scalar/20260530T155607Z/`
- packet artifacts:
  - `artifacts/100k-scalar/results.jsonl`
  - `artifacts/100k-scalar/suite-run.log`
  - `artifacts/100k-scalar/suite-manifest.json`
  - `artifacts/100k-scalar/load-100k-rabitq8-headline-scalar.log`
  - `artifacts/100k-scalar/sidecar-100k-rabitq8-headline-scalar.log`
- key load lines: 100000 corpus rows, 1000 query rows; copy corpus 25.80s, encode corpus 30.60s, build index 4.22s, total 66.54s; sidecar size 147.63 MiB.

### Auto

- command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode auto --config reviews/task-67/036-scale-benchmark-100k-1m/artifacts/task67-rabitq8-100k-auto-suite.json --suite task67-rabitq8-100k-auto --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/036-scale-benchmark-100k-1m/artifacts/100k-auto/cloud-bench-100k-auto.log`
- remote artifact URI: `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq8-100k-auto/20260530T155927Z/`
- packet artifacts:
  - `artifacts/100k-auto/results.jsonl`
  - `artifacts/100k-auto/suite-run.log`
  - `artifacts/100k-auto/suite-manifest.json`
  - `artifacts/100k-auto/load-100k-rabitq8-headline-auto.log`
  - `artifacts/100k-auto/sidecar-100k-rabitq8-headline-auto.log`
- key load lines: 100000 corpus rows, 1000 query rows; copy corpus 25.86s, encode corpus 41.12s, build index 4.44s, total 77.94s; sidecar size 147.63 MiB.

### Comparison

`artifacts/100k-comparison.tsv` summarizes scalar-vs-auto from `results.jsonl`.

- Sidecar score p50 range: scalar 0.022-0.023 ms, auto 0.022-0.026 ms.
- Total bound p50 range: scalar 12.297-21.971 ms, auto 12.455-22.131 ms.
- Total speedup range: 0.982-1.000x. Candidate SQL dominates total latency at this scale.
- Recall range across variants/nprobe: 0.9470-0.9940.

## 1m Attempts

- `target/debug/ecaz cloud --log-file reviews/task-67/036-scale-benchmark-100k-1m/artifacts/preflight/1m-up-confirmed.log up --profile 1m --git-ref main --confirm-cost 11 --database postgres`
  - failed during Terraform apply before usable DB provisioning because the AWS account had reached the VPC quota; see `artifacts/preflight/1m-vpc-quota-note.md`.
- `target/debug/ecaz cloud --log-file reviews/task-67/036-scale-benchmark-100k-1m/artifacts/preflight/1m-down-after-vpc-limit.log down --profile 1m --yes --no-snapshot-required --database postgres`
  - cleaned up partial `1m` resources; final `target/debug/ecaz cloud --log-file reviews/task-67/036-scale-benchmark-100k-1m/artifacts/preflight/1m-status-final.log status --profile 1m` reports `state: down`.
- A fallback HNSW run on `10k-intel` using `artifacts/task67-hnsw-1m-suite.json` reached `ecaz bench suite` execution but failed before producing packet-local suite results; `artifacts/1m-hnsw/cloud-bench-1m-hnsw-on-10k-intel.log` records SSM command `1cf22ca6-13ca-4167-a1fa-30b3100a8a80` ending failed.
- Minimal 1m HNSW and DiskANN configs were dry-run validated:
  - `artifacts/1m-hnsw/suite-min-dry-run.log`
  - `artifacts/1m-hnsw/suite-min-dry-run-manifest.json`
  - `artifacts/1m-diskann/suite-min-dry-run.log`
  - `artifacts/1m-diskann/suite-min-dry-run-manifest.json`
- No 1m HNSW or DiskANN result is claimed in this packet.

## Final Cloud State

- `target/debug/ecaz cloud --log-file reviews/task-67/036-scale-benchmark-100k-1m/artifacts/preflight/10k-intel-pause-final.log pause --profile 10k-intel`
  - stopped DB and loader instances.
- `target/debug/ecaz cloud --log-file reviews/task-67/036-scale-benchmark-100k-1m/artifacts/preflight/10k-intel-status-final.log status --profile 10k-intel`
  - final status: `paused`, running cost `$0.00/hr`, retained storage about `$8.00/mo`.
