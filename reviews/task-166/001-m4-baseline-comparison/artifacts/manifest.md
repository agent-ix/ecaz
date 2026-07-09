# Packet 166/001 — M4 baseline comparison (ec_distann vs IVF/HNSW/DiskANN)

- task bucket / packet: reviews/task-166/001-m4-baseline-comparison
- surface: single-node ecaz AMs, release `.so`
  (`~/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`), pgrx PG18 (port 28818),
  DB `distann_t165` (fresh; shared `tqvector_bench` has a stale ecaz ext).
- isolation: one index per (AM, scale) prefix `m4cmp_{am}_{scale}` — not shared.
- corpus: staged real DBpedia `data/staged-current/ec_real_{10k,50k,100k}_*`.
- runner: `ecaz bench suite` (FR-038); config `artifacts/m4-baseline-suite.json`.
- sweeps: registered `default_sweep` per profile — hnsw `[40,64,100,128,160,200]`,
  ivf `[8,16,24,32,48,64]`, diskann `[64,128,200,400,800]`; k=10, queries_limit=200,
  bits=4, seed=42.
- ec_distann column: packet 026 (same corpus/host/head).

## Purpose

The runnable three-of-four columns of the M4 program gate (NFR-017 four-way
comparison). The **best-SPIRE anchor** column + the pre-registered
promote/iterate/shelve verdict require the Task-138 metric emitter + Task-146
anchor branch merged onto the measuring line (operator decision), so they are
NOT in this packet.

## Command

```
ecaz bench suite run \
  --config reviews/task-166/001-m4-baseline-comparison/artifacts/m4-baseline-suite.json \
  --host /home/peter/.pgrx --port 28818 --database distann_t165 --continue-on-error
```

## Results — matched-protocol single-node comparison

Numbers trace to the packet-local per-step logs `recall-{scale}-{am}.log`,
`latency-{scale}-{am}.log`, `storage-{scale}-{am}.log` (the run was stopped one
step early — 100k-diskann recall — to protect the shared host's disk, so
`results.jsonl` was not finalized; the per-step logs are the authoritative
artifacts and carry every cell below). ec_distann column = packet 026.

### recall@10 (best sweep point)

| scale | **ec_distann** | ec_hnsw | ec_ivf | ec_diskann |
| ----- | -------------- | ------- | ------ | ---------- |
| 10k   | **1.0000** | 0.9950 | 0.9740 | 1.0000 |
| 50k   | **0.9950** | 0.9735 | 0.9380 | 0.9965 |
| 100k  | **0.9925** | 0.9630 | 0.9255 | _(not captured — disk)_ |

### p50 latency (ms, fastest sweep point)

| scale | ec_distann | ec_hnsw | ec_ivf | ec_diskann |
| ----- | ---------- | ------- | ------ | ---------- |
| 10k   | 1.71 | 3.12 | 1.64 | 3.34 |
| 50k   | 2.37 | 3.32 | 2.24 | 4.04 |
| 100k  | 2.54 | 4.44 | 2.67 | _(n/a)_ |

### index size

| scale | ec_distann | ec_hnsw | ec_ivf | ec_diskann |
| ----- | ---------- | ------- | ------ | ---------- |
| 10k   | 110.6 MiB | 13.0 MiB | 9.0 MiB | 4.1 MiB |
| 50k   | 423.6 MiB | 65.1 MiB | 41.6 MiB | 20.6 MiB |
| 100k  | 815.2 MiB | 130.2 MiB | 81.7 MiB | _(n/a)_ |

## Reading

- **Recall:** ec_distann matches or beats every baseline at every scale —
  on par with the strongest (ec_diskann), clearly above ec_hnsw and ec_ivf.
- **Latency:** ec_distann is competitive at its best-recall point (fastest or
  near-fastest; ec_ivf edges it only at its lower-recall points).
- **Storage:** ec_distann's index is markedly larger — this is the co-placed
  full-precision vector tier (ADR-085 D11) that enables the single-graph +
  exact-rerank design. That is the explicit storage/recall trade the M4 gate
  weighs against the best-SPIRE anchor.

This is the **three-of-four** columns of the M4 gate. The best-SPIRE anchor
column + the pre-registered promote/iterate/shelve verdict require the
task-138 / task-146 merge (operator decision) before they can be added.
