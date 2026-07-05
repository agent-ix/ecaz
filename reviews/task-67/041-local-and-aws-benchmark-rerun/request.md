# Task 67 Local and AWS Benchmark Rerun

This packet captures the requested local Intel and AWS benchmark evidence for Task 67.

## Summary

- AWS `1m` DiskANN completed on `m7g.2xlarge`: build `6639.91s`, total `7469.10s`; recall@10 list_size `64,128,200,400,800` = `0.9340,0.9620,0.9690,0.9730,0.9770`; latency means = `4.95ms,6.77ms,8.59ms,13.6ms,22.8ms`.
- AWS `1m` HNSW completed on `m7g.2xlarge`: build `1103.96s`, total `1929.76s`; recall@10 ef `64,128,200,400` = `0.8970,0.9190,0.9270,0.9310`; latency means = `8.04ms,13.1ms,19.0ms,34.0ms`.
- Local Intel full-standard 50k and 100k suite completed for IVF/RaBitQ, HNSW, and DiskANN after repairing the stale local DiskANN catalog state.
- AWS profile `1m` was paused after collection; status reports `~$0.00/hr` running cost.

## Local Standard Sweep Highlights

- 50k DiskANN: build `160.47s`, total `236.28s`; recall@10 list_size `64,128,200,400,800` = `0.9510,0.9700,0.9755,0.9815,0.9855`; latency means = `3.98ms,4.49ms,5.11ms,7.01ms,9.53ms`.
- 100k DiskANN: build `392.77s`, total `659.55s`; recall@10 list_size `64,128,200,400,800` = `0.9190,0.9640,0.9755,0.9835,0.9865`; latency means = `4.38ms,5.08ms,5.85ms,8.76ms,13.6ms`.
- 50k HNSW: `m=8` build `564.72s`, `m=16` build `797.92s`, total `1515.96s`; recall@10 ef `40,80,120,200,400` = `0.8650,0.9110,0.9185,0.9315,0.9225`.
- 100k HNSW: `m=8` build `127.96s`, `m=16` build `156.37s`, total `550.81s`; recall@10 ef `40,80,120,200,400` = `0.7470,0.8540,0.8950,0.9245,0.9385`.
- 50k IVF/RaBitQ: build `638.78s`, total `792.95s`; recall@10 nprobe `8,16,24,32,48,64` = `0.8330,0.8910,0.9095,0.9215,0.9330,0.9375`.
- 100k IVF/RaBitQ: build `11.24s`, total `287.81s`; recall@10 nprobe `8,16,24,32,48,64` = `0.7490,0.8205,0.8595,0.8820,0.9015,0.9175`.

## Evidence

- Manifest: `artifacts/manifest.md`
- AWS 1m DiskANN: `artifacts/aws-1m-diskann-m7g2xlarge/results.jsonl`
- AWS 1m HNSW: `artifacts/aws-1m-hnsw-m7g2xlarge/results.jsonl`
- Local standard initial parsed results: `artifacts/local/results-standard-initial.jsonl`
- Local standard resumed parsed results: `artifacts/local/results-standard-resume-report.jsonl`
- Local standard resume status: `artifacts/local/suite-standard-resume-status.log`
- Local DiskANN catalog repair evidence: `artifacts/local/pg-am-after-diskann-catalog-repair.log`
- AWS pause verification: `artifacts/preflight/1m-status-after-pause.log`

## Notes

The first local standard suite run failed at `load-50k-diskann` because the local PG18 database catalog did not contain `ec_diskann`, even though the installed SQL file defined it. I reinstalled the local extension, restarted PG18, applied the narrow missing DiskANN handler/access-method/opclass catalog repair, verified `pg_am`, and resumed the suite through all remaining 50k/100k steps.
