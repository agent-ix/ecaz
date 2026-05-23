# Manifest: Task-50 IVF/RaBitQ 25k / 50k / 100k Scale Rerun

- Task bucket: `reviews/task-50`
- Packet: `reviews/task-50/399-ivf-rabitq-scale-rerun`
- Branch: `task-50-unsafe-closeout`
- Source HEAD: `e81dcf8fd16cc02ddf4e88b7861af94c5f80ff48`
- Installed `.so`: 17,202,072 B, mtime `2026-05-21 22:41:10`, byte-identical
  to `target/release/libecaz.so` (release; same install used in packet 398).
- PostgreSQL: 18.3 / pgrx local install, `localhost:28818`, db `tqvector_bench`
- Host: `DESKTOP-BMB4AFO` (WSL2, i9-10900K, no AVX-512) — same host as
  `benchmarks/task-50-local-baseline/` and packets 397 / 398.
- Surfaces: isolated one-index-per-table corpora
  `ec_real_25k_ivfrabitq`, `ec_real_50k_ivfrabitq`, `ec_real_100k_ivfrabitq`,
  all with access method `ec_ivf` (verified via `ecaz corpus list`).
- SPIRE not exercised at any scale in this packet — the `ec_spire` index
  was never built at 50k / 100k (known bug above 25k); 10k SPIRE is
  covered in packet 398.

## Suite

- `ivf-rabitq-scale-suite.json` — checked-in suite config (input).
- 6 steps: {recall, latency} × {25k, 50k, 100k}, full sweep
  `[8,16,24,32,48,64]`, `k=10`, `bits=4`, `seed=42`, `--force-index`,
  concurrency=1.
- Per-step `queries_limit` / `iterations` not overridden → use
  `ecaz bench {recall,latency}` defaults that match the May-19 baseline
  (200 recall queries, 1000 latency iterations).

## Files

- `ivf-rabitq-scale-suite.json` — suite config.
- `ecaz-bench-suite-run.log` — driver log.
- `suite-manifest.json` — suite manifest emitted by the runner.
- `results.jsonl` — authoritative result rows (6 steps × 6 sweep values).
- `ivf-rabitq-25k-recall-k10.log`,
  `ivf-rabitq-25k-latency-k10-c1.log`,
  `ivf-rabitq-50k-recall-k10.log`,
  `ivf-rabitq-50k-latency-k10-c1.log`,
  `ivf-rabitq-100k-recall-k10.log`,
  `ivf-rabitq-100k-latency-k10-c1.log` — per-step formatted tables.

## Headline result

- Recall@k reproduces the May-19 baseline to four decimal places at every
  (scale × nprobe) cell — bit-exact regression check passes.
- p50 latency is 2–10% **below** the May-19 baseline at every cell across
  25k / 50k / 100k.
- Combined with packet 398's 10k result, IVF/RaBitQ has no detectable
  regression on the local lane after the full task-50 unsafe-block work.

## Re-run command

```
target/debug/ecaz bench suite run \
  --config reviews/task-50/399-ivf-rabitq-scale-rerun/artifacts/ivf-rabitq-scale-suite.json \
  --host localhost --port 28818 --database tqvector_bench \
  --manifest-output reviews/task-50/399-ivf-rabitq-scale-rerun/artifacts/suite-manifest.json \
  --results-output  reviews/task-50/399-ivf-rabitq-scale-rerun/artifacts/results.jsonl \
  --log-file        reviews/task-50/399-ivf-rabitq-scale-rerun/artifacts/ecaz-bench-suite-run.log
```

Prereq: `cargo pgrx install --release --no-default-features --features pg18
--pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config` (i.e. the
release-mode install verified in packet 398).
