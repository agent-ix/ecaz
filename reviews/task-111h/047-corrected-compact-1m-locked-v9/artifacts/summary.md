# Task 111h corrected compact 1M locked v9 summary

Source artifacts: `suite-manifest.json`, `results.jsonl`,
`suite-report-results.jsonl`, `suite-report.md`, and per-step logs under
`artifacts/suite/`.

Run status: suite completed 44 selected steps, 0 failures, 0 skipped, 0 missing
artifacts. Corpus was DBPedia OpenAI3 staged 1M with 990000 corpus rows and
10000 query rows, `dim=1536`, `k=10`, 100 measured queries, `rerank_width=64`,
PG18 local socket `/home/peter/.pgrx`, warm latency cache state
`post_recall_warm`, and nprobe sweep `8,16,32,64,128,200`.

## Main Comparison

| cell | r@10 n64 | mean n64 ms | p95 n64 ms | r@10 n128 | mean n128 ms | r@10 n200 | mean n200 ms | ec_ivf index | build index s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| source f32 | 0.9770 | 18.7 | 21.3 | 0.9860 | 29.9 | 0.9880 | 40.6 | 226.8 MiB | 196.92 |
| index f16 | 0.9770 | 21.4 | 26.8 | 0.9860 | 31.2 | 0.9880 | 43.0 | 3.2 GiB | 215.16 |
| index rq4 est c3 | 0.9290 | 18.6 | 22.7 | 0.9380 | 28.5 | 0.9400 | 41.0 | 1.0 GiB | 186.99 |
| index rq8 est c4 | 0.9730 | 18.1 | 20.9 | 0.9820 | 30.1 | 0.9840 | 42.3 | 1.8 GiB | 192.93 |
| index rq8 exact c4 | 0.9730 | 17.9 | 20.5 | 0.9820 | 30.2 | 0.9840 | 54.3 | 1.8 GiB | 193.52 |
| index tq default | 0.9400 | 18.1 | 20.8 | 0.9480 | 28.7 | 0.9490 | 41.7 | 1.0 GiB | 180.89 |
| index tq exact | 0.9390 | 18.0 | 21.1 | 0.9470 | 28.5 | 0.9480 | 45.5 | 1.0 GiB | 186.61 |

## Recall Thresholds

| cell | first nprobe r@10 >= 0.9700 | first nprobe r@10 >= 0.9900 | best r@10 | best nprobe |
| --- | ---: | ---: | ---: | ---: |
| source f32 | 64 | not hit | 0.9880 | 200 |
| index f16 | 64 | not hit | 0.9880 | 200 |
| index rq4 est c3 | not hit | not hit | 0.9400 | 200 |
| index rq8 est c4 | 64 | not hit | 0.9840 | 200 |
| index rq8 exact c4 | 64 | not hit | 0.9840 | 200 |
| index tq default | not hit | not hit | 0.9490 | 200 |
| index tq exact | not hit | not hit | 0.9480 | 200 |

## Full Recall Sweep

| cell | n8 | n16 | n32 | n64 | n128 | n200 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| source f32 | 0.8930 | 0.9240 | 0.9570 | 0.9770 | 0.9860 | 0.9880 |
| index f16 | 0.8930 | 0.9240 | 0.9570 | 0.9770 | 0.9860 | 0.9880 |
| index rq4 est c3 | 0.8590 | 0.8860 | 0.9140 | 0.9290 | 0.9380 | 0.9400 |
| index rq8 est c4 | 0.8900 | 0.9210 | 0.9530 | 0.9730 | 0.9820 | 0.9840 |
| index rq8 exact c4 | 0.8900 | 0.9210 | 0.9530 | 0.9730 | 0.9820 | 0.9840 |
| index tq default | 0.8660 | 0.8930 | 0.9220 | 0.9400 | 0.9480 | 0.9490 |
| index tq exact | 0.8650 | 0.8920 | 0.9210 | 0.9390 | 0.9470 | 0.9480 |

## Full Warm Latency Mean Sweep, ms

| cell | n8 | n16 | n32 | n64 | n128 | n200 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| source f32 | 9.04 | 10.5 | 13.2 | 18.7 | 29.9 | 40.6 |
| index f16 | 10.4 | 13.0 | 15.6 | 21.4 | 31.2 | 43.0 |
| index rq4 est c3 | 7.49 | 8.66 | 11.4 | 18.6 | 28.5 | 41.0 |
| index rq8 est c4 | 8.00 | 9.35 | 12.2 | 18.1 | 30.1 | 42.3 |
| index rq8 exact c4 | 8.14 | 9.54 | 12.2 | 17.9 | 30.2 | 54.3 |
| index tq default | 7.58 | 9.04 | 11.6 | 18.1 | 28.7 | 41.7 |
| index tq exact | 8.06 | 9.26 | 12.0 | 18.0 | 28.5 | 45.5 |

## Storage

| cell | ec_ivf index | ec_ivf per row |
| --- | ---: | ---: |
| source f32 | 226.8 MiB | 240.2 B |
| index f16 | 3.2 GiB | 3441.4 B |
| index rq4 est c3 | 1.0 GiB | 1136.5 B |
| index rq8 est c4 | 1.8 GiB | 1905.1 B |
| index rq8 exact c4 | 1.8 GiB | 1905.1 B |
| index tq default | 1.0 GiB | 1136.2 B |
| index tq exact | 1.0 GiB | 1136.2 B |

## Immediate Readouts

- All seven 1M locked cells completed under the same width and nprobe sweep.
- Source f32 and index f16 have identical recall in this run. Index f16 is
  slower at the measured latency sweep and much larger on disk than source f32.
- No measured 1M/w64 cell reaches recall@10 >= 0.9900 by nprobe 200.
- Source f32, index f16, and RaBitQ-8 clip 4 reach recall@10 >= 0.9700 at
  nprobe 64. RQ8 estimator is slightly faster than source f32 at nprobe 64
  latency in this run, but its recall is lower at every measured high nprobe.
- RQ8 exact-dequant does not improve recall over the RQ8 estimator in this
  run. It is similar at nprobe 64/128 and worse at nprobe 200 latency.
- RQ4 and TurboQuant are storage-efficient relative to f16, but do not reach
  recall@10 >= 0.9700 at width 64 on this 1M slice.
- TurboQuant exact-dequant does not improve recall over default in this run.
