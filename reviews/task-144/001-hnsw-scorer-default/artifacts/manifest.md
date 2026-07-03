# Task 144 packet 001 — HNSW exact vs int8_approx mode A/B: manifest

- Dylib + runner: `815518d82` (installed main code state; in-suite
  `precheck-build-sha.log` also records the HNSW mode default = `exact`).
  Single binary; the mode is a session GUC in both cells.
- Host/lane: m5-local PG18 (socket /Users/peter/.pgrx:28818), 2026-07-03.
- Fixture: the Task 141 HNSW tables (`task141_hnsw_real{10k,50k,100k}`,
  m=16, ef_construction=128, TurboQuant no-QJL 4-bit, dbpedia 1536-dim).
- A/B axis: `ec_hnsw.turboquant_exact_score_mode` = `exact` (current
  default) vs `int8_approx` (the Task 141 SDOT kernel). Recall on the
  registered ec_hnsw default sweep [40,64,100,128,160,200]; latency at
  ef 64 and 100; storage per table. queries 64/48/32, iters 64/48/32.
- Command: `ecaz bench suite run --config
  reviews/task-144/001-hnsw-scorer-default/task144-hnsw-mode-ab-suite.json ...`
  (16/16 steps, exit 0).

## Key result lines

### Recall@10 (exact → int8_approx)

| scale | ef40 | ef64 | ef100 | ef128 | ef160 | ef200 |
|---|---|---|---|---|---|---|
| 10k | 0.8734→0.8719 | 0.9219→0.9203 | 0.9516→0.9500 | 0.9656→0.9641 | 0.9656→0.9641 | 0.9672→0.9656 |
| 50k | 0.9062→0.9042 | 0.9375→0.9333 | 0.9437→0.9396 | 0.9437→0.9396 | 0.9479→0.9437 | 0.9479→0.9437 |
| 100k | 0.7906→0.7875 | 0.8781→0.8750 | 0.9094→0.9062 | 0.9094→0.9062 | 0.9094→0.9062 | 0.9187→0.9156 |

Max dip 0.42 pp (50k ef64/ef100) — the same noise-band magnitude as the
IVF evidence (Tasks 136/143); the Task 98 recall caution does not
reproduce beyond noise on these corpora.

### Latency mean/p50 (ms)

| scale | exact ef64 | int8 ef64 | delta | exact ef100 | int8 ef100 | delta |
|---|---|---|---|---|---|---|
| 10k | 0.65/0.59 | 0.66/0.60 | +1.5% | 0.87/0.81 | 0.84/0.77 | −3.4% |
| 50k | 0.83/0.77 | 0.75/0.71 | −9.6% | 0.99/0.95 | 0.97/0.91 | −2.0% |
| 100k | 1.09/1.03 | 0.98/0.94 | −10.1% | 1.31/1.24 | 1.31/1.23 | 0% |

### Storage

Unchanged (mode is query-side): `storage-real*.log`.

## Not committed

- `truth-cache/` (gitignored).
