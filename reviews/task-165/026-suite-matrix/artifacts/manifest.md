# Packet 026 — ec_distann single-node suite matrix 10k/50k/100k (artifacts manifest)

- head SHA: (see request.md; code unchanged from packet 025 df9aad95 — this is a
  measurement packet against the release build)
- task bucket / packet: reviews/task-165/026-suite-matrix
- surface: single-node `ec_distann` index, **release** `.so`
  (`target/release/libecaz.so`, installed at
  `~/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`), pgrx PG18 (port 28818),
  fresh database `distann_t165` (the shared `tqvector_bench` carries a stale ecaz
  extension without the ec_distann AM; a fresh DB avoids a destructive
  `DROP EXTENSION CASCADE`).
- isolation: one index per corpus prefix (`distann_real{10k,50k,100k}`), not a
  shared table.
- corpus: staged real DBpedia at `data/staged-current/ec_real_{10k,50k,100k}_*`
  (10k sha256 `c67c5810…`; 50k, 100k per `data/staged-current/*_manifest.json`).
- runner: `ecaz bench suite` (FR-038). Config is bespoke because `ec_distann` is
  the new 5th AM and is **not** in the canonical `intel-local.json` lane config
  (which carries ec_hnsw/ec_ivf/ec_diskann/ec_spire). The sweep is the registered
  `ec_distann` `default_sweep` `[16,32,64,100,200]` verbatim (profiles.rs), k=10,
  queries_limit=200, bits=4, seed=42.

## Command

```
ecaz bench suite run \
  --config reviews/task-165/026-suite-matrix/artifacts/distann-suite.json \
  --host /home/peter/.pgrx --port 28818 --database distann_t165
```

Config: `artifacts/distann-suite.json` (13 steps: precheck + 3 scales ×
load/recall/latency/storage). Canonical artifacts: `artifacts/suite-manifest.json`
+ `artifacts/results.jsonl`; per-step logs `artifacts/{recall,latency,storage,load}-{scale}-distann.log`.

## Results — recall@10 (top-k sweep), single-node ec_distann

| scale | tk=16 | tk=32 | tk=64 | tk=100 | tk=200 |
| ----- | ----- | ----- | ----- | ------ | ------ |
| 10k   | 0.9935 | 0.9990 | 0.9995 | 1.0000 | 1.0000 |
| 50k   | 0.9150 | 0.9545 | 0.9840 | 0.9880 | 0.9950 |
| 100k  | 0.8685 | 0.9260 | 0.9650 | 0.9770 | 0.9925 |

## Results — latency p50 / p95 (ms), single-node, warm

| scale | tk=16 | tk=32 | tk=64 | tk=100 | tk=200 |
| ----- | ----- | ----- | ----- | ------ | ------ |
| 10k p50  | 1.71 | 2.43 | 3.73 | 5.56 | 10.0 |
| 50k p50  | 2.37 | 3.05 | 4.88 | 6.95 | 13.2 |
| 100k p50 | 2.54 | 3.33 | 5.24 | 7.21 | 14.2 |
| 100k p95 | 3.36 | 4.14 | 6.45 | 8.71 | 16.9 |

## Results — storage

| scale | index size | index B/row | table+index total |
| ----- | ---------- | ----------- | ----------------- |
| 10k   | 110.6 MiB  | ~11.6 KiB   | 269.4 MiB |
| 50k   | 423.6 MiB  | 8883.7 B    | 1.2 GiB |
| 100k  | 815.2 MiB  | 8548.1 B    | 2.3 GiB |

Index build times: 10k 13.8s, 50k 151.2s, 100k 377.7s.

## Reading

- Recall is high and monotone in the ef bar (`ec_distann.top_k`), degrading
  gracefully with scale (10k ≈ 0.99+ by tk=32; 100k reaches 0.9925 at tk=200),
  as expected for a single global Vamana graph at fixed graph_degree=32.
- Latency scales sub-linearly with the ef bar and mildly with corpus size.
- Every cited number traces to `results.jsonl` (per NFR-007); no fabricated
  values.

## Scope note

This packet is the **single-node** ec_distann repo-closeout matrix (recall +
latency + storage at 10k/50k/100k). The **multinode** distinct_recall gate
(TC-044, M4) is proven byte-identical to single-node on the real 3-node fixture
(packets 012/019/025, `suite_recall_gate delta=0.0000 pass=true` + RECALL_RESULT
mismatched_ids=0); packaging it as an `ecaz bench suite` `distann-local-multinode`
step (analogous to `spire-local-multinode`) is a follow-up and is an M4 gate item
(prerequisite: task-138 + task-146 merged).
