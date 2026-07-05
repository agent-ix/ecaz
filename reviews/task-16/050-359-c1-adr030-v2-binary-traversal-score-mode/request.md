# Review Request: C1 ADR-030 V2 Binary Traversal Score Mode

## Context

Packet `355` proved the grouped-built graph itself was recoverable:

- verified grouped exact traversal on `50k @ ef=128` reached `Recall@10 = 0.878`
- scalar on the same lane was `0.890`

Packets `356` through `358` then closed the cheap exact-like seams around the
existing grouped-PQ traversal score:

- budgeted exact traversal only held up near full exact cost
- frontier-head exactification was effectively inert

That left the upstream candidate signal as the main suspect. Reviewer feedback
on the verified-runtime sequence also kept pointing at the same missing
experiment:

> if the grouped-built graph is mostly fine, can a different approximate
> traversal score recover recall without paying exact-traversal cost?

The repo already had a strong hint for that candidate signal. ADR-031’s binary
sign sidecar study showed high correlation with exact order on real data, and
the grouped-v2 scan path already uses that binary sidecar as a first-stage
rejector. This packet promotes that binary score from “rejector only” to an
optional grouped traversal score mode.

## Problem

Verified grouped-v2 approximate runtime was still too weak to use:

- packet `354`: `50k @ ef=128, window=16` grouped-PQ runtime was `0.674`
  `Recall@10`
- scalar on the same lane was `0.890`

The branch still lacked a verified answer to the highest-signal remaining
question:

> Is the grouped-PQ traversal score itself the core problem, and does replacing
> it with the persisted binary sign score materially improve recall on the
> verified grouped-v2 lane?

The scratch workflow also still had one operational footgun:

- `cargo pgrx install` will happily fall back to the system `pg_config`
  (`pg14` on this machine) unless the `pg17` path is forced explicitly

## Planned Slice

Batch the tightly related work together:

1. add a grouped-v2 traversal score mode surface with:
   - `pq` (default; current behavior)
   - `binary` (new experiment)
2. resolve that mode once per `amrescan`
3. prepare the binary query whenever `binary` mode is selected, even if the
   older binary-prefilter GUC would otherwise suppress it
4. use the binary sign score as the primary grouped traversal score for:
   - entry / single-candidate grouped scoring
   - grouped successor traversal
   - budgeted exact candidate ordering
5. leave full grouped exact traversal behavior unchanged when its gate is on
6. expose the selected traversal score mode in the runtime settings probe and
   scratch helpers
7. add pg coverage that proves grouped-PQ traversal scoring is bypassed in
   `binary` mode
8. add a scratch install helper that forces the correct `pg17` `pg_config`
9. measure verified `50k` first, then `10k`, and compare both against same-cluster
   scalar baselines

This slice intentionally does not:

- change the grouped-v2 on-disk format
- claim a gate-lifted operating point
- redesign the existing budgeted exact traversal path

## Implementation

Updated:

- `src/am/scan.rs`
- `src/lib.rs`
- `scripts/restart_adr030_scratch.sh`
- `scripts/sql/refresh_adr030_scratch_debug_helpers.sql`
- `scripts/install_adr030_pg17_pg_test.sh`

Concrete changes:

1. added `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_GROUPED_SCORE_MODE`
2. added `GroupedTraversalScoreMode::{GroupedPq, Binary}`
3. resolved the grouped traversal score mode once per `amrescan` for
   grouped-v2 descriptors and stored it on `TqScanOpaque`
4. changed prepared-query setup so `binary` mode always prepares the binary sign
   query on the no-QJL 4-bit lane, even if `tqhnsw.disable_binary_prefilter`
   would otherwise suppress the older rejector path
5. rejected `binary` mode with a clear runtime error if the binary-sign lane is
   unavailable
6. added a grouped binary traversal scorer that validates the persisted binary
   sidecar width and scores candidates through
   `score_binary_sign_words_no_qjl_4bit(...)`
7. changed grouped traversal dispatch so:
   - full grouped exact traversal still exact-scores candidates when enabled
   - otherwise, `binary` mode uses the binary score instead of grouped-PQ score
   - the already-computed binary rejector score is reused on the grouped
     successor path instead of recomputing grouped-PQ work
8. extended `tests.tqhnsw_debug_adr030_runtime_settings()` with
   `grouped_scan_score_mode`
9. added pg coverage for:
   - runtime settings reflecting `grouped_scan_score_mode`
   - invalid grouped traversal score-mode env rejection
   - grouped binary mode emitting results while leaving grouped-PQ and grouped
     exact traversal counters at zero
10. extended `scripts/restart_adr030_scratch.sh` with `--grouped-score-mode`
11. updated the scratch debug-helper SQL refresh wrapper to expose the new
    runtime-settings column
12. added `scripts/install_adr030_pg17_pg_test.sh` so the scratch pg-test
    install path forces the correct `pg17` `pg_config`

## Validation

Compile and lint validation on the final code:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

The required lib-test commands were run but are currently blocked by a local
linker environment issue on this workstation:

- `cargo test`
- `/bin/bash -lc 'PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17'`

Both fail during the final test-binary link step with unresolved PostgreSQL
symbols such as `CurrentMemoryContext`, `PG_exception_stack`, and `errstart`.
This failure is local-environment-specific; the scratch runtime build/install
path below succeeds on the same code.

Scratch workflow validation passed:

1. install the new `pg17 pg_test` build with:
   - `scripts/install_adr030_pg17_pg_test.sh`
2. restart scratch with:
   - `./scripts/restart_adr030_scratch.sh --window 16 --grouped-score-mode binary`
3. refresh wrappers with:
   - `./scripts/refresh_adr030_scratch_debug_helpers.sh`
4. verify runtime settings:
   - `grouped_build_enabled = true`
   - `grouped_scan_enabled = true`
   - `grouped_scan_window = 16`
   - `grouped_scan_score_mode = binary`
   - grouped exact traversal disabled
5. verify the `50k` grouped lane is still real grouped-v2:
   - `emitted_result_count = 40`
   - `grouped_result_count = 40`
   - `compared_result_count = 40`

## Measurements

All main measurements below were taken on the verified grouped-v2 lane with:

- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN=1`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW=16`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_GROUPED_SCORE_MODE=binary`

### Verified `50k` grouped-v2 vs scalar at `ef_search = 128`

| path | Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|------|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| grouped binary traversal | 0.8200 | 0.7008 | 0.8823 | 464.24554 | 0.58327276 | 0.8600 | 25 | 4 |
| scalar baseline | 0.8900 | 0.8734 | 0.9289 | 0.0055691516 | 0.7583029 | 0.8600 | 1 | 1 |

Interpretation:

- compared to packet `354`'s verified grouped-PQ runtime (`0.674`), binary
  traversal scoring recovers **14.6 recall points**
- the remaining `50k` gap to scalar at this operating point is now `0.07`
  Recall@10 instead of `0.216`
- the very large absolute score error is expected here because raw binary sign
  scores live on a different numeric scale than the exact `<#>` score

`50k` hot-path sample over the first `10` queries:

| path | amrescan us | graph element load us | candidate score us | grouped approx calls | grouped exact calls | score cache hits | score cache misses | candidate score calls |
|------|-------------|------------------------|--------------------|----------------------|---------------------|------------------|--------------------|-----------------------|
| grouped binary traversal | 1082.7 | 349.6 | 16.1 | 0.0 | 0.0 | 0.0 | 16.0 | 16.0 |
| scalar baseline | 1710.6 | 285.2 | 533.1 | 0.0 | 0.0 | 202.5 | 522.8 | 522.8 |

This confirms two things together:

- grouped-PQ traversal scoring is actually out of the loop in `binary` mode
- the grouped binary lane is still materially cheaper than scalar on the
  hot-path sample at this operating point

### Verified `10k` grouped-v2 vs scalar at `ef_search = 128`

| path | Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|------|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| grouped binary traversal | 0.9260 | 0.8498 | 0.9526 | 784.5221 | 0.8418182 | 0.9170 | 1 | 1 |
| scalar baseline | 0.9400 | 0.93625 | 0.9618 | 0.8653941 | 0.9310 | 0.9310 | 0 | 0 |

Interpretation:

- compared to packet `354`'s verified grouped-PQ runtime (`0.815`), binary
  traversal scoring recovers **11.1 recall points**
- the remaining `10k` gap to scalar is now `0.014` Recall@10
- the large absolute score error is again a scale artifact of using raw binary
  traversal scores instead of exact `<#>` distances

`10k` hot-path sample over the first `10` queries:

| path | amrescan us | graph element load us | candidate score us | grouped approx calls | grouped exact calls | score cache hits | score cache misses | candidate score calls |
|------|-------------|------------------------|--------------------|----------------------|---------------------|------------------|--------------------|-----------------------|
| grouped binary traversal | 1105.8 | 474.6 | 16.9 | 0.0 | 0.0 | 0.0 | 16.0 | 16.0 |
| scalar baseline | 2908.2 | 1958.7 | 333.2 | 0.0 | 0.0 | 205.8 | 328.7 | 328.7 |

### Follow-up note: current budgeted exact path is not binary-compatible yet

After the main checkpoint measurements, I also ran one exploratory local follow-up
on the already-implemented budget seam:

- restart scratch with:
  - `./scripts/restart_adr030_scratch.sh --window 16 --grouped-score-mode binary --exact-limit 1`
- remeasure verified `50k @ ef=128`

That collapsed to:

- `Recall@10 = 0.114`

I am **not** treating that as part of this code slice’s primary claim. The most
likely read is that the current budgeted exact path mixes binary and exact score
scales badly, so “binary + exact budget” is not a free follow-on without more
care. The main checkpoint should therefore be read as:

- plain binary traversal scoring is a strong improvement
- current budgeted exact traversal should stay out of the operating-point story
  until it is made scale-compatible with binary scoring

## Outcome

This packet materially changes the grouped-v2 runtime picture again.

Binary sign scoring is a viable traversal signal on the verified grouped lane.

Compared to the last verified grouped-PQ baseline:

1. `50k @ ef=128, window=16` improves from `0.674` to `0.820`
2. `10k @ ef=128, window=16` improves from `0.815` to `0.926`
3. grouped-PQ traversal work drops to zero in the hot-path profile
4. the grouped binary lane remains materially faster than same-cluster scalar on
   the sampled hot path

This is still not a gate-lift result:

- `50k` is still below scalar by about `0.07` Recall@10 at the tested point
- the main required test commands are currently blocked locally by a linker
  environment issue

But this packet does prove a real path to viability:

- the grouped-built graph is not the blocker
- grouped-PQ traversal score was the dominant quality loss
- persisted binary sign scoring is a much better candidate-selection signal for
  grouped-v2 than the current grouped-PQ traversal score

## Next Slice

The next runtime batch should stay on top of this now-better binary lane:

1. run an `ef_search` sweep on verified `50k` binary traversal scoring to find
   the best same-latency and same-recall comparisons against scalar
2. add a clean planner-facing latency surface for the binary lane once the query
   launcher cannot silently muddy grouped-vs-scalar comparisons
3. if recall still needs more help after the sweep, revisit exact-like follow-on
   work only after making its score mixing compatible with binary traversal
