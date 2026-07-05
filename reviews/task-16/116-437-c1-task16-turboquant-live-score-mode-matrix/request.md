# Review Request: C1 Task16 TurboQuant Live Score-Mode Matrix

Current head at execution: `e49e835`

## Context

Packet `433` measured the remaining scorer options offline and concluded:

- full LUT: no win
- tiled LUT: worse
- int8 approx: only remaining scorer option with a real speed signal

Packet `434` then added the first live scan-path seam for `int8_approx`, and
packet `436` extended that same live seam to `full_lut` and `tiled_lut`.

This packet answers the open task-16 question directly on the real TurboQuant
scan path:

- compare `exact`, `full_lut`, `tiled_lut`, and `int8_approx`
- on both:
  - the quantized lane
  - the recall-preserving `heap_f32` lane
- at the task-16 cell:
  - `m = 16`
  - `ef_search = 128`
  - warm verified SQL, `50` queries

No repo code changed in this packet.

## Environment

### Scratch setup

Installed current head into the normal user scratch and started the TurboQuant
matrix lane with args-only helpers:

```bash
./scripts/install_adr030_pg17_pg_test.sh --pgrx-home /home/peter/.pgrx
./scripts/restart_adr030_scratch.sh --pgrx-home /home/peter/.pgrx --rerank-mode quantized
```

### Isolated matrix surface

Loaded one isolated TurboQuant real-corpus surface:

```bash
./scripts/load_real_corpus_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  --database postgres \
  --prefix tqhnsw_real_50k_task16_lutcmp \
  --corpus-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv \
  --queries-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv \
  --m 16 \
  --storage-format turboquant \
  --allow-manifest-mismatch
```

This produced:

- corpus table: `tqhnsw_real_50k_task16_lutcmp_corpus`
- queries table: `tqhnsw_real_50k_task16_lutcmp_queries`
- index: `tqhnsw_real_50k_task16_lutcmp_turboquant_m16_idx`

All eight cells below ran against that same rebuilt index.

## Commands

### Verified SQL latency

Each cell used:

```bash
./scripts/bench_sql_latency_verified_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  --prefix tqhnsw_real_50k_task16_lutcmp \
  --storage-format turboquant \
  --m 16 \
  --ef-search 128 \
  --query-limit 50 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output <cell summary path>
```

### Recall summary

Each cell used:

```bash
./scripts/run_real_corpus_recall_scratch.sh \
  --socket-dir /home/peter/.pgrx \
  summary \
  --prefix tqhnsw_real_50k_task16_lutcmp \
  --storage-format turboquant \
  --m 16 \
  --ef-search 128 \
  --queries-table tqhnsw_real_50k_task16_lutcmp_queries \
  --corpus-table tqhnsw_real_50k_task16_lutcmp_corpus
```

### Mode selection

Mode changes were done only through the standardized restart helper:

- baseline exact:

```bash
./scripts/restart_adr030_scratch.sh --pgrx-home /home/peter/.pgrx --rerank-mode <quantized|heap_f32>
```

- `full_lut`:

```bash
./scripts/restart_adr030_scratch.sh \
  --pgrx-home /home/peter/.pgrx \
  --rerank-mode <quantized|heap_f32> \
  -e TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE=full_lut
```

- `tiled_lut`:

```bash
./scripts/restart_adr030_scratch.sh \
  --pgrx-home /home/peter/.pgrx \
  --rerank-mode <quantized|heap_f32> \
  -e TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE=tiled_lut
```

- `int8_approx`:

```bash
./scripts/restart_adr030_scratch.sh \
  --pgrx-home /home/peter/.pgrx \
  --rerank-mode <quantized|heap_f32> \
  -e TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE=int8_approx
```

## Artifacts

Latency summaries:

- `tmp/task16-turboquant-quantized-exact-m16.summary`
- `tmp/task16-turboquant-quantized-full-lut-m16.summary`
- `tmp/task16-turboquant-quantized-tiled-lut-m16.summary`
- `tmp/task16-turboquant-quantized-int8-m16.summary`
- `tmp/task16-turboquant-heapf32-exact-m16.summary`
- `tmp/task16-turboquant-heapf32-full-lut-m16.summary`
- `tmp/task16-turboquant-heapf32-tiled-lut-m16.summary`
- `tmp/task16-turboquant-heapf32-int8-m16.summary`

Recall summaries:

- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T215526Z_summary_tqhnsw_real_50k_task16_lutcmp_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_task16_lutcmp_queries.tsv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T215718Z_summary_tqhnsw_real_50k_task16_lutcmp_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_task16_lutcmp_queries.tsv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T215906Z_summary_tqhnsw_real_50k_task16_lutcmp_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_task16_lutcmp_queries.tsv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T220057Z_summary_tqhnsw_real_50k_task16_lutcmp_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_task16_lutcmp_queries.tsv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T220250Z_summary_tqhnsw_real_50k_task16_lutcmp_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_task16_lutcmp_queries.tsv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T220446Z_summary_tqhnsw_real_50k_task16_lutcmp_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_task16_lutcmp_queries.tsv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T220645Z_summary_tqhnsw_real_50k_task16_lutcmp_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_task16_lutcmp_queries.tsv`
- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260418T220837Z_summary_tqhnsw_real_50k_task16_lutcmp_turboquant_m16_idx_m16_ef128_tqhnsw_real_50k_task16_lutcmp_queries.tsv`

## Results

### Quantized lane

| mode | mean latency | delta vs exact | recall@10 | exact-gap queries |
| --- | ---: | ---: | ---: | ---: |
| exact | `2.786ms` | baseline | `0.9251` | `14` |
| full_lut | `2.333ms` | `-0.453ms` / `-16.26%` | `0.9251` | `14` |
| tiled_lut | `2.325ms` | `-0.461ms` / `-16.55%` | `0.9251` | `14` |
| int8_approx | `2.266ms` | `-0.520ms` / `-18.66%` | `0.9245` | `14` |

Additional recall fields:

- exact / full_lut / tiled_lut all matched on:
  - `mean_abs_score_error = 0.006030937`
  - `graph_below_exact_queries = 14`
  - `worst_exact_gap = 1`
- `int8_approx` shifted slightly to:
  - `mean_abs_score_error = 0.005790341`
  - `graph_recall_at_10 = 0.9245`
  - `graph_below_exact_queries = 14`
  - `worst_exact_gap = 1`

### Heap-f32 serious lane

| mode | mean latency | delta vs exact | recall@10 | exact-gap queries |
| --- | ---: | ---: | ---: | ---: |
| exact | `4.610ms` | baseline | `0.9629` | `0` |
| full_lut | `4.542ms` | `-0.068ms` / `-1.48%` | `0.9629` | `0` |
| tiled_lut | `4.530ms` | `-0.080ms` / `-1.74%` | `0.9629` | `0` |
| int8_approx | `4.747ms` | `+0.137ms` / `+2.97%` | `0.9629` | `0` |

All four heap-f32 runs matched on:

- `mean_abs_score_error = 0`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

## Readout

### 1. Live runtime changes the option ranking from packet `433`

The offline scorer study in packet `433` said:

- full LUT was a wash
- tiled LUT was worse

That conclusion does **not** survive the real scan-path measurement on the
quantized lane. On the actual TurboQuant runtime:

- `full_lut` is materially faster than exact (`-16.26%`)
- `tiled_lut` is materially faster than exact (`-16.55%`)
- both preserve the current quantized-lane recall exactly

So lever 4 is real on the live scan path even though it was not compelling in
the isolated scorer microbench.

### 2. Lever 4 beats lever 5 on the quantized lane if recall stability matters

On the quantized lane:

- `int8_approx` is still the single fastest mode (`2.266ms`)
- but its win over lever 4 is small:
  - vs `full_lut`: `0.067ms`
  - vs `tiled_lut`: `0.059ms`
- and it is the only mode that moved recall at all (`0.9251 -> 0.9245`)

So if the choice is "fastest while preserving the current quantized-lane
operating point," lever 4 now looks stronger than lever 5 on current head.

### 3. None of the scorer levers closes the serious lane

The task-16 decision criterion remains the recall-preserving `heap_f32` lane.
On that lane:

- `full_lut`: only `-1.48%`
- `tiled_lut`: only `-1.74%`
- `int8_approx`: regression (`+2.97%`)

That means:

- lever 4 is not enough to close task 16 on the serious lane
- lever 5 is actively worse there
- the remaining serious-lane bottleneck is still not the TurboQuant scorer
  itself

This is consistent with packets `429` / `430` / `432` pointing at rerank
fetch/decode cost rather than scorer cost as the load-bearing issue.

### 4. Tiled LUT is not justified over full LUT from this cell alone

The live deltas between full and tiled are tiny:

- quantized: `2.333ms` vs `2.325ms`
- heap-f32: `4.542ms` vs `4.530ms`

That is not enough evidence to justify tile complexity by itself. On the live
runtime cell measured here, `full_lut` gives the lever-4 win already.

### 5. The requested V3 vacuum-concurrency rerun is not green

While clearing the outstanding feedback, I also reran:

```bash
scripts/vacuum_concurrency_scratch.sh --socket-dir /home/peter/.pgrx --duration 60
```

on current head after a neutral quantized restart.

That run failed because both scan workers hit:

```text
unexpected tqhnsw scan result count: 0
```

So the V3 concurrency question is now answered with data:

- the rerun did happen
- it did **not** confirm safety
- this is now a separate follow-on issue from the scorer decision in task 16
