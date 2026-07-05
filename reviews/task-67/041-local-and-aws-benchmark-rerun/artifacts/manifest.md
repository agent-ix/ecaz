# Task 67 Local and AWS Benchmark Rerun Manifest

- Packet: `reviews/task-67/041-local-and-aws-benchmark-rerun`
- Packaged at: 2026-05-31T04:09:31Z
- Packaging head SHA: `81b7f7ea6c3902aa90c126e88d705f586811d174`
- AWS installed repo head evidence: `artifacts/aws-1m-diskann-fresh/repo-head-after-stop-start-postgres-invocation.json` reported `d67e39d1ca07444a948e4e8584f5e9976ee50211`.
- Runner: `ecaz bench suite` for all benchmark matrices and sweeps.

## AWS 1m DiskANN

- Lane: AWS profile `1m`, DB host `m7g.2xlarge`, `ec_real_1m`, `ec_diskann`, bits=4, seed=42.
- Fixture: `/var/lib/pgsql/18/datasets/staged-task67-1m/ec_real_ann_benchmarks_anchor_manifest.json`.
- Surface: isolated one-index table prefix `task67_1m_diskann_m7g2xlarge`.
- Config: `artifacts/task67-diskann-1m-m7g2xlarge-suite.json`.
- Suite manifest: `artifacts/aws-1m-diskann-m7g2xlarge/suite-manifest.json`.
- Results: `artifacts/aws-1m-diskann-m7g2xlarge/results.jsonl`.
- Command: `target/debug/ecaz cloud bench --profile 1m --config artifacts/task67-diskann-1m-m7g2xlarge-suite.json --suite task67-diskann-1m-m7g2xlarge --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file artifacts/aws-1m-diskann-m7g2xlarge/cloud-bench-1m-diskann-m7g2xlarge.log`.
- Timestamp: suite manifest generated at 2026-05-30T23:35:28Z.
- Key results: build index `6639.91s`, total load/build `7469.10s`; recall@10 list_size `64,128,200,400,800` = `0.9340,0.9620,0.9690,0.9730,0.9770`; latency mean = `4.95ms,6.77ms,8.59ms,13.6ms,22.8ms`; storage total `15.8 GiB`, DiskANN index `455.1 MiB`.

## AWS 1m HNSW

- Lane: AWS profile `1m`, DB host `m7g.2xlarge`, `ec_real_1m`, `ec_hnsw`, bits=4, seed=42, `m=16`, `ef_construction=128`.
- Fixture: `/var/lib/pgsql/18/datasets/staged-task67-1m/ec_real_ann_benchmarks_anchor_manifest.json`.
- Surface: isolated one-index table prefix `task67_1m_hnsw_m7g2xlarge`.
- Config: `artifacts/task67-hnsw-1m-m7g2xlarge-suite.json`.
- Suite manifest: `artifacts/aws-1m-hnsw-m7g2xlarge/suite-manifest.json`.
- Results: `artifacts/aws-1m-hnsw-m7g2xlarge/results.jsonl`.
- Command: `target/debug/ecaz cloud bench --profile 1m --config artifacts/task67-hnsw-1m-m7g2xlarge-suite.json --suite task67-hnsw-1m-m7g2xlarge --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file artifacts/aws-1m-hnsw-m7g2xlarge/cloud-bench-1m-hnsw-m7g2xlarge.log`.
- Timestamp: suite manifest generated at 2026-05-31T01:47:00Z.
- Key results: build index `1103.96s`, total load/build `1929.76s`; recall@10 ef `64,128,200,400` = `0.8970,0.9190,0.9270,0.9310`; latency mean = `8.04ms,13.1ms,19.0ms,34.0ms`; storage total `16.6 GiB`, HNSW index `1.3 GiB`.

## Local Intel 50k/100k Standard Sweep

- Lane: local Intel Core i9-10900K, PG18 socket `/home/peter/.pgrx`, database `postgres`, bits=4, seed=42.
- Config: `artifacts/task67-local-50k-100k-standard-suite.json`.
- Initial manifest: `artifacts/local/suite-standard-manifest.json`.
- Resume manifest: `artifacts/local/suite-standard-manifest-resume-diskann.json`.
- Initial parsed results: `artifacts/local/results-standard-initial.jsonl`.
- Resume parsed results: `artifacts/local/results-standard-resume-report.jsonl`.
- Initial command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 bench suite run --config artifacts/task67-local-50k-100k-standard-suite.json --artifact-dir artifacts/local --manifest-output artifacts/local/suite-standard-manifest.json --results-output artifacts/local/results.jsonl --log-file artifacts/local/suite-standard-run.log`.
- Resume command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 bench suite run --config artifacts/task67-local-50k-100k-standard-suite.json --artifact-dir artifacts/local --manifest-output artifacts/local/suite-standard-manifest-resume-diskann.json --results-output artifacts/local/results-resume-diskann.jsonl --resume-from artifacts/local/suite-standard-manifest.json --only load-50k-diskann --only recall-50k-diskann --only latency-50k-diskann --only storage-50k-diskann --only load-100k-ivfrabitq --only recall-100k-ivfrabitq --only latency-100k-ivfrabitq --only storage-100k-ivfrabitq --only load-100k-hnsw --only recall-100k-hnsw --only latency-100k-hnsw --only storage-100k-hnsw --only load-100k-diskann --only recall-100k-diskann --only latency-100k-diskann --only storage-100k-diskann --log-file artifacts/local/suite-standard-resume-diskann-run.log`.
- Local DiskANN catalog repair: `artifacts/local/install-ecaz-pg18-after-diskann-missing.log`, `artifacts/local/pg-diskann-catalog-repair-resolved-module.log`, `artifacts/local/pg-am-after-diskann-catalog-repair.log`.
- Surface: standard shared-table suite for HNSW `m=8,m=16`; isolated per-engine prefixes for IVF/RaBitQ and DiskANN.
- 50k fixture: `target/real-corpus/staged-task67-local`, 50,000 corpus rows, 1,000 query rows.
- 100k fixture: `target/real-corpus/staged-task50`, 100,000 corpus rows, 1,000 query rows.
- Key 50k IVF/RaBitQ results: build index `638.78s`, total `792.95s`; recall@10 nprobe `8,16,24,32,48,64` = `0.8330,0.8910,0.9095,0.9215,0.9330,0.9375`; latency mean = `59.3ms,88.3ms,113.2ms,140.4ms,194.5ms,245.5ms`; storage total `840.9 MiB`.
- Key 50k HNSW results: `m=8` build `564.72s`, `m=16` build `797.92s`, total `1515.96s`; recall@10 ef `40,80,120,200,400` = `0.8650,0.9110,0.9185,0.9315,0.9225`; latency mean = `17.6ms,24.9ms,31.2ms,44.2ms,48.2ms`; storage total `918.9 MiB`.
- Key 50k DiskANN results: build index `160.47s`, total `236.28s`; recall@10 list_size `64,128,200,400,800` = `0.9510,0.9700,0.9755,0.9815,0.9855`; latency mean = `3.98ms,4.49ms,5.11ms,7.01ms,9.53ms`; storage total `818.0 MiB`.
- Key 100k IVF/RaBitQ results: build index `11.24s`, total `287.81s`; recall@10 nprobe `8,16,24,32,48,64` = `0.7490,0.8205,0.8595,0.8820,0.9015,0.9175`; latency mean = `3.56ms,5.32ms,7.01ms,8.90ms,12.3ms,15.8ms`; storage total `1.6 GiB`.
- Key 100k HNSW results: `m=8` build `127.96s`, `m=16` build `156.37s`, total `550.81s`; recall@10 ef `40,80,120,200,400` = `0.7470,0.8540,0.8950,0.9245,0.9385`; latency mean = `3.30ms,4.49ms,5.35ms,7.29ms,12.1ms`; storage total `1.8 GiB`.
- Key 100k DiskANN results: build index `392.77s`, total `659.55s`; recall@10 list_size `64,128,200,400,800` = `0.9190,0.9640,0.9755,0.9835,0.9865`; latency mean = `4.38ms,5.08ms,5.85ms,8.76ms,13.6ms`; storage total `1.6 GiB`.

## Focused Local Intel HNSW Runs

- 50k focused config: `artifacts/task67-local-50k-hnsw-current-shape-suite.json`.
- 50k focused results: `artifacts/local/current-shape-50k/results.jsonl`.
- 100k focused config: `crates/ecaz-cli/suites/current/intel-local.json`.
- 100k focused results: `artifacts/local/current-intel-local/results.jsonl`.
- Surface: isolated one-index-per-table prefixes `task67_current_shape_real50k_hnsw` and `current_intel_real100k_hnsw`.
- Key focused 50k HNSW: build index `805.10s`, total `958.28s`; recall@10 ef `40,80,120,200,400` = `0.8650,0.9110,0.9185,0.9315,0.9385`; latency mean = `16.9ms,23.9ms,30.3ms,42.6ms,73.9ms`; storage total `860.0 MiB`.
- Key focused 100k HNSW: build index `2249.31s`, total `2577.45s`; recall@10 ef `40,80,120,200,400` = `0.7470,0.8540,0.8950,0.9245,0.9385`; latency mean = `17.9ms,26.0ms,32.7ms,46.7ms,80.4ms`; storage total `1.7 GiB`.

## AWS Cleanup

- Command: `target/debug/ecaz cloud pause --profile 1m --log-file artifacts/preflight/1m-pause-after-benchmarks.log`.
- Verification: `artifacts/preflight/1m-status-after-pause.log`.
- Result: profile `1m` is paused; running cost reported as `~$0.00/hr`, retained storage `~$8.00/mo`.
