# Task 50 M5 HNSW Baseline Manifest

Post-burndown HNSW reference numbers on the M5 Pro laptop. Closes the
remaining Task 50 §Exit Criteria gate ("No bench lane regresses beyond
tolerance") for HNSW and establishes a new M5 baseline that subsequent
Task 50 / Task 33 packets can compare against.

The pre-burndown WSL2 reference at `benchmarks/task-50-local-baseline/`
is **not** directly comparable (different host: WSL2 i9-10900K vs.
Apple M5 Pro arm64). These numbers stand as the new M5 baseline.

## Head and host

| Field | Value |
| --- | --- |
| HEAD SHA | `18acf379a` (post-merge `ebb022a7a`, after IVF build-fix `54a2c1409`) |
| Captured | `2026-05-23` (America/Los_Angeles) |
| Host | Peters-MBP (Apple Silicon) |
| CPU | Apple M5 Pro |
| Memory | 64 GiB |
| OS | macOS 26.4.1 (Darwin 25.4.0 arm64) |
| PostgreSQL | 18 (pgrx local install, socket `/Users/peter/.pgrx`, port 28818) |
| Extension build | `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config` — see `artifacts/pgrx-install.log` |

## Scope

HNSW only. IVF / SPIRE / DiskANN deferred to follow-up packets per
the post-burndown scope-lock (`reviews/task-50/448-.../feedback/2026-05-22-02-reviewer.md`).

Corpora: `ec_real_10k` (10k corpus / 200 queries) and `ec_real_100k`
(100k / 1000) from the local DBpedia/OpenAI3 fixtures at
`fixtures/m5_diskann_real{10k,100k}/`. 1536-dim, ip metric.

HNSW build sweep: `m ∈ {8, 16}` at 10k, `m = 16` at 100k, `ef_construction = 128`.
HNSW scan sweep: `ef_search ∈ {40, 80, 120, 200, 400}`, `k = 10`.

## Re-run

```sh
/Users/peter/.cargo/bin/ecaz \
  --host /Users/peter/.pgrx --port 28818 --database tqvector_bench \
  bench suite run --config benchmarks/task-50-m5-hnsw-baseline/suite.json \
  --log-file benchmarks/task-50-m5-hnsw-baseline/artifacts/suite-run.log
```

The suite expands to **8 steps** (`load`, `recall`, `latency`, `storage`
× 2 sizes). Connection settings flow from the parent ecaz invocation;
the suite config carries only `pg`, `socket_dir`, `seed` defaults.

## Artifacts

| Step | Log |
| --- | --- |
| pgrx install (release, pg18) | `artifacts/pgrx-install.log` |
| load 10k HNSW | `artifacts/corpus-load-ec_real_10k-hnsw.log` |
| recall 10k HNSW | `artifacts/recall-ec_real_10k-hnsw.log` |
| latency 10k HNSW | `artifacts/latency-ec_real_10k-hnsw.log` |
| storage 10k HNSW | `artifacts/storage-ec_real_10k-hnsw.log` |
| load 100k HNSW | `artifacts/corpus-load-ec_real_100k-hnsw.log` |
| recall 100k HNSW | `artifacts/recall-ec_real_100k-hnsw.log` |
| latency 100k HNSW | `artifacts/latency-ec_real_100k-hnsw.log` |
| storage 100k HNSW | `artifacts/storage-ec_real_100k-hnsw.log` |
| suite manifest | `artifacts/suite-manifest.json` |
| structured results | `artifacts/results.jsonl` |
| full suite stdout/stderr | `artifacts/suite-run.log` |

## Index isolation

Each `(corpus, AM, storage)` variant is loaded under an isolated PG
prefix per the NFR-007 storage rule. This packet loads HNSW only, so
prefixes are `ec_real_10k_hnsw` and `ec_real_100k_hnsw`; no shared-table
planner crossing.

## Key results

Suite status: **8/8 succeeded** (`completed=8 failed=0 skipped=0 dry_run=0
missing_artifacts=0 stale=0`). Full per-step status: `artifacts/suite-manifest.json`.
Structured rows: `artifacts/results.jsonl`.

### Recall (m=16 default, k=10, ip metric)

10k corpus (200 queries × 10 NN trials each):

| ef_search | recall@10 | ci95 low | ci95 high | ndcg@10 | mean q-time |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 0.9040 | 0.8903 | 0.9161 | 0.9617 | 0.65 ms |
| 80  | 0.9530 | 0.9428 | 0.9614 | 0.9821 | 0.90 ms |
| 120 | 0.9605 | 0.9510 | 0.9682 | 0.9866 | 0.92 ms |
| 200 | 0.9775 | 0.9700 | 0.9831 | 0.9933 | 1.11 ms |
| 400 | 0.9950 | 0.9908 | 0.9973 | 0.9996 | 1.72 ms |

100k corpus (1000 queries × 10 NN trials each):

| ef_search | recall@10 | ci95 low | ci95 high | ndcg@10 | mean q-time |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 0.7426 | 0.7339 | 0.7511 | 0.9696 | 1.05 ms |
| 80  | 0.8506 | 0.8435 | 0.8575 | 0.9862 | 1.65 ms |
| 120 | 0.8973 | 0.8912 | 0.9031 | 0.9915 | 2.10 ms |
| 200 | 0.9414 | 0.9366 | 0.9458 | 0.9962 | 2.83 ms |
| 400 | 0.9676 | 0.9639 | 0.9709 | 0.9987 | 4.79 ms |

### Latency (concurrency=1, iterations=1000 per ef_search)

10k corpus:

| ef_search | mean | stddev | p50 | p95 | p99 | max |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 0.59 ms | 0.12 ms | 0.57 ms | 0.74 ms | 0.88 ms | 3.32 ms |
| 80  | 0.93 ms | 0.15 ms | 0.89 ms | 1.15 ms | 1.37 ms | 3.59 ms |
| 120 | 0.85 ms | 0.15 ms | 0.82 ms | 1.08 ms | 1.28 ms | 3.32 ms |
| 200 | 1.09 ms | 0.16 ms | 1.05 ms | 1.36 ms | 1.58 ms | 3.88 ms |
| 400 | 1.72 ms | 0.21 ms | 1.69 ms | 2.02 ms | 2.26 ms | 4.67 ms |

100k corpus:

| ef_search | mean | stddev | p50 | p95 | p99 | max |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 1.00 ms | 0.35 ms | 0.92 ms | 1.61 ms | 2.25 ms | 3.93 ms |
| 80  | 1.67 ms | 0.51 ms | 1.57 ms | 2.56 ms | 3.50 ms | 5.36 ms |
| 120 | 2.09 ms | 0.56 ms | 1.96 ms | 3.11 ms | 4.19 ms | 6.28 ms |
| 200 | 2.92 ms | 0.65 ms | 2.80 ms | 4.03 ms | 5.54 ms | 8.69 ms |
| 400 | 5.02 ms | 0.82 ms | 4.92 ms | 6.38 ms | 7.98 ms | 12.8 ms |

### Storage (index size on disk)

| corpus | rows | index | reloptions | size | bytes/row |
| --- | ---: | --- | --- | ---: | ---: |
| ec_real_10k_hnsw  | 10k  | ec_real_10k_hnsw_m8_idx  | m=8, ef_construction=128  | 11.8 MiB  | 1235.4 |
| ec_real_10k_hnsw  | 10k  | ec_real_10k_hnsw_m16_idx | m=16, ef_construction=128 | 13.0 MiB  | 1366.4 |
| ec_real_100k_hnsw | 100k | ec_real_100k_hnsw_m16_idx | m=16, ef_construction=128 | 130.2 MiB | 1365.4 |

bytes/row is nearly identical between 10k (m=16) and 100k (m=16), as expected
for HNSW once the metadata page is amortized.

## Cross-references

- Task definition: `plan/tasks/50-unsafe-structural-reduction.md`
  §Performance Gate / §Exit Criteria.
- Closeout that named this gate as the outstanding item:
  `reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md`
  §"Bench gate".
- Pre-burndown reference baseline (different host, NOT a direct A/B):
  `benchmarks/task-50-local-baseline/manifest.md`.
- Storage / provenance rule: `spec/non-functional/NFR-007-benchmark-provenance.md`.
