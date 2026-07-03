# Task 141 packet 001 — int8_approx SDOT kernel A/B: artifact manifest

- Code under review: `2d98ec5b7` ("Add SDOT (dotprod) fast path to the
  int8_approx32 NEON kernel", branch `task-141-sdot-kernel` off main
  `e5ef96109`).
- Task bucket / packet: `reviews/task-141/001-sdot-kernel/`
- Host: Apple M5 Pro (m5-local), PG18 socket `/Users/peter/.pgrx` port 28818,
  db `tqvector_bench`. 2026-07-02.
- A/B form: **before/after commit**, same session, same tables, back-to-back
  runs with one dylib swap between them (no suite was running during the
  install; both cells verified in-suite):
  - baseline run: `artifacts/baseline/`, `precheck-build-sha.log` records
    `ecaz_build_git_sha()` = `e5ef96109...` (main, pre-SDOT), release dylib
    15,550,736 B installed 17:16 local.
  - sdot run: `artifacts/sdot/`, `precheck-build-sha.log` records
    `2d98ec5b7...`, release dylib 15,550,736 B installed 17:21 local.
- Fixture: the Task 135 packet 002 row-layout tables
  (`task135ab_row_real{10k,50k,100k}` prefixes, plain turboquant, dbpedia
  1536-dim, loaded 2026-07-02 morning at `8f7bce3cc`; loader/build code
  untouched since). Both cells read the same tables — table-rebuild drift
  excluded by construction. nprobe [32], k 10, warm cache, concurrency 1.
- Scorer under test: `ec_ivf.turboquant_scorer=int8_approx` (session GUC) in
  BOTH cells; the only delta is the kernel implementation (legacy
  `vmull`/`vpadal` vs `sdot` inline-asm path, runtime-dispatched on
  `dotprod`).
- Runner: `target/release/ecaz` at each cell's respective sha —

  ```sh
  target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 \
    bench suite run --config reviews/task-141/001-sdot-kernel/task141-sdot-ab-suite.json \
    --artifact-dir reviews/task-141/001-sdot-kernel/artifacts/<baseline|sdot>
  ```

- Bespoke config reason: no-load, int8-only A/B on existing tables (kernel
  change; LUT cells and fresh loads are not part of this axis).
- Both runs exit 0, 10/10 steps each (`suite-manifest.json`,
  `results.jsonl`, `suite-run.log` per run dir).

## Key result lines (cited by request.md)

### Recall@10 (nprobe=32) — byte-identical, as required by bit-exactness

| scale | pre-SDOT | post-SDOT |
|---|---|---|
| 10k (64q) | 0.9719 | 0.9719 |
| 50k (48q) | 0.9521 | 0.9521 |
| 100k (32q) | 0.8938 | 0.8938 |

### Latency (ms, warm, concurrency 1, int8_approx scorer both cells)

| scale | pre mean / p50 | post mean / p50 | mean delta |
|---|---|---|---|
| 10k | 0.76 / 0.73 | 0.67 / 0.62 | −11.8% |
| 50k | 1.55 / 1.50 | 1.29 / 1.23 | −16.8% |
| 100k | 2.34 / 2.24 | 1.95 / 1.84 | −16.7% |

Baseline reproduces the Task 136 packet int8 cells (0.79/1.62/2.33 there,
0.76/1.55/2.34 here on different table instances — within the documented
cross-rebuild drift).

### scorer_batch stage (per-sweep elapsed_ms)

| scale | pre-SDOT | post-SDOT | delta |
|---|---|---|---|
| 10k | 15.883 | 8.958 | −43.6% |
| 50k | 26.555 | 14.156 | −46.7% |
| 100k | 25.774 | 13.522 | **−47.5%** |

approximate_scan at 100k: 60.998 → 48.183 (−21.0%); posting_visit contains
the scorer via scratch_flush, hence its drop.

### Storage

Unchanged by construction (query-side kernel change; same tables measured in
both runs): `storage-real{10k,50k,100k}.log` per run dir.

## Not committed (regenerable / banned)

- `baseline/truth-cache/`, `sdot/truth-cache/` (gitignored).

## Addendum 2026-07-03 (feedback response)

- HNSW A/B added per reviewer finding 1: fresh HNSW loads
  (`task141_hnsw_real{10k,50k,100k}`, m=16, ef_construction=128, loads at
  branch `e6b08f497`; `load-hnsw-real*.log` + `hnsw-loads-run.log`), then
  same-tables A/B at ef sweep point 64 with
  `ec_hnsw.turboquant_exact_score_mode=int8_approx` in both cells:
  `hnsw-baseline/` (dylib `e5ef96109`, pre-SDOT) vs `hnsw-sdot/` (dylib
  `e6b08f497`). Key lines: recall@10 0.9203/0.9333/0.8750 IDENTICAL in both
  cells; latency mean 0.63→0.67 / 0.75→0.81 / 1.01→1.02 ms (neutral within
  noise, no win) — claim narrowed to IVF-only in request.md. Bespoke fixed
  ef=64 point (not the full default sweep) because this is a kernel-parity
  A/B, not an HNSW operating-point study.
- Packet-local validation logs added per finding 2:
  `focused-tests-int8approx32.log`, `clippy-pg18.log` (regenerated at
  `e6b08f497`).
