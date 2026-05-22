# Manifest: Release-Rebuild PG18 RaBitQ / IVF / SPIRE Rerun

- Task bucket: `reviews/task-50`
- Packet: `reviews/task-50/398-release-rebuild-rabitq-ivf-spire-rerun`
- Branch: `task-50-unsafe-closeout`
- Source HEAD: `e81dcf8fd16cc02ddf4e88b7861af94c5f80ff48` (same as packet 397)
- Build: `cargo pgrx install --release --no-default-features --features
  pg18 --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- Installed `.so`: 17,202,072 B, mtime `2026-05-21 22:41:10`, byte-identical
  to `target/release/libecaz.so` (release build verified)
- PostgreSQL: 18.3 / pgrx local install, `localhost:28818`, db `tqvector_bench`
- Host: `DESKTOP-BMB4AFO` (WSL2, i9-10900K, no AVX-512) — same host as
  baseline `benchmarks/task-50-local-baseline/` and as packet 397
- Surfaces exercised: isolated one-index-per-table corpora
  `ec_real_10k_ivfrabitq` (access method `ec_ivf`) and
  `ec_real_10k_spirerabitq` (access method `ec_spire`); SPIRE deliberately
  capped at 10k per project rule (≤25k known-safe).

## Files

- `rabitq-ivf-spire-local-suite.json` — suite config (input).
- `ecaz-bench-suite-run.log` — driver log for the `ecaz bench suite run`.
- `suite-manifest.json` — suite manifest emitted by the runner.
- `results.jsonl` — authoritative result rows (4 steps × 2 sweep values).
- `ivf-rabitq-10k-recall-k10.log`,
  `ivf-rabitq-10k-latency-k10-c1.log`,
  `spire-rabitq-10k-recall-k10.log`,
  `spire-rabitq-10k-latency-k10-c1.log` — per-step formatted tables.

## Headline numbers

| Step                              | nprobe | recall@k | mean q-time | p50      |
| --------------------------------- | ------ | -------- | ----------- | -------- |
| `ivf-rabitq-10k-recall-k10`       | 8      | 0.9720   | 5.16 ms     | —        |
| `ivf-rabitq-10k-recall-k10`       | 16     | 0.9780   | 8.14 ms     | —        |
| `ivf-rabitq-10k-latency-k10-c1`   | 8      | —        | 4.84 ms     | 4.86 ms  |
| `ivf-rabitq-10k-latency-k10-c1`   | 16     | —        | 7.96 ms     | 8.08 ms  |
| `spire-rabitq-10k-recall-k10`     | 8      | 0.9880   | 35.42 ms    | —        |
| `spire-rabitq-10k-recall-k10`     | 16     | 0.9960   | 63.35 ms    | —        |
| `spire-rabitq-10k-latency-k10-c1` | 8      | —        | 34.2 ms     | 33.5 ms  |
| `spire-rabitq-10k-latency-k10-c1` | 16     | —        | 63.7 ms     | 65.2 ms  |

## Comparison reference

- `benchmarks/task-50-local-baseline/` — 2026-05-19 release baseline
  (head `cc06046177`).
- `reviews/task-50/397-current-head-pg18-rabitq-ivf-spire-sweep/` — same
  source as this packet, but run against the accidentally-debug-built `.so`.

## Re-run command

```
cargo pgrx install --release --no-default-features --features pg18 \
  --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config

target/debug/ecaz bench suite run \
  --config reviews/task-50/398-release-rebuild-rabitq-ivf-spire-rerun/artifacts/rabitq-ivf-spire-local-suite.json \
  --host localhost --port 28818 --database tqvector_bench \
  --manifest-output reviews/task-50/398-release-rebuild-rabitq-ivf-spire-rerun/artifacts/suite-manifest.json \
  --results-output  reviews/task-50/398-release-rebuild-rabitq-ivf-spire-rerun/artifacts/results.jsonl \
  --log-file        reviews/task-50/398-release-rebuild-rabitq-ivf-spire-rerun/artifacts/ecaz-bench-suite-run.log
```
