# IVF 111g–115 — Benchmark Attribution + Rerank Matrix

Empirical evidence for the IVF lane `111g → 112 → 113 → 115`, which was merged to
`main` with **no benchmarks**. This packet measures each change's effect (A/B)
and the rerank/quant matrix, so promote/iterate/abandon verdicts rest on facts.
Conforms to `spec/non-functional/NFR-007-benchmark-provenance.md`.

## Lane / host / corpus

- **Lane:** intel-local (this Intel desktop **is** the bench host).
- **Host PG:** pgrx PG18, socket `/home/peter/.pgrx`, port 28818, DB `ivf_attr_bench`.
- **Backend build:** release (`ecaz_build_profile()` → `release`), `.so` SHA verified
  == `target/release/libecaz.so` before each run.
- **Corpus:** real DBpedia (qdrant openai-3-large, dim 1536), one-index-per-table.
  Staged at `data/staged-current/` (symlinks):
  - 10k — `data/task106_intel_dbpedia_staged` — corpus SHA `c67c5810…`, 10000 rows / 200 q
  - 50k — `data/task111a_real50k` — corpus SHA `56023baa…`, 50000 rows / 1000 q
  - 100k — `data/task106_full_sweep_100k` — corpus SHA `07275cfd…`, 100000 rows / 1000 q
- **Common knobs:** k=10, nprobe sweep `[8,16,32,64,128,200]` (extended from the
  registered `[8,16,24,32,48,64]` default so curves reach ~0.999 and matched-recall
  comparison is possible — see FINDINGS Finding 2), queries_limit=200,
  iterations=200, concurrency=1, cache_state=post_recall_warm, rerank_width=64;
  nlists per scale 100 / 224 / 316 (explicit, held constant for attribution).
- **HEAD backend SHA (all 8 configs):** `67e1534cfcf82f43…` (release, f16-fixed),
  recorded in each `artifacts/head-*/suite-manifest.json`. The committed
  `artifacts/head-<config>/{results.jsonl,suite-manifest.json,*.log}` are the
  decision-grade evidence behind the verdict table below + FINDINGS.md.

## Layers

| Layer | Config | What it isolates |
|---|---|---|
| Historical attribution | `configs/constant-rabitq.json` at each merge commit | net effect of each merge on the stable plain-RaBitQ path |
| 111g rerank reps | `configs/rerank-format-matrix.json` (HEAD) | coarse_rerank rerank_format f32 / f16 (halfvec) / rabitq4 |
| RaBitQ bit-depth | `configs/quant-bits-matrix.json` (HEAD) | quant_bits 1 / 2 / 4 / 8 |
| 113 posting prune | `configs/prune-ab.json` (HEAD) | `ec_ivf.posting_bound_prune` off vs on |
| 112 lazy rerank | `configs/lazy-ab.json` (HEAD) | `ec_ivf.lazy_heap_rerank` off vs on |
| 115 residual | `configs/residual-ab.json` (HEAD) | plain vs `rabitq_residual=1` |

Historical merge commits: baseline `99dc70e53` → 111g `61fd84f95` → 112
`6d60eec50` → 113 `9ddb3be7c` → 115 `93a015ecc` (HEAD). Each historical point:
worktree checkout → `cargo pgrx install --release` → fresh DB + `CREATE EXTENSION`
→ run `constant-rabitq.json`; `.so` SHA verified each time.

## Artifacts

`artifacts/<layer-or-commit>/` each holds `suite-manifest.json` + `results.jsonl`
+ per-step `.log`. Result fields: `recall@k`, `ndcg@k`, latency `p50/p95/p99/mean`,
storage size/per-row, and posting/heap/prune counters where emitted.

## Verdict table (per change)

Measured @100k on the fixed `.so` (full curves + 10/50k in FINDINGS.md and the
committed `artifacts/head-*/results.jsonl`). Reconciled with codex code review.

| Change | Result @100k | Verdict | Evidence |
|---|---|---|---|
| 111g table f16(fixed) vs f32 | recall 0.9975 vs 0.9985; ~6% slower; same 25.4 MiB | no benefit vs f32 | `head-rerank-format-matrix` |
| 111g table rabitq4 | recall 0.942 (lower) | worse — don't use | `head-rerank-format-matrix` |
| 111g **index-side** f16/rq4 | p50 ~540 ms (flat) vs 5–13 ms table; idx 416 vs 25 MiB | **BROKEN, O(N) → ADR-079** | `head-sidecar-index-placement` |
| 112 lazy on/off | recall byte-identical; latency neutral | **inert — no win** | `head-lazy-ab` |
| 113 row prune on/off | recall identical; p50 16.7 vs 17.4 / 43.6 vs 45.5 ms | recall-safe, **+4%** keep | `head-prune-ab` |
| 113 dense prune on/off | recall identical; ~3% slower | recall-safe but inert → gate to row | `head-dense-prune-ab` |
| 115 residual plain/on | recall identical; ~9% slower; same size | **no win** (masked by exact rerank) | `head-residual-ab` |
| quant_bits 1/2/4/8 | recall all ≈0.999; idx 32.7/52.5/90.4/198.7 MiB; p50 7.8/53.3*/16.6/17.8 ms | **1-bit sweet spot** | `head-quant-bits-matrix` |

\* qb2 p50=53 ms anomaly (likely unoptimized 2-bit scan). Full per-nprobe curves
and the overall conclusion: `FINDINGS.md`.

## Re-run

```
export PGHOST=/home/peter/.pgrx PGPORT=28818 PGDATABASE=ivf_attr_bench
target/release/ecaz bench suite run --config benchmarks/ivf-111g-115-attribution/configs/<cfg>.json \
  --artifact-dir benchmarks/ivf-111g-115-attribution/artifacts/<layer>
```

NFR-007: no fabricated numbers; every cited value traces to a `results.jsonl` line.
