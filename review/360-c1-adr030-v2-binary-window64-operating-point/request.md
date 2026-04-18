# Review Request: C1 ADR-030 V2 Binary Window-64 Operating Point

## Context

Packet `359` established the first credible grouped-v2 runtime lane:

- grouped-v2 traversal scored by the persisted binary sign sidecar instead of
  grouped-PQ
- verified `50k @ ef=128, window=16` improved from `0.674` to `0.820`
  `Recall@10`
- verified `10k @ ef=128, window=16` improved from `0.815` to `0.926`

That answered the biggest open question from packets `354` through `358`:

> the grouped-built graph was not dead; the grouped-PQ traversal score was
> the main recall bottleneck

But packet `359` still left two important gaps:

1. the live grouped rerank cap was still hard-limited to `16`, even though the
   emitted-set diagnostics were still improving at simulated windows `32` and
   `64`
2. the verified SQL launcher still assumed canonical `<prefix>_{corpus,queries}`
   naming, so it could not target an isolated grouped-only planner surface

The branch needed one more combined batch to answer:

> if binary traversal is the right candidate signal, does a larger live rerank
> window produce a real operating point on canonical `50k`, and can the
> verified SQL tooling measure grouped lanes explicitly when the planner surface
> is isolated?

## Problem

The packet `359` frontier was promising but still not good enough:

- canonical verified `50k @ ef=128, window=16` was `0.820 Recall@10`
- scalar on the same lane was `0.890`

There were also two sources of measurement ambiguity:

1. the canonical `50k` grouped index on scratch had not been rebuilt since the
   corrected binary-lane work started, so the branch still risked comparing a
   fresh scalar baseline to an older grouped build
2. the shared canonical `tqhnsw_real_50k_corpus` table has both scalar and
   grouped indexes, and the planner still preferred the scalar one even when
   the verified launcher was pointed at grouped naming

Without clearing both seams, the branch still could not answer whether the
binary lane had a real same-recall latency win or was only “less bad than
grouped-PQ.”

## Planned Slice

Batch the related runtime and measurement work together:

1. raise the grouped live rerank runtime cap from `16` to `64`
2. prove the higher-window env works in pg coverage
3. extend the verified SQL launchers so real-corpus runs can explicitly target:
   - a non-canonical corpus table
   - a non-canonical query table
   - a specific index name
4. add regression coverage for the new verified-launcher override surface
5. reinstall the current pg17 scratch build and remeasure the binary lane on
   canonical `50k`
6. if the shared canonical planner still chooses scalar, create an isolated
   scratch-only grouped table to validate the new verified-launcher override
   path separately

This slice intentionally does not:

- change grouped-v2 storage format
- redesign traversal scoring again
- claim the shared canonical planner lane is solved

## Implementation

Updated:

- `src/am/scan.rs`
- `src/lib.rs`
- `scripts/bench_sql_latency.sh`
- `scripts/bench_sql_latency_verified.sh`
- `scripts/tests/test_bench_sql_latency_verified.py`

Concrete changes:

1. raised `ADR030_GROUPED_V2_MAX_LIVE_RERANK_WINDOW` from `16` to `64`
2. updated the invalid-window pg expectation string accordingly
3. added pg coverage that proves a higher configured window (`32`) still makes
   the live grouped runtime match the windowed simulation
4. taught `scripts/bench_sql_latency.sh` to accept explicit:
   - `--corpus-table`
   - `--query-table`
   - `--index-name`
5. kept `--prefix` as the canonical naming anchor, but let those overrides
   replace `<prefix>_corpus`, `<prefix>_queries`, and `<prefix>_m{N}_idx`
   when the caller needs a different real-corpus measurement surface
6. required `--index-name` runs to use exactly one effective `--m`, since the
   override names one concrete index
7. taught `scripts/bench_sql_latency_verified.sh` to forward the new table
   overrides and to verify against an explicit `--index-name` when present
8. extended the fake-`psql` regression harness so the verified-launcher tests
   support custom corpus/query/index names and strip `\o` meta-commands in the
   per-cell path
9. added a regression proving the verified launcher succeeds on an explicit
   grouped-style override surface

## Validation

Compile, lint, and script validation on the final code:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `python3 -m unittest scripts.tests.test_bench_sql_latency_verified`
- `bash -n scripts/bench_sql_latency.sh scripts/bench_sql_latency_verified.sh scripts/bench_sql_latency_verified_scratch.sh scripts/restart_adr030_scratch.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

The required lib-test commands were run and still hit the same local linker
environment failure on this workstation:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Both fail during the final test-binary link step with unresolved PostgreSQL
symbols such as `CurrentMemoryContext`, `PG_exception_stack`,
`error_context_stack`, and `errstart`.

Scratch runtime validation passed:

1. install current pg17 scratch build:
   - `./scripts/install_adr030_pg17_pg_test.sh`
2. restart scratch in binary grouped mode:
   - `./scripts/restart_adr030_scratch.sh --window 32 --grouped-score-mode binary`
   - `./scripts/restart_adr030_scratch.sh --window 64 --grouped-score-mode binary`
3. refresh scratch helpers after restart:
   - `./scripts/refresh_adr030_scratch_debug_helpers.sh`
4. verify runtime settings:
   - `grouped_scan_window = 32` / `64`
   - `grouped_scan_score_mode = binary`

## Measurements

All main direct measurements below use:

- canonical `tqhnsw_real_50k_corpus`
- query subset `tqhnsw_real_50k_queries_50`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN=1`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_GROUPED_SCORE_MODE=binary`

### Canonical `50k` direct frontier before rebuild

On the previously-built canonical grouped index, widening the live window from
`16` to `32` and `64` already moved recall materially:

- `window=32`
  - `ef=128`: `0.846 Recall@10 @ 1.833ms`
  - `ef=200`: `0.860 Recall@10 @ 2.280ms`
  - `ef=320`: `0.864 Recall@10 @ 3.610ms`
- `window=64`
  - `ef=128`: `0.860 Recall@10 @ 1.695ms`
  - `ef=200`: `0.870 Recall@10 @ 2.600ms`
  - `ef=320`: `0.874 Recall@10 @ 3.902ms`

That was enough to prove the `16` cap was leaving real performance on the
table, but the canonical grouped index had not been freshly rebuilt on the
current scratch install yet.

### Canonical `50k` direct frontier after rebuild

After reinstalling the extension and rebuilding
`tqhnsw_real_50k_grouped_m8_idx` on the canonical `50k` corpus, the `window=64`
binary lane settled at:

| path | ef_search | Recall@10 | exact-quantized Recall@10 | mean query latency ms |
|------|-----------|-----------|---------------------------|-----------------------|
| grouped binary `window=64` | 40 | 0.836 | 0.860 | 0.947 |
| grouped binary `window=64` | 64 | 0.860 | 0.860 | 1.007 |
| grouped binary `window=64` | 100 | 0.864 | 0.860 | 1.367 |
| grouped binary `window=64` | 128 | 0.868 | 0.860 | 1.708 |
| grouped binary `window=64` | 160 | 0.870 | 0.860 | 2.274 |
| grouped binary `window=64` | 200 | 0.870 | 0.860 | 2.436 |
| grouped binary `window=64` | 256 | 0.874 | 0.860 | 3.126 |
| grouped binary `window=64` | 320 | 0.874 | 0.860 | 3.894 |

Same-cluster scalar baseline on the same query subset:

| path | ef_search | Recall@10 | exact-quantized Recall@10 | mean query latency ms |
|------|-----------|-----------|---------------------------|-----------------------|
| scalar | 40 | 0.860 | 0.860 | 1.398 |
| scalar | 64 | 0.876 | 0.860 | 1.861 |
| scalar | 100 | 0.884 | 0.860 | 2.619 |
| scalar | 128 | 0.890 | 0.860 | 3.171 |
| scalar | 160 | 0.892 | 0.860 | 3.691 |
| scalar | 200 | 0.894 | 0.860 | 4.484 |
| scalar | 256 | 0.896 | 0.860 | 5.571 |
| scalar | 320 | 0.898 | 0.860 | 7.044 |

Interpretation:

- the canonical grouped binary lane now has a real same-recall direct win:
  - grouped `window=64, ef=64` reaches `0.860 Recall@10` in `1.007ms`
  - scalar needs `ef=40` for the same `0.860 Recall@10`, at `1.398ms`
- grouped still does not win at higher-recall operating points:
  - grouped tops out around `0.874`
  - scalar continues to `0.898`
- the earlier packet `359` canonical `window=16` frontier was directionally
  right, but too pessimistic once the wider live window and rebuilt canonical
  grouped index were both in play

### Verified SQL launcher override surface

The new launcher overrides now work mechanically:

- explicit corpus/query/index names are accepted and verified
- the regression harness covers that path

On the shared canonical `tqhnsw_real_50k_corpus` table, however, the planner
still prefers the scalar index even when the verified launcher is pointed at
`tqhnsw_real_50k_grouped_m8_idx`. Representative plan at `ef=128`:

- planner chose `Index Scan using tqhnsw_real_50k_m8_idx`
- the verified launcher aborted before timing

So the shared-table planner lane is still blocked by planner index choice, not
by the measurement script anymore.

### Isolated grouped-only SQL surface

To validate the new verified-launcher override path end-to-end, I created a
scratch-only isolated grouped corpus:

- table: `tqhnsw_real_50k_grouped_only_corpus`
- index: `tqhnsw_real_50k_grouped_only_m8_idx`

That isolated surface produced clean verified SQL timings:

| path | ef_search | mean SQL latency ms |
|------|-----------|---------------------|
| grouped-only isolated | 128 | 5.971 |
| grouped-only isolated | 200 | 7.173 |
| grouped-only isolated | 320 | 8.800 |

Same query subset on canonical scalar:

| path | ef_search | mean SQL latency ms |
|------|-----------|---------------------|
| scalar | 128 | 7.275 |
| scalar | 200 | 9.510 |
| scalar | 320 | 12.132 |

Important caveat:

- the isolated grouped-only table is a different build surface from the
  canonical shared `50k` corpus
- its direct recall came out materially stronger than the canonical grouped
  index (`0.896 / 0.904 / 0.908` at `ef=128 / 200 / 320`)
- so those SQL numbers prove the override path and grouped-only planner lane,
  but they should not be treated as the canonical `50k` operating point yet

## Outcome

This batch changes the ADR030 status materially:

1. canonical `50k` grouped-v2 binary traversal now has a real same-recall
   direct operating point against scalar
2. widening the live rerank cap was necessary to expose that point
3. the verified SQL launcher can now target explicit grouped-only surfaces
4. the remaining planner problem is now clearly isolated:
   - on the shared canonical table, the planner still prefers the scalar index
   - on an isolated grouped-only table, the grouped lane times cleanly

The next high-signal slice is no longer “does binary traversal work at all?”
It is one of:

- make the shared canonical planner lane choose / isolate the grouped index
  cleanly, or
- explain why the isolated grouped-only build surface is stronger than the
  canonical shared-table grouped build

Either way, the branch now has a concrete path to viability instead of only a
qualitative hint.
