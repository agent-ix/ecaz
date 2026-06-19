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
- **Common knobs:** k=10, nprobe sweep `[8,16,24,32,48,64]`, queries_limit=200,
  iterations=200, concurrency=1, cache_state=post_recall_warm, rerank_width=64;
  nlists per scale 100 / 224 / 316 (explicit, held constant for attribution).

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

_Filled from results once runs complete — promote / iterate / abandon, each cited
to a `results.jsonl` line. Reconciled against codex/gpt5.5 code-review feedback._

| Change | Metric moved? | Direction | Matched-recall latency Δ | Verdict |
|---|---|---|---|---|
| 111g rerank reps (f16/rabitq4 vs f32) | _tbd_ | | | |
| 112 lazy rerank (on vs off) | _tbd_ | | | |
| 113 posting prune (on vs off) | _tbd_ | | | |
| 115 residual (on vs plain) | _tbd_ | | | |
| quant_bits 1/2/4/8 | _tbd_ | | | |

## Re-run

```
export PGHOST=/home/peter/.pgrx PGPORT=28818 PGDATABASE=ivf_attr_bench
target/release/ecaz bench suite run --config benchmarks/ivf-111g-115-attribution/configs/<cfg>.json \
  --artifact-dir benchmarks/ivf-111g-115-attribution/artifacts/<layer>
```

NFR-007: no fabricated numbers; every cited value traces to a `results.jsonl` line.
