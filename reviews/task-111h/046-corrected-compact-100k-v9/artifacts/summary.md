# Task 111h corrected compact 100k v9 summary

Source artifacts: `results.jsonl`, `suite-manifest.json`, `suite-report.md`, and per-step logs under `artifacts/suite/`.

Run status: suite completed 65 selected steps, 0 failures, 0 skipped, 0 missing artifacts. Corpus was `ec_real_100k`, dim=1536, k=10, 200 queries, width=64, PG18 local socket `/home/peter/.pgrx`, warm latency cache state `post_recall_warm`.

## Main comparison at nprobe 64 and 200

| cell | r@10 n64 | mean n64 ms | p95 n64 ms | r@10 n200 | mean n200 ms | ec_ivf index MiB | total | build index s |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| source f32 | 0.9720 | 7.72 | 8.49 | 0.9985 | 14.90 | 24.60 | 1.6 GiB | 9.18 |
| index f16 | 0.9710 | 8.16 | 9.68 | 0.9975 | 15.10 | 330.10 | 1.9 GiB | 14.82 |
| index rq4 est c2 | 0.9210 | 5.80 | 6.43 | 0.9380 | 13.40 | 110.20 | 1.7 GiB | 13.35 |
| index rq4 exact c2 | 0.9210 | 6.25 | 7.50 | 0.9365 | 14.60 | 110.20 | 1.7 GiB | 12.75 |
| index rq4 est c3 | 0.9360 | 6.13 | 6.89 | 0.9530 | 13.60 | 110.20 | 1.7 GiB | 11.11 |
| index rq4 exact c3 | 0.9355 | 6.19 | 7.16 | 0.9525 | 14.10 | 110.20 | 1.7 GiB | 11.31 |
| index rq4 est c4 | 0.9260 | 6.31 | 7.17 | 0.9460 | 14.50 | 110.20 | 1.7 GiB | 12.52 |
| index rq4 exact c4 | 0.9240 | 6.38 | 7.20 | 0.9435 | 13.80 | 110.20 | 1.7 GiB | 13.06 |
| index rq8 est c2 | 0.9345 | 6.31 | 7.37 | 0.9525 | 14.70 | 183.60 | 1.7 GiB | 11.73 |
| index rq8 exact c2 | 0.9365 | 6.16 | 7.20 | 0.9535 | 14.90 | 183.60 | 1.7 GiB | 12.18 |
| index rq8 est c3 | 0.9590 | 6.50 | 8.78 | 0.9830 | 14.30 | 183.60 | 1.7 GiB | 12.35 |
| index rq8 exact c3 | 0.9585 | 6.73 | 7.74 | 0.9820 | 14.10 | 183.60 | 1.7 GiB | 11.79 |
| index rq8 est c4 | 0.9670 | 6.46 | 7.49 | 0.9915 | 13.70 | 183.60 | 1.7 GiB | 11.96 |
| index rq8 exact c4 | 0.9670 | 6.42 | 7.36 | 0.9920 | 14.40 | 183.60 | 1.7 GiB | 12.55 |
| index tq default | 0.9375 | 8.33 | 9.92 | 0.9565 | 13.70 | 110.10 | 1.7 GiB | 10.56 |
| index tq exact | 0.9375 | 9.06 | 10.70 | 0.9565 | 14.90 | 110.10 | 1.7 GiB | 10.98 |

## Recall thresholds

| cell | first nprobe r@10 >= 0.9700 | first nprobe r@10 >= 0.9900 | best r@10 | best nprobe |
|---|---:|---:|---:|---:|
| source f32 | 64 | 128 | 0.9985 | 200 |
| index f16 | 64 | 128 | 0.9975 | 200 |
| index rq4 est c2 | not hit | not hit | 0.9380 | 200 |
| index rq4 exact c2 | not hit | not hit | 0.9365 | 200 |
| index rq4 est c3 | not hit | not hit | 0.9530 | 200 |
| index rq4 exact c3 | not hit | not hit | 0.9525 | 200 |
| index rq4 est c4 | not hit | not hit | 0.9460 | 200 |
| index rq4 exact c4 | not hit | not hit | 0.9435 | 200 |
| index rq8 est c2 | not hit | not hit | 0.9525 | 200 |
| index rq8 exact c2 | not hit | not hit | 0.9535 | 200 |
| index rq8 est c3 | 128 | not hit | 0.9830 | 200 |
| index rq8 exact c3 | 128 | not hit | 0.9820 | 200 |
| index rq8 est c4 | 128 | 200 | 0.9915 | 200 |
| index rq8 exact c4 | 128 | 200 | 0.9920 | 200 |
| index tq default | not hit | not hit | 0.9565 | 200 |
| index tq exact | not hit | not hit | 0.9565 | 200 |

## Full recall sweep

| cell | n8 | n16 | n32 | n64 | n128 | n200 |
|---|---:|---:|---:|---:|---:|---:|
| source f32 | 0.7865 | 0.8710 | 0.9350 | 0.9720 | 0.9945 | 0.9985 |
| index f16 | 0.7860 | 0.8705 | 0.9345 | 0.9710 | 0.9935 | 0.9975 |
| index rq4 est c2 | 0.7670 | 0.8420 | 0.8945 | 0.9210 | 0.9365 | 0.9380 |
| index rq4 exact c2 | 0.7675 | 0.8415 | 0.8955 | 0.9210 | 0.9350 | 0.9365 |
| index rq4 est c3 | 0.7745 | 0.8510 | 0.9080 | 0.9360 | 0.9520 | 0.9530 |
| index rq4 exact c3 | 0.7745 | 0.8505 | 0.9065 | 0.9355 | 0.9515 | 0.9525 |
| index rq4 est c4 | 0.7665 | 0.8425 | 0.8985 | 0.9260 | 0.9425 | 0.9460 |
| index rq4 exact c4 | 0.7665 | 0.8420 | 0.8960 | 0.9240 | 0.9405 | 0.9435 |
| index rq8 est c2 | 0.7750 | 0.8515 | 0.9060 | 0.9345 | 0.9495 | 0.9525 |
| index rq8 exact c2 | 0.7750 | 0.8510 | 0.9060 | 0.9365 | 0.9505 | 0.9535 |
| index rq8 est c3 | 0.7825 | 0.8650 | 0.9260 | 0.9590 | 0.9790 | 0.9830 |
| index rq8 exact c3 | 0.7810 | 0.8635 | 0.9250 | 0.9585 | 0.9780 | 0.9820 |
| index rq8 est c4 | 0.7850 | 0.8685 | 0.9305 | 0.9670 | 0.9875 | 0.9915 |
| index rq8 exact c4 | 0.7850 | 0.8685 | 0.9305 | 0.9670 | 0.9880 | 0.9920 |
| index tq default | 0.7745 | 0.8520 | 0.9075 | 0.9375 | 0.9530 | 0.9565 |
| index tq exact | 0.7745 | 0.8520 | 0.9075 | 0.9375 | 0.9530 | 0.9565 |

## Full warm latency mean sweep, ms

| cell | n8 | n16 | n32 | n64 | n128 | n200 |
|---|---:|---:|---:|---:|---:|---:|
| source f32 | 3.98 | 4.58 | 5.50 | 7.72 | 11.40 | 14.90 |
| index f16 | 3.89 | 4.73 | 5.83 | 8.16 | 11.80 | 15.10 |
| index rq4 est c2 | 2.45 | 2.70 | 3.78 | 5.80 | 9.35 | 13.40 |
| index rq4 exact c2 | 2.60 | 3.14 | 4.06 | 6.25 | 9.96 | 14.60 |
| index rq4 est c3 | 2.42 | 2.95 | 3.91 | 6.13 | 9.50 | 13.60 |
| index rq4 exact c3 | 2.48 | 3.04 | 4.01 | 6.19 | 10.00 | 14.10 |
| index rq4 est c4 | 2.90 | 3.34 | 4.34 | 6.31 | 10.60 | 14.50 |
| index rq4 exact c4 | 2.52 | 2.97 | 3.98 | 6.38 | 9.92 | 13.80 |
| index rq8 est c2 | 2.88 | 3.29 | 4.36 | 6.31 | 9.86 | 14.70 |
| index rq8 exact c2 | 2.74 | 3.38 | 4.50 | 6.16 | 10.20 | 14.90 |
| index rq8 est c3 | 2.94 | 3.37 | 4.48 | 6.50 | 10.30 | 14.30 |
| index rq8 exact c3 | 2.82 | 3.32 | 4.73 | 6.73 | 10.60 | 14.10 |
| index rq8 est c4 | 2.90 | 3.41 | 4.45 | 6.46 | 10.30 | 13.70 |
| index rq8 exact c4 | 2.79 | 3.39 | 4.42 | 6.42 | 10.20 | 14.40 |
| index tq default | 2.34 | 4.28 | 5.72 | 8.33 | 10.00 | 13.70 |
| index tq exact | 3.26 | 3.58 | 5.16 | 9.06 | 12.60 | 14.90 |

## Storage

| cell | ec_ivf index | ec_ivf per row | all indexes | table | total |
|---|---:|---:|---:|---:|---:|
| source f32 | 24.6 MiB | 258.2 B | 26.8 MiB | 1.6 GiB | 1.6 GiB |
| index f16 | 330.1 MiB | 3461.8 B | 332.3 MiB | 1.6 GiB | 1.9 GiB |
| index rq4 est c2 | 110.2 MiB | 1155.2 B | 112.3 MiB | 1.6 GiB | 1.7 GiB |
| index rq4 exact c2 | 110.2 MiB | 1155.2 B | 112.3 MiB | 1.6 GiB | 1.7 GiB |
| index rq4 est c3 | 110.2 MiB | 1155.2 B | 112.3 MiB | 1.6 GiB | 1.7 GiB |
| index rq4 exact c3 | 110.2 MiB | 1155.2 B | 112.3 MiB | 1.6 GiB | 1.7 GiB |
| index rq4 est c4 | 110.2 MiB | 1155.2 B | 112.3 MiB | 1.6 GiB | 1.7 GiB |
| index rq4 exact c4 | 110.2 MiB | 1155.2 B | 112.3 MiB | 1.6 GiB | 1.7 GiB |
| index rq8 est c2 | 183.6 MiB | 1925.4 B | 185.8 MiB | 1.6 GiB | 1.7 GiB |
| index rq8 exact c2 | 183.6 MiB | 1925.4 B | 185.8 MiB | 1.6 GiB | 1.7 GiB |
| index rq8 est c3 | 183.6 MiB | 1925.4 B | 185.8 MiB | 1.6 GiB | 1.7 GiB |
| index rq8 exact c3 | 183.6 MiB | 1925.4 B | 185.8 MiB | 1.6 GiB | 1.7 GiB |
| index rq8 est c4 | 183.6 MiB | 1925.4 B | 185.8 MiB | 1.6 GiB | 1.7 GiB |
| index rq8 exact c4 | 183.6 MiB | 1925.4 B | 185.8 MiB | 1.6 GiB | 1.7 GiB |
| index tq default | 110.1 MiB | 1154.4 B | 112.2 MiB | 1.6 GiB | 1.7 GiB |
| index tq exact | 110.1 MiB | 1154.4 B | 112.2 MiB | 1.6 GiB | 1.7 GiB |

## Immediate readouts

- Source f32 and index f16 remain the only cells that hit recall@10 >= 0.97 at nprobe 64 and recall@10 >= 0.99 at nprobe 128 in this 100k/w64 slice.
- Index f16 has nearly identical recall to source f32, but is slower at nprobe 64 (8.16 ms versus 7.72 ms), slower at nprobe 200 (15.10 ms versus 14.90 ms), and uses a much larger ec_ivf index (330.1 MiB versus 24.6 MiB).
- Best RaBitQ-4 recall is `index rq4 est c3`: best recall@10 0.9530 at nprobe 200, nprobe 64 mean 6.13 ms, ec_ivf index 110.2 MiB. RaBitQ-4 does not hit recall@10 >= 0.97 or 0.99 on 100k/w64.
- Best RaBitQ-8 recall is `index rq8 exact c4`: best recall@10 0.9920 at nprobe 200, nprobe 64 mean 6.42 ms, ec_ivf index 183.6 MiB. The estimator variant is close at recall@10 0.9915 and faster at nprobe 200 (13.70 ms versus 14.40 ms).
- At the 0.97 recall target, RaBitQ-8 clip 4 first qualifies at nprobe 128 with 10.20 to 10.30 ms mean latency. Source f32 qualifies at nprobe 64 with 7.72 ms, and index f16 qualifies at nprobe 64 with 8.16 ms.
- At the 0.99 recall target, RaBitQ-8 clip 4 first qualifies at nprobe 200 with 13.70 to 14.40 ms mean latency. Source f32 qualifies at nprobe 128 with 11.40 ms, and index f16 qualifies at nprobe 128 with 11.80 ms.
- TurboQuant exact-dequant did not improve recall in this slice: default and exact both best recall@10 0.9565. Exact was slower at nprobe 64 (9.06 ms versus 8.33 ms) and nprobe 200 (14.90 ms versus 13.70 ms).
- This is a corrected 100k sweep, not a 111h closeout. It should feed the final matched-recall decision and any final-scale locked run.
