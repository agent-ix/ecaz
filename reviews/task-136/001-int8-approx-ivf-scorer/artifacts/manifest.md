# Task 136 packet 001 — int8_approx IVF scorer A/B: artifact manifest

- Head SHA (code under review + dylib + runner): `9514c7518`
  ("Wire int8_approx factored scorer into IVF no-QJL 4-bit dispatch (Task 136)",
  branch `task-136-rank1-scorer`, base `d13ddbe86`).
- Task bucket / packet: `reviews/task-136/001-int8-approx-ivf-scorer/`
- Host: Apple M5 Pro (m5-local lane), PG18 via socket `/Users/peter/.pgrx`
  port 28818, db `tqvector_bench`. Quiet machine, single run, 2026-07-02.
- Installed dylib verified before the suite:
  `SELECT ecaz_build_git_sha()` → `9514c75180e578155c80d16ebcd4b8660fd191f0`
  (equals head SHA); installed `/opt/homebrew/lib/postgresql@18/ecaz.dylib`
  shasum `028cc48e...` equals `target/release/libecaz.dylib`. No dylib installs
  occurred during the run.
- Lane / fixture: IVF `ec_ivf` profile, storage_format `turboquant` (no-QJL
  4-bit lane, dim 1536, bits 4, seed 42), dbpedia real corpus staged at
  `data/staged-current/ec_real_{10k,50k,100k}_*` (recall baselines reproduce
  task-125/132/133 exactly). nprobe [32], k 10.
- Surfaces: **isolated one-index-per-table** (`task136_tq_ivf_real{scale}`
  prefixes, fresh loads in this run/session).
- A/B axis: session GUC `ec_ivf.turboquant_scorer` = `lut` (baseline, i16-LUT
  block kernel) vs `int8_approx` (factored rank-1 in-register kernel,
  `quant::int8_approx32`). Same session, same tables, one binary. Both cells
  per scale ran back-to-back per the same-session A/B rule.
- Runner: `target/release/ecaz` (same SHA) —

  ```sh
  target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 \
    bench suite run --config reviews/task-136/001-int8-approx-ivf-scorer/task136-int8-approx-ab-suite.json \
    --artifact-dir reviews/task-136/001-int8-approx-ivf-scorer/artifacts
  ```

- Suite config: `task136-int8-approx-ab-suite.json` (packet root). Bespoke
  (not the canonical lane config) because the task needs the
  `ec_ivf.turboquant_scorer` on/off axis per scale with `ivf_stage_counters` +
  `task87_candidate_batch_counters` on the fixed nprobe=32 point — same shape
  as the Task 133 stage-attribution config it was copied from.
- Completed 2026-07-02 ~15:00 UTC, exit code 0, 19/19 steps
  (`suite-manifest.json`, `results.jsonl`, `suite-run.log`).

## Key result lines (cited by request.md)

All values from `results.jsonl` / the per-step logs below.

### Recall@10 (nprobe=32)

| scale | lut | int8_approx | delta | ci95 (lut) |
|---|---|---|---|---|
| 10k (64q) | 0.9734 | 0.9719 | −0.15 pp | [0.9579, 0.9834] |
| 50k (48q) | 0.9521 | 0.9521 | ±0.00 pp | [0.9291, 0.9679] |
| 100k (32q) | 0.8969 | 0.8938 | −0.31 pp | [0.8587, 0.9256] |

Artifacts: `recall-ivf-tq-real{10k,50k,100k}-{lut,int8-approx}.log`.
LUT baselines reproduce the task-125 gates (0.9734 / 0.9521 / 0.8969) exactly.

### Latency (ms, nprobe=32, warm cache, concurrency 1)

| scale | iters | lut mean / p50 | int8_approx mean / p50 | mean delta |
|---|---|---|---|---|
| 10k | 64 | 0.92 / 0.86 | 0.79 / 0.73 | −14.1% |
| 50k | 48 | 1.89 / 1.81 | 1.62 / 1.56 | −14.3% |
| 100k | 32 | 2.79 / 2.70 | 2.33 / 2.22 | −16.5% |

Artifacts: `latency-ivf-tq-real{10k,50k,100k}-{lut,int8-approx}.log`.

### Stage counters (per-sweep elapsed_ms from `[ivf-stage-counters]` lines)

| scale | stage | lut | int8_approx | delta |
|---|---|---|---|---|
| 10k | scorer_batch | 24.447 | 16.339 | −33.2% |
| 10k | approximate_scan | 42.494 | 34.882 | −17.9% |
| 50k | scorer_batch | 39.384 | 26.700 | −32.2% |
| 50k | approximate_scan | 73.362 | 61.057 | −16.8% |
| 100k | scorer_batch | 38.601 | 25.184 | −34.8% |
| 100k | approximate_scan | 74.705 | 60.369 | −19.2% |

posting_visit − scratch_flush (the Task 135 target) is unchanged within noise
(100k: 25.6 ms lut vs 24.7 ms int8) — the win is isolated to the scorer stage,
per-change attribution clean.

### Storage

Query-side change only; on-disk codes identical by construction. One storage
step per scale on the shared tables:
`storage-ivf-tq-real{10k,50k,100k}.log` (100k: indexes 92.6 MiB, total
1.6 GiB, 17616.8 B/row — matches the task-125-family baseline).

## Not committed (regenerable / banned)

- `truth-cache/` (recall ground-truth cache; gitignored per repo policy).

## Addendum 2026-07-03 (feedback response)

- Packet-local validation logs added per reviewer finding 1:
  `focused-tests-quantizer.log` (38 passed, includes the 5 int8_approx
  dispatch/parity tests) and `clippy-pg18.log` (pre-existing finding only),
  regenerated at branch head `e6b08f497` (136 code unchanged since
  `9514c7518`).
