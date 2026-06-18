# Task 111e Packet 005 Artifact Manifest

- head SHA: `fa5b3a012b69ad5aae146e58f59802de356c0719`
- task bucket: `reviews/task-111e/005-compact-sidecar-rerank`
- purpose: compact table-side rerank representation and placement evidence
- generated: `2026-06-18T10:10:24-07:00`
- lane / fixture: local PG18, real 50k corpus from `data/task111a_real50k/`
- coarse format: RaBitQ-1 dense page-local postings, `rerank=off` candidate frontier
- rerank representations: table-side `f32`, `f16`, `rabitq8`
- placement/read modes: `free`, `random-id`, `tid-sorted`
- table surface: shared fixture prefix `task111e_005_50k_rb1_dense_page_off`

## Commands

```text
target/debug/ecaz bench suite audit --config reviews/task-111e/005-compact-sidecar-rerank/artifacts/task111e-compact-sidecar-suite.json

script -q -c 'target/debug/ecaz bench suite run --config reviews/task-111e/005-compact-sidecar-rerank/artifacts/task111e-compact-sidecar-suite.json --artifact-dir reviews/task-111e/005-compact-sidecar-rerank/artifacts/suite --database task111e_coarse_rerank --host /home/peter/.pgrx --port 28818' reviews/task-111e/005-compact-sidecar-rerank/artifacts/suite-run.log

script -q -c 'target/debug/ecaz bench suite report --manifest reviews/task-111e/005-compact-sidecar-rerank/artifacts/suite/suite-manifest.json' reviews/task-111e/005-compact-sidecar-rerank/artifacts/suite-report.log
```

## Artifacts

| Artifact | Result |
| --- | --- |
| `task111e-compact-sidecar-suite.json` | Suite config with one load step and two sidecar-rerank steps. |
| `suite-audit.log` | Audit passed: 3 steps. |
| `suite-run.log` | Suite completed successfully: 3 completed, 0 failed. |
| `suite-report.log` | Parsed report for load and sidecar-rerank results. |
| `suite/suite-manifest.json` | Runner manifest for the completed suite. |
| `suite/results.jsonl` | Structured load and sidecar-rerank result rows. |
| `suite/load-50k-rb1-dense-page-rerank-off.log` | Fresh 50k RaBitQ-1 dense page-local load. |
| `compact-sidecar-50k-k50-p32.log` | Sidecar rerank matrix for `candidate_k=50`, `nprobe=32`. |
| `compact-sidecar-50k-k100-p32.log` | Sidecar rerank matrix for `candidate_k=100`, `nprobe=32`. |

## Load Cell

The load step used:

```text
storage_format = rabitq
nlists = 64
nprobe = 32
quant_bits = 1
training_sample_rows = 10000
dense_posting_blocks = 1
dense_posting_pack_pages = 1
dense_posting_typed_layout = 1
rerank = off
```

Key load timings from `suite-report.log`:

```text
copy_corpus: 49.040000 s
encode_corpus: 28.220000 s
copy_queries: 0.982720 s
build_index: 38.900000 s
total: 199.000000 s
heap_tuples = 50000
index_tuples = 50000
```

## Sidecar Results

All rows used `queries=100`, `warmup_queries=10`, `k=10`, `nprobe=32`, and
`concurrency=1`.

### candidate_k=50

| Variant | Read mode | Recall@10 | NDCG | Sidecar size | Bytes touched p50 | IO p50 | Score p50 | Sidecar p50 | Total-bound p50 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| f32 | free | 0.9940 | 0.9997 | 292.97 MiB | 300.00 KiB | 0.000 ms | 1.578 ms | 1.578 ms | 252.800 ms |
| f32 | random-id | 0.9940 | 0.9997 | 292.97 MiB | 300.00 KiB | 19.750 ms | 3.142 ms | 22.957 ms | 273.802 ms |
| f32 | tid-sorted | 0.9940 | 0.9997 | 292.97 MiB | 300.00 KiB | 1.783 ms | 3.127 ms | 4.938 ms | 256.146 ms |
| f16 | free | 0.9940 | 0.9997 | 146.48 MiB | 150.00 KiB | 0.000 ms | 2.667 ms | 2.667 ms | 253.907 ms |
| f16 | random-id | 0.9940 | 0.9997 | 146.48 MiB | 150.00 KiB | 19.512 ms | 5.128 ms | 24.627 ms | 276.156 ms |
| f16 | tid-sorted | 0.9940 | 0.9997 | 146.48 MiB | 150.00 KiB | 1.416 ms | 5.097 ms | 6.563 ms | 258.162 ms |
| rabitq8 | free | 0.9460 | 0.9994 | 73.81 MiB | 75.59 KiB | 0.000 ms | 0.729 ms | 0.729 ms | 251.959 ms |
| rabitq8 | random-id | 0.9460 | 0.9994 | 73.81 MiB | 75.59 KiB | 18.894 ms | 0.635 ms | 19.545 ms | 271.158 ms |
| rabitq8 | tid-sorted | 0.9460 | 0.9994 | 73.81 MiB | 75.59 KiB | 0.950 ms | 0.609 ms | 1.577 ms | 252.780 ms |

### candidate_k=100

| Variant | Read mode | Recall@10 | NDCG | Sidecar size | Bytes touched p50 | IO p50 | Score p50 | Sidecar p50 | Total-bound p50 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| f32 | free | 0.9960 | 0.9997 | 292.97 MiB | 600.00 KiB | 0.000 ms | 3.078 ms | 3.078 ms | 274.915 ms |
| f32 | random-id | 0.9960 | 0.9997 | 292.97 MiB | 600.00 KiB | 33.676 ms | 6.212 ms | 39.915 ms | 312.450 ms |
| f32 | tid-sorted | 0.9960 | 0.9997 | 292.97 MiB | 600.00 KiB | 2.975 ms | 6.195 ms | 9.225 ms | 280.753 ms |
| f16 | free | 0.9960 | 0.9997 | 146.48 MiB | 300.00 KiB | 0.000 ms | 5.371 ms | 5.371 ms | 277.031 ms |
| f16 | random-id | 0.9960 | 0.9997 | 146.48 MiB | 300.00 KiB | 53.006 ms | 11.739 ms | 65.547 ms | 331.328 ms |
| f16 | tid-sorted | 0.9960 | 0.9997 | 146.48 MiB | 300.00 KiB | 1.921 ms | 10.059 ms | 12.036 ms | 283.989 ms |
| rabitq8 | free | 0.9460 | 0.9994 | 73.81 MiB | 151.17 KiB | 0.000 ms | 1.140 ms | 1.140 ms | 272.859 ms |
| rabitq8 | random-id | 0.9460 | 0.9994 | 73.81 MiB | 151.17 KiB | 31.732 ms | 1.098 ms | 32.826 ms | 305.094 ms |
| rabitq8 | tid-sorted | 0.9460 | 0.9994 | 73.81 MiB | 151.17 KiB | 1.214 ms | 1.053 ms | 2.273 ms | 274.243 ms |

## Interpretation

- `f16` is the best compact table-side representation from this slice. It
  matches `f32` recall and NDCG at both candidate widths while halving sidecar
  size and bytes touched.
- `f16` currently costs more scorer CPU than `f32` in this harness. At
  `candidate_k=100` with `tid-sorted`, sidecar p50 is 12.036 ms for `f16`
  versus 9.225 ms for `f32`.
- `rabitq8` is fastest and smallest, but recall@10 drops to 0.9460 at both
  candidate widths. Reject it as the immediate high-recall compact rerank
  representation for Task 111e.
- `tid-sorted` table-side reads dominate `random-id` reads. At
  `candidate_k=100`, IO p50 falls from 33.676 ms to 2.975 ms for `f32`, from
  53.006 ms to 1.921 ms for `f16`, and from 31.732 ms to 1.214 ms for
  `rabitq8`.
- Absolute candidate SQL p50 in this packet is slower than earlier packet 001
  frontier runs, so this packet should be used for representation and placement
  comparison, not as final promotion latency evidence.
