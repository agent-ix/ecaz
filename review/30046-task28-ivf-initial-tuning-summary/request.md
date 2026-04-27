# Review Request: Task 28 IVF Initial Tuning Summary

## Summary

This packet summarizes the IVF-first initial tuning lane through commit
`14ce246`. The work stayed local, PG18-only for default Postgres validation, and
did not include DiskANN implementation or measurement.

## Code Landed

- `7e5a2b3`: added the `ec_ivf` ecaz benchmark profile.
- `9b42a71`: added IVF `rerank = 'heap_f32'` by heap-fetching indexed
  `ecvector` values and rescoring with the SQL `<#>`-compatible raw-f32 path.
- `4d894bd`: added `rerank_width` so heap rerank can operate on a bounded
  approximate frontier instead of the full probed candidate set.

Focused validation for the code slices included:

- `cargo test -p ecaz-cli`
- `cargo test --lib test_ec_ivf_heap_f32 --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
- `cargo test --lib test_ec_ivf_gettuple_emits_probe_candidates_with_scores --no-default-features --features pg18`
- `cargo test --lib test_ec_ivf_full_probe_matches_simple_exact_oracle_top1 --no-default-features --features pg18`
- `git diff --check`

The broader `cargo test --lib ec_ivf --no-default-features --features pg18`
compiled and passed the non-concurrent IVF tests reached before failing the
three existing concurrent-insert tests because the pg_test backend could not
spawn `psql` from its process environment.

## Measurement Packets

- 30035: `ec_ivf` CLI smoke profile.
- 30036: DBPedia 10k x 1536 nprobe debug; confirmed `ec_ivf.nprobe` is honored.
- 30037: full-probe scorer alignment; found full-probe `rerank=off` misses were
  scorer-order drift, not reachability.
- 30038: first heap-rerank smoke; full-probe recall reached `1.0000`, but full
  frontier p50 was about `686 ms`.
- 30039: `rerank_width` sweep; width 50 recovered `1.0000` at about `181 ms`
  p50 on the 20-query 10k slice.
- 30040: `nprobe x rerank_width`; routing breadth dominated recall.
- 30041: `nlists x nprobe`; larger `nlists` did not improve the 10k full-recall
  latency/build frontier.
- 30042: 100-query 10k anchor check.
- 30043: 100-query 10k midprobe check.
- 30044: 25k candidate check.
- 30045: 25k routing followup.

## Current Local Frontier

10k x 1536, 100 ordered queries:

| point | recall@10 | p50 | p95 | build |
|---|---:|---:|---:|---:|
| `32/16,width=50` | `0.9800` | `100.013 ms` | `120.946 ms` | `24.418 s` |
| `32/24,width=25` | `0.9980` | `135.073 ms` | `146.331 ms` | `24.340 s` |
| `32/32,width=25` | `1.0000` | `177.806 ms` | `202.947 ms` | `24.329 s` |

25k x 1536, 100 ordered queries:

| point | recall@10 | p50 | p95 | build |
|---|---:|---:|---:|---:|
| `32/24,width=25` | `0.9760` | `331.674 ms` | `371.690 ms` | `46.138 s` |
| `32/28,width=25` | `0.9830` | `382.821 ms` | `414.706 ms` | `46.086 s` |
| `32/32,width=25` | `1.0000` | `434.858 ms` | `456.380 ms` | `45.068 s` |
| `64/48,width=25` | `1.0000` | `433.318 ms` | `452.825 ms` | `74.444 s` |

## Interpretation

The IVF path is now usable for local tuning and has a clear correctness knob:
`heap_f32` rerank fixes the full-probe scorer drift. `rerank_width` makes that
path practical by exact-reranking a small approximate frontier.

On these local DBPedia-derived slices:

- `nlists=32` is the best local default so far.
- `rerank_width=25` is enough for the measured high-recall points.
- recall loss is mainly routing miss, not rerank width, once width is at least
  25.
- increasing `nlists` to 64/128 increased build cost and did not improve the
  measured full-recall latency frontier.
- 25k/100 exact truth took `18:46.749`, so larger local exact baselines need
  reuse/caching or fewer queries.

## Recommended Next Slice

Stop Task 28 initial local tuning here unless a reviewer asks for a specific
extra data point. For the next IVF slice:

1. Reuse the 25k exact table instead of recomputing truth.
2. Add planner/bench ergonomics around `rerank_width` in ecaz so sweeps do not
   require bespoke SQL.
3. Only after that, try a larger corpus or a managed Graviton-class benchmark.

Product claims remain blocked on a dedicated Graviton-class benchmark. DiskANN
belongs to task 29 and is not part of this packet.

## Artifacts

This summary cites the packet-local artifacts in review packets 30035 through
30045. No new measurement log is introduced here.

## Validation

Packet-only change.

- `git diff --check`
