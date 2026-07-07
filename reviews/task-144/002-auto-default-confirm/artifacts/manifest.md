# Manifest — Task 144 auto-default confirmation (packet 002)

- **Head SHA:** `3f69d74c08d24f54f99b71d7e9be2451b5da8808` (main; the Task 144
  default-flip commit). In-suite precheck confirms
  `ecaz_build_git_sha()` = `3f69d74c0…` and
  `current_setting('ec_hnsw.turboquant_exact_score_mode')` = `auto`
  (`artifacts/precheck-build-sha.log`).
- **Task bucket / packet:** `reviews/task-144/002-auto-default-confirm/`
- **Lane / host:** m5-local (Apple M5), PG18, pgrx socket `/Users/peter/.pgrx`
  port 28818, db `tqvector_bench`. Runner + installed dylib both at
  `3f69d74c0` (release build, 15,550,640 B). SHA re-verified unchanged after
  the suite; no dylib swap during the run.
- **Access method / quant:** `ec_hnsw`, TurboQuant storage, 4-bit, no-QJL
  4-bit lane (1536-dim → `tile_dim(1536)=Some(512)` → int8_approx supported).
- **Fixtures:** `task141_hnsw_real{10k,50k,100k}` (m16 / ef_construction=128),
  reused from Task 141; isolated one-index-per-table surfaces (no shared
  table). Fallback smoke fixture `tq144fallback` (dim 64, 2000 rows, QJL-active
  4-bit lane) generated fresh via `ecaz corpus generate` (regenerable; TSV not
  committed).
- **Rerank / mode:** NO session GUCs on the default-path cells — the `auto`
  default is exercised as installed.

## Why this packet exists (non-standard grid)

This is a **narrow default-path confirmation**, not the standard lane sweep:
Task 144 landed a default-GUC flip (`ec_hnsw.turboquant_exact_score_mode`
`exact` → `auto`) as a query-time-only change. The A/B evidence (recall +
latency + storage across the HNSW 10k/50k/100k × ef 40–200 matrix) already
lives in packet 001 (`reviews/task-144/001-hnsw-scorer-default/`). This packet
proves the **installed default path** now takes the int8 lane and that the
`auto` fallback is safe on lanes the factored kernel does not serve. It
deliberately reuses packet 001's HNSW grid (same prefixes, same ef sweep,
same k/seed/queries_limit) with the `session_gucs` dropped, so its cells are
directly comparable to 001's explicit `exact` / `int8_approx` cells.

## Commands

Suite (default-path recall + latency + storage, no GUCs):

    ecaz bench suite run \
      --config reviews/task-144/002-auto-default-confirm/task144-auto-default-confirm-suite.json \
      --artifact-dir reviews/task-144/002-auto-default-confirm/artifacts \
      --database tqvector_bench --host /Users/peter/.pgrx --port 28818

Fallback-lane fixture + smoke (QJL-active dim-64, auto default must NOT error):

    ecaz corpus generate --output <sp>/tq144fallback_corpus.tsv  --n 2000 --dim 64 --seed 42 --kind corpus
    ecaz corpus generate --output <sp>/tq144fallback_queries.tsv --n 64   --dim 64 --seed 7  --kind queries
    ecaz corpus load --prefix tq144fallback --dim 64 --profile ec_hnsw --bits 4 \
      --corpus-file <sp>/tq144fallback_corpus.tsv --queries-file <sp>/tq144fallback_queries.tsv \
      --log-file .../fallback-load.log
    ecaz bench recall --prefix tq144fallback --profile ec_hnsw --k 10 --sweep 64,100 \
      --queries-limit 64 --bits 4 --seed 42 --force-index --log-output .../fallback-recall-auto.log

Counterfactual (explicit int8 on the same QJL lane must error):

    ecaz bench recall --prefix tq144fallback --profile ec_hnsw --k 10 --sweep 64 \
      --queries-limit 8 --bits 4 --seed 42 --force-index \
      --session-guc ec_hnsw.turboquant_exact_score_mode=int8_approx \
      --log-output .../fallback-explicit-int8-errors.log

- **Timestamp:** captured_at `2026-07-03 13:28:21 -07` (precheck row).
- **Isolation:** isolated one-index-per-table surfaces.

## Key result lines cited by request.md

### 1. Default-path cell — auto == int8 byte-for-byte (recall@k)

Auto default (this packet) vs packet 001 explicit `int8_approx` and `exact`:

| prefix  | ef  | exact (001) | int8 (001) | **auto (002)** |
|---------|-----|-------------|------------|----------------|
| 10k     | 40  | 0.8734      | 0.8719     | **0.8719**     |
| 10k     | 64  | 0.9219      | 0.9203     | **0.9203**     |
| 10k     | 100 | 0.9516      | 0.9500     | **0.9500**     |
| 50k     | 40  | 0.9062      | 0.9042     | **0.9042**     |
| 50k     | 64  | 0.9375      | 0.9333     | **0.9333**     |
| 50k     | 100 | 0.9437      | 0.9396     | **0.9396**     |
| 100k    | 40  | 0.7906      | 0.7875     | **0.7875**     |
| 100k    | 64  | 0.8781      | 0.8750     | **0.8750**     |
| 100k    | 100 | 0.9094      | 0.9062     | **0.9062**     |

Auto matches the explicit-int8 column to 4 decimal places at **all 18** ef
points (40/64/100/128/160/200 × 3 scales) and differs from `exact` at every
point by the same ≤0.42 pp. Full grid in `recall-auto-real{10k,50k,100k}.log`.

Latency mean (ms), auto default:

| prefix | ef  | mean | p50  | p95  | p99  |
|--------|-----|------|------|------|------|
| 10k    | 64  | 0.65 | 0.60 | 0.82 | 1.59 |
| 10k    | 100 | 0.91 | 0.85 | 1.16 | 1.92 |
| 50k    | 64  | 0.83 | 0.75 | 1.00 | 2.46 |
| 50k    | 100 | 1.09 | 1.03 | 1.37 | 2.63 |
| 100k   | 64  | 1.05 | 0.95 | 1.44 | 2.89 |
| 100k   | 100 | 1.33 | 1.23 | 1.80 | 2.68 |

(Consistent with 001's int8 band; ef64 mid-points are the win vs exact.)
Files: `latency-auto-real{10k,50k,100k}.log`.

Storage (query-side-invariant, unchanged vs exact): index per-row
1366.4 / 1365.6 / 1365.4 B at 10k / 50k / 100k
(`storage-real{10k,50k,100k}.log`).

### 2. Resolved-mode proof

`current_setting('ec_hnsw.turboquant_exact_score_mode')` reports **`auto`**,
NOT `int8_approx` — this is the intended design, not a miss. A literal
`int8_approx` default would `pgrx::error!("… requires the no-QJL 4-bit lane")`
on every QJL-active / non-4-bit TQ HNSW scan; `auto` resolves **per scan**
(`resolve_turboquant_exact_score_mode`, `src/am/ec_hnsw/scan.rs:1395`):
int8_approx when `quantizer.int8_approx_no_qjl_4bit_supported()`, exact
otherwise. The resolved mode is proven two ways:

- **Behavioral (this packet):** auto recall == explicit-int8 recall byte-for-byte
  at all 18 cells (table above) — the installed default is running the int8
  kernel on the 1536-dim tables.
- **Surface (pg_test):** `src/tests/ec_hnsw_runtime_profiles.rs`
  `test_turboquant_scan_stage_profile_auto_default_resolves_int8` (added with
  this packet) asserts the stage-profile surface reports
  `turboquant_exact_score_mode = int8_approx_no_qjl_4bit` on the default
  (no-override) lane; the sibling `…_int8_mode` test pins the same string
  under an explicit GUC. (pg_test run deferred to Linux — macOS pgrx-test is
  blocked by the known `dyld _BufferBlocks` issue; compile-gated here.)

### 3. Fallback-lane smoke (auto is safe where int8 is unsupported)

`tq144fallback` (dim 64, QJL-active 4-bit → `int8_approx_no_qjl_4bit_supported()`
= false):

- Auto default (no GUC): recall **exit 0**, recall@k 0.7781 (ef64) / 0.7891
  (ef100) — scan succeeds, resolving to `exact`
  (`fallback-recall-auto.log`).
- Explicit `int8_approx` GUC on the **same** lane:
  `ERROR: ec_hnsw TurboQuant exact score mode int8_approx requires the no-QJL
  4-bit lane` (`fallback-explicit-int8-errors.log`). This is the counterfactual
  a naive literal-int8 default would have hit on every such scan; `auto`
  avoids it.
