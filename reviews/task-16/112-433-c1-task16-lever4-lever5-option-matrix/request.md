# Review Request: C1 Task16 Lever4 / Lever5 Option Matrix

Current head at execution: `7a0df32`

## Context

The remaining open item in task 16 is the decision on:

- lever 4: explicit LUT / tiled LUT scoring
- lever 5: int8 LUT-style scoring

Packets `423`, `426`, `429`, `430`, and `432` already established that levers
1–3 did not make the recall-preserving TurboQuant lane clearly competitive
enough. The user asked for data comparing all remaining scorer options directly,
not an inference-only closeout.

This slice does two things:

1. extends the offline real-corpus scorer study so the current no-QJL
   `1536 @ 4-bit` TurboQuant lane can be measured against explicit full-LUT and
   tiled-LUT scorers in addition to the existing int8 and binary-sign studies
2. records one option matrix covering the current no-LUT exact scorer,
   full LUT, tiled LUT, int8 approx, and binary-sign

## Code Changes

### 1. Exposed explicit no-QJL 4-bit LUT scorer study paths

Added:

- `PreparedLutNoQjl4BitQuery`
- `PreparedTiledLutNoQjl4BitQuery`
- `prepare_ip_query_lut_no_qjl_4bit(...)`
- `prepare_ip_query_tiled_lut_no_qjl_4bit(...)`
- `score_ip_from_parts_lut_no_qjl_4bit(...)`
- `score_ip_from_parts_tiled_lut_no_qjl_4bit(...)`

These are intentionally narrow helpers for the current serious lane:

- `1536` dims
- `4-bit`
- no QJL

### 2. Extended `approx_score_study`

New study modes:

- `full-lut`
- `tiled-lut`

The study also now accepts:

- `--tile-size <n>` for tiled-LUT runs

### 3. Real-corpus TSV loader fix

The real corpus TSV rows are emitted as:

```text
id<TAB>[comma,separated,floats]
```

The study loader now strips the surrounding `[` / `]` so the saved real-corpus
TSVs can be used directly.

### 4. Exactness tests

Added quantizer tests proving that, on the no-QJL 4-bit lane:

- explicit full LUT matches the current direct scorer exactly
- tiled LUT matches the current direct scorer exactly
- LUT prep rejects QJL-active lanes

## Validation

Ran on this exact tree before the code checkpoint commit:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

All passed.

## Measurement Setup

### Dataset

Used the staged 50k real corpus TSVs:

- `/home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv`
- `/home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv`

### Study configuration

All runs used:

- `query_count = 50`
- `bench_iters = 4`
- the existing no-QJL `1536 @ 4-bit` TurboQuant quantizer lane

Important baseline note:

- the current task-16 serious lane already uses the direct no-LUT exact scorer
- the `exact_ns_per_score` line in each study run is therefore the current
  runtime reference point for that same query/code set

## Commands

```bash
cargo run --release --bin approx_score_study -- \
  --study-mode full-lut \
  --query-count 50 \
  --bench-iters 4 \
  --corpus-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv \
  --queries-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv \
  > tmp/task16-scorestudy-full-lut.txt

cargo run --release --bin approx_score_study -- \
  --study-mode tiled-lut \
  --tile-size 512 \
  --query-count 50 \
  --bench-iters 4 \
  --corpus-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv \
  --queries-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv \
  > tmp/task16-scorestudy-tiled-lut.txt

cargo run --release --bin approx_score_study -- \
  --study-mode int8-approx \
  --query-count 50 \
  --bench-iters 4 \
  --corpus-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv \
  --queries-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv \
  > tmp/task16-scorestudy-int8-approx.txt

cargo run --release --bin approx_score_study -- \
  --study-mode binary-sign \
  --query-count 50 \
  --bench-iters 4 \
  --corpus-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv \
  --queries-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv \
  > tmp/task16-scorestudy-binary-sign.txt
```

## Artifacts

- `tmp/task16-scorestudy-full-lut.txt`
- `tmp/task16-scorestudy-tiled-lut.txt`
- `tmp/task16-scorestudy-int8-approx.txt`
- `tmp/task16-scorestudy-binary-sign.txt`

## Results

### Option Matrix

| option | fidelity vs current no-LUT scorer | measured scorer cost | relative speed |
| --- | --- | ---: | ---: |
| current no-LUT direct scorer | exact reference | `~1300-1320 ns/score` | `1.00x` |
| full LUT | exact match (`rho=1.0000`, `overlap=1.0000`) | `1303.9 ns/score` | `1.00x` |
| tiled LUT (`tile_size=512`) | exact match (`rho=1.0000`, `overlap=1.0000`) | `1480.1 ns/score` | `0.89x` |
| int8 approx | near-exact (`rho=1.0000`, `overlap=0.9980`) | `831.0 ns/score` | `1.57x` |
| binary sign, cached | prefilter only (`rho=0.8819`, `overlap=0.7160`) | `25.7 ns/score` | `51.44x` |
| binary sign, derived | prefilter only | `5598.0 ns/score` | `0.24x` |

### Full LUT

- `spearman_rho mean=1.0000 min=1.0000`
- `pearson_r mean=1.0000 min=1.0000`
- `top10_overlap mean=1.0000`
- `exact_top10_captured_by_approx_top20 mean=1.0000`
- `microbench exact_ns_per_score=1300.3`
- `microbench approx_ns_per_score=1303.9`
- `speedup=1.00x`

### Tiled LUT

- `tile_size=512`
- `spearman_rho mean=1.0000 min=1.0000`
- `pearson_r mean=1.0000 min=1.0000`
- `top10_overlap mean=1.0000`
- `exact_top10_captured_by_approx_top20 mean=1.0000`
- `microbench exact_ns_per_score=1318.9`
- `microbench approx_ns_per_score=1480.1`
- `speedup=0.89x`

### Int8 Approx

- `spearman_rho mean=1.0000 min=1.0000`
- `pearson_r mean=1.0000 min=1.0000`
- `top10_overlap mean=0.9980`
- `exact_top10_captured_by_approx_top20 mean=1.0000`
- `exact_top10_captured_by_approx_top50 mean=1.0000`
- `microbench exact_ns_per_score=1305.9`
- `microbench approx_ns_per_score=831.0`
- `speedup=1.57x`

### Binary Sign

- `spearman_rho mean=0.8819 min=0.8173`
- `pearson_r mean=0.8993 min=0.8426`
- `top10_overlap mean=0.7160`
- `exact_top10_captured_by_approx_top20 mean=0.9080`
- `exact_top10_captured_by_approx_top50 mean=0.9880`
- `microbench exact_ns_per_score=1319.8`
- `binary_cached_ns_per_score=25.7`
- `binary_derived_ns_per_score=5598.0`
- `cached_speedup=51.44x`
- `derived_speedup=0.24x`

## Readout

### 1. Lever 4 is not justified on the current serious lane

The current serious lane already uses the no-LUT exact scorer. Measuring lever 4
explicitly shows:

- full LUT is effectively a wash
- tiled LUT is strictly worse on this lane

So neither lever-4 variant earns runtime-path work for task 16 as measured here.

### 2. Lever 5 is the only remaining scorer option with a real speed signal

Int8 approx kept near-perfect ordering on the 50k real-corpus scorer study:

- `rho = 1.0000`
- `pearson_r = 1.0000`
- `top10_overlap = 0.9980`
- `exact top10 captured by approx top20 = 1.0000`

while cutting scorer cost from about `1306 ns/score` to `831 ns/score`
(`1.57x` faster).

That is the only remaining scorer option in this matrix that plausibly moves the
task-16 serious lane.

### 3. Binary sign remains a prefilter, not a replacement scorer

The cached binary path is extremely cheap, but the ranking loss is far too large
to treat it as a drop-in serious-lane scorer. Packet `423` already showed its
value as a traversal-side prefilter; this matrix does not change that
interpretation.

### 4. Task-16 closeout state after this packet

The open plan item is now no longer "which scorer options should be measured?"
but "is lever 5 worth wiring into a real TurboQuant scan-path experiment?"

This packet supports:

- do not pursue lever 4 on the current task-16 lane
- pursue one narrow lever-5 scan-path experiment if task 16 is to continue
