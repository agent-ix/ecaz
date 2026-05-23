# Task-50 Local Bench vs Baseline — IVF/RaBitQ + SPIRE/RaBitQ (k=10, 10k)

Packet: `reviews/task-50/397-current-head-pg18-rabitq-ivf-spire-sweep`
Current head: `e81dcf8fd16cc02ddf4e88b7861af94c5f80ff48` (branch `task-50-unsafe-closeout`)
Baseline head: `cc06046177ce63f969da51150d66a83260efe4d7` (2026-05-19,
`benchmarks/task-50-local-baseline/`)
Host: `DESKTOP-BMB4AFO` (WSL2, i9-10900K, no AVX-512) — same as baseline.
PG: 18.3 / pgrx local on `localhost:28818`, db `tqvector_bench`.
DiskANN/HNSW lanes were intentionally out of scope for this smoke pass; only
the RaBitQ-bearing access methods (IVF, SPIRE) were exercised.

## Run Surface

- Suite config: `artifacts/rabitq-ivf-spire-local-suite.json`
- Manifest: `artifacts/suite-manifest.json`
- Result rows: `artifacts/results.jsonl`
- Suite log: `artifacts/ecaz-bench-suite-run.log`
- Per-step logs:
  - `artifacts/ivf-rabitq-10k-recall-k10.log`
  - `artifacts/ivf-rabitq-10k-latency-k10-c1.log`
  - `artifacts/spire-rabitq-10k-recall-k10.log`
  - `artifacts/spire-rabitq-10k-latency-k10-c1.log`

Sweep: `nprobe ∈ {8, 16}`; `k=10`; `bits=4`; `seed=42`; `--force-index` on.
Recall queries: 50 (baseline 200). Latency iterations: 50 / concurrency 1
(baseline 1000 / concurrency 1). SPIRE deliberately capped at 10k per the
≤25k guidance.

## Recall — comparable to baseline (no quality regression)

| Lane              | nprobe | recall@k now | recall@k baseline | Δ |
| ----------------- | ------ | ------------ | ----------------- | --- |
| IVF/RaBitQ 10k    | 8      | 0.9720       | 0.9735            | −0.0015 (within CI: 0.9536–0.9832) |
| IVF/RaBitQ 10k    | 16     | 0.9780       | 0.9780            | 0.0000 |
| SPIRE/RaBitQ 10k  | 8      | 0.9880       | 0.9920            | −0.0040 (within CI: 0.9741–0.9945) |
| SPIRE/RaBitQ 10k  | 16     | 0.9960       | 0.9985            | −0.0025 (within CI: 0.9855–0.9989) |

ndcg@k is ≥ 0.999 across the board. Recall is statistically unchanged from
baseline — task-50 unsafe-block consolidation does **not** appear to have
broken IVF or SPIRE scan correctness on the 10k corpus.

## Latency — large regression vs baseline

Mean and p50 per-query times (smaller is better):

| Lane              | nprobe | now mean | baseline mean | now p50 | baseline p50 | ratio (p50) |
| ----------------- | ------ | -------- | ------------- | ------- | ------------ | ----------- |
| IVF/RaBitQ 10k    | 8      | 48.9 ms  | 5.00 ms       | 48.7 ms | 5.00 ms      | **~9.7×**   |
| IVF/RaBitQ 10k    | 16     | 77.0 ms  | 8.86 ms       | 75.9 ms | 8.86 ms      | **~8.6×**   |
| SPIRE/RaBitQ 10k  | 8      | 225.2 ms | 38.0 ms       | 226.2 ms| 39.0 ms      | **~5.8×**   |
| SPIRE/RaBitQ 10k  | 16     | 382.0 ms | 70.7 ms       | 396.5 ms| 74.2 ms      | **~5.3×**   |

Tail (p95/p99) tracks the same multiple — not a "rare slow path"; the whole
distribution shifted right. Sample-size differences (50 iters vs 1000) cannot
plausibly explain a 5–10× shift in mean and median.

## Identified Issues

### 1. **Critical — installed extension is a debug build, not release**

This is the leading explanation for the latency regression and almost
certainly the dominant factor. Evidence:

- `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
  is **248,904,952 bytes**, mtime `2026-05-21 20:50:34` — byte-exact match
  with `target/debug/libecaz.so` (also 248,904,952 bytes, mtime 20:48 today).
- `target/release/libecaz.so` is **17,473,672 bytes**, mtime `2026-05-20
  07:56` — the day the baseline was captured. The baseline numbers
  (5 ms IVF / 38 ms SPIRE at nprobe=8) are consistent with release SIMD
  scoring on this host; the current 49 ms / 225 ms numbers are consistent
  with debug-mode RaBitQ scoring (no inlining, bound checks, no SIMD codegen
  on hot loops).
- `readelf -p .comment` on the installed `.so` shows `with debug_info, not
  stripped`.

Action: rebuild and reinstall the extension via the repo's release recipe
(`Makefile` target uses `cargo pgrx install --sudo --release`) and rerun this
suite before drawing any conclusions about the task-50 unsafe-block work.
Until then **the latency regression cannot be attributed to source changes
on this branch**.

### 2. Apples-vs-oranges sample sizes vs baseline

Recall used `queries_limit=50` here vs `200` in the baseline (default).
Latency used `iterations=50` here vs `1000`. Recall CIs are wide enough that
no quality conclusion would be safe at this sample size if recall *had*
moved. The plotted CI95 ranges are roughly ±0.015 (recall) here; baseline CIs
were ~±0.006. For the AWS optimization gate this should be bumped back to
`queries_limit=200, iterations=1000` to match baseline.

### 3. Connection mode drift — UDS → TCP

Baseline used `--host /home/peter/.pgrx` (Unix socket); this run used
`--host localhost` (TCP). The handoff bench command pre-specified TCP. UDS
vs TCP costs ~tens of microseconds per round-trip — negligible at these
millisecond scales, but worth pinning down for the AWS comparison so the
local lane stays a clean reference. Recommend reverting to socket for local.

### 4. SPIRE absolute latency is high even at baseline

Even at the baseline 38 ms / nprobe=8, SPIRE on the 10k corpus is ~7× slower
than IVF/RaBitQ on the same data with comparable recall. That gap was
present before task-50 work and is not a regression — but it is the headline
fact for any "is SPIRE ready for AWS optimization?" question. SPIRE's
absolute latency budget at 25k–100k should be tracked explicitly as a
non-regression goal in the next packet.

### 5. SPIRE was not exercised beyond 10k

Per user instruction (≤25k known-safe ceiling). The 25k SPIRE corpus is
loaded with `ec_spire` and ready (see `corpus list`). A 25k pass would be
informative; a >25k pass remains gated on the SPIRE size-fail issue noted in
prior packets.

## Suggested Next Steps (in order)

1. Rebuild the PG18 extension release-mode and reinstall:
   `cargo pgrx install --sudo --release` (or the equivalent Makefile target),
   then `psql` to confirm `regclass`/version sanity.
2. Re-run the same suite, restoring `queries_limit=200, iterations=1000` and
   switching `--host /home/peter/.pgrx`. Land that as a follow-on packet
   under `reviews/task-50/398-…`. The latency rows here should drop back to
   the baseline neighborhood; if they do **not**, that delta is the real
   task-50 regression and warrants a bisect against `cc06046` → `e81dcf8f`
   focused on RaBitQ score paths.
3. Add a 25k SPIRE recall+latency step to the suite (10k → 25k only) and
   compare against the 25k baseline rows in `benchmarks/task-50-local-baseline/`.
4. Only after the local lane is clean should we proceed to AWS optimization.

## Raw evidence summary

`artifacts/results.jsonl` (excerpted):

```
ivf-rabitq-10k-recall-k10   nprobe=8   recall@k=0.9720  mean=51.06 ms
ivf-rabitq-10k-recall-k10   nprobe=16  recall@k=0.9780  mean=75.97 ms
ivf-rabitq-10k-latency-k10  nprobe=8   p50=48.7 ms      p95=66.8 ms
ivf-rabitq-10k-latency-k10  nprobe=16  p50=75.9 ms      p95=98.6 ms
spire-rabitq-10k-recall-k10  nprobe=8  recall@k=0.9880  mean=269.01 ms
spire-rabitq-10k-recall-k10  nprobe=16 recall@k=0.9960  mean=394.05 ms
spire-rabitq-10k-latency-k10 nprobe=8  p50=226.2 ms     p95=286.1 ms
spire-rabitq-10k-latency-k10 nprobe=16 p50=396.5 ms     p95=463.6 ms
```

Baseline reference (`benchmarks/task-50-local-baseline/artifacts/`):

```
recall-ec_real_10k-ivfrabitq      nprobe=8   recall@k=0.9735  mean=5.12 ms
recall-ec_real_10k-ivfrabitq      nprobe=16  recall@k=0.9780  mean=8.63 ms
latency-ec_real_10k-ivfrabitq     nprobe=8   p50=5.00 ms      p95=6.10 ms
latency-ec_real_10k-ivfrabitq     nprobe=16  p50=8.86 ms      p95=10.5 ms
recall-ec_real_10k-spirerabitq    nprobe=8   recall@k=0.9920  mean=44.19 ms
recall-ec_real_10k-spirerabitq    nprobe=16  recall@k=0.9985  mean=74.29 ms
latency-ec_real_10k-spirerabitq   nprobe=8   p50=39.0 ms      p95=50.2 ms
latency-ec_real_10k-spirerabitq   nprobe=16  p50=74.2 ms      p95=84.8 ms
```
