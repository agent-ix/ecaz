# Results Summary

## 1M Key Results

- Load/index total: 12357.42s
- DiskANN build: 11682.44s
- Rows: 990000
- Storage total: 15.9 GiB
- DiskANN index: 455.1 MiB, 482.0 B/row

| list_size | recall@10 | mean | p50 | p95 | p99 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 0.9385 | 3.90 ms | 3.72 ms | 5.80 ms | 7.69 ms |
| 128 | 0.9655 | 5.41 ms | 5.20 ms | 8.20 ms | 9.65 ms |
| 200 | 0.9735 | 6.98 ms | 6.68 ms | 10.8 ms | 12.7 ms |
| 400 | 0.9800 | 11.4 ms | 11.3 ms | 18.2 ms | 21.3 ms |
| 800 | 0.9825 | 19.9 ms | 19.7 ms | 30.9 ms | 35.6 ms |

## Provenance

- 10k/50k/100k logs: `artifacts/s3-final-224632/`
- 1M logs: `artifacts/one-million-resume/`
- Space cleanup and failed 1M attempts are retained under `artifacts/diagnose-*`, `artifacts/drop-partial-1m-single/`, `artifacts/cleanup-duplicate-fetches/`, and `artifacts/s3-1m-resume-230204/`.
