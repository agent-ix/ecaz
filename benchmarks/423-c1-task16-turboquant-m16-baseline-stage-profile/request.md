# Review Request: C1 Task16 TurboQuant M16 Baseline Stage Profile

Current head at execution: `a7c1da1`

## Context

Task 16 requires a baseline measurement packet before any new lever wiring:

- same scratch lane shape as packets `413` / `414`
- one-index-per-table surface
- planner-verified warm SQL cell
- per-stage readout for the turboquant scan path at `50k, m=16, ef=128`

This packet now includes the standalone instrumentation seam needed to make the
baseline code-backed instead of inferred from generic scan counters.

### Current-code reality on `main` as of 2026-04-18

1. **TurboQuant already has the generic ADR-031 binary prefilter.**
   `src/am/scan.rs` now prepares a binary query whenever the quantizer supports
   the no-QJL `4`-bit lane and `tqhnsw.disable_binary_prefilter` is not set.
   The frontier scan path then truncates approximate candidates before exact
   scoring. This is not PqFastScan-only anymore.
2. **The `1536`-dimensional `4`-bit lane is already no-QJL.**
   In `src/quant/prod.rs`, `qjl_enabled(dim, bits)` returns `false` when
   `rotation::tile_dim(dim)` is present; `src/quant/rotation.rs` defines that
   tiled compat case for `1536`.

That means the requested task-16 labels need to be mapped carefully on current
head:

- requested `lut_gather`: currently inactive on this lane
- requested `qjl_accumulate`: currently `0` on this lane
- requested `rerank`: currently `0` on this lane
- requested `binary prefilter` lever: already present in today's turboquant path

### Code landed in this packet

Commit `a7c1da1` adds a dedicated debug surface:

- `tests.tqhnsw_debug_turboquant_scan_stage_profile(index_oid, query)`

It reports the current-head turboquant stages directly:

- traversal residual
- binary prefilter calls / elapsed / survivors
- exact quantized score calls / elapsed
- rerank calls / elapsed
- exact-score mode flags (`mode`, `uses_lut`, `uses_qjl`)

This keeps the measurement packet separate from any lever wiring.

## Environment

### Isolated planner-facing surface

Used a one-index-per-table scratch surface cloned from the canonical 50k corpus:

- corpus table: `tqhnsw_real_50k_turboquant_m16only_corpus`
- query table: `tqhnsw_real_50k_turboquant_m16only_queries`
- index: `tqhnsw_real_50k_turboquant_m16only_m16_idx`

Surface creation:

```sql
CREATE TABLE tqhnsw_real_50k_turboquant_m16only_corpus
AS TABLE tqhnsw_real_50k_corpus;

CREATE TABLE tqhnsw_real_50k_turboquant_m16only_queries
AS TABLE tqhnsw_real_50k_queries;

CREATE INDEX tqhnsw_real_50k_turboquant_m16only_m16_idx
ON tqhnsw_real_50k_turboquant_m16only_corpus
USING tqhnsw (embedding tqvector_ip_ops)
WITH (
  m = 16,
  ef_construction = 128,
  build_source_column = 'source'
);
```

### Warm verified SQL cell

Verified launcher:

```bash
./scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k_turboquant_m16only \
  --m 16 \
  --ef-search 128 \
  --query-limit 50 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output tmp/task16-baseline-turboquant-m16only-stageprofile.summary
```

### Internal scan-profile readout

Used the new dedicated helper over the same 50 queries at `ef_search = 128`:

- `tests.tqhnsw_debug_turboquant_scan_stage_profile(...)`

Because the long-lived scratch database predated the new extension SQL, I
registered the helper from the freshly installed
`/home/peter/.pgrx/17.9/pgrx-install/share/postgresql/extension/tqvector--0.1.0.sql`
definition before measurement. The runtime code itself came from commit
`a7c1da1`; this was only a SQL registration step so the existing 50k lane could
call the new wrapper without reloading the whole corpus.

Artifacts:

- `tmp/task16-baseline-turboquant-m16only-stageprofile.summary`
- `tmp/task16-baseline-turboquant-m16only-stageprofile.txt`

## Results

### Verified SQL latency cell

- mean: `5.046ms`
- p50: `4.987ms`
- p95: `6.369ms`
- p99: `7.952ms`

### Internal hot-path baseline

From `tests.tqhnsw_debug_turboquant_scan_stage_profile(...)` over the same
50-query set:

- internal hot `amrescan` mean: `4.75400ms`
- internal traversal residual mean: `3.04980ms`
- internal binary-prefilter bucket mean: `0.00582ms`
- internal exact-score bucket mean: `1.69838ms`
- binary-prefilter calls per query mean: `1793.88`
- binary-prefilter survivors per query mean: `1604.52`
- exact-score calls per query mean: `1605.52`
- exact-score mode: `mse_no_qjl_4bit`
- exact-score `uses_lut`: `false`
- exact-score `uses_qjl`: `false`

Mapped to the task-16 requested labels on **current** code:

| requested stage | current-head interpretation | mean |
|---|---|---:|
| traversal | `amrescan_total - binary_prefilter - exact_score - rerank` residual | `3.04980ms` |
| LUT gather | inactive on the `mse_no_qjl_4bit` lane | `0.00000ms` |
| QJL accumulate | current `1536x4` lane is no-QJL | `0.00000ms` |
| binary prefilter | generic ADR-031 sidecar stage already in turboquant | `0.00582ms` |
| exact score | surviving turboquant exact-score work | `1.69838ms` |
| rerank | no separate turboquant rerank stage today | `0.00000ms` |

### Binary-prefilter A/B sanity check

Small current-head A/B on the same isolated surface using
`tqhnsw.disable_binary_prefilter`:

| mode | exact-score calls mean | exact-score ms mean |
|---|---:|---:|
| prefilter on | `1605.52` | `1.71546ms` |
| prefilter off | `1767.92` | `1.85050ms` |

That is direct evidence that current turboquant scans are already using the
generic ADR-031 binary prefilter.

## Readout

### 1. The baseline packet is captured, but it invalidates part of the written task scope

Two task-16 assumptions are already false on current head:

- lever 1 is already present on turboquant
- the serious `1536x4` lane already has no QJL stage

So the next implementation slice should **not** try to "port" either of those
behaviors as if they were missing.

### 2. Current turboquant still spends real time in exact candidate scoring after the prefilter

Even with the generic binary prefilter already on:

- exact-score bucket mean is still `1.69838ms`
- exact-score call volume is still ~`1605` candidates/query at this cell
- binary prefilter itself is tiny (`0.00582ms`) relative to the surviving exact
  score and traversal residual

That means current turboquant's remaining cost is not "QJL accumulation" on
this lane; it is the surviving exact-score work plus the rest of traversal.

### 3. The next real missing work is the turboquant-specific follow-on, not more ADR-031 plumbing

Based on current code + this baseline, the likely remaining turboquant work is:

- explicit turboquant rerank behavior, if still justified
- payload-layout / hot-cold work, if the surviving exact-score bucket remains
  dominant after any rerank change

But the branch should treat those as **current-head gaps**, not blindly replay
the original task text.
