# Task 111h corrected compact 50k v9 summary

Source artifacts: `results.jsonl`, `suite-manifest.json`, `suite-report.md`, and per-step logs under `artifacts/suite/`.

Run status: suite completed 65 selected steps, 0 failures, 0 skipped, 0 missing artifacts. Corpus was `ec_real_50k`, dim=1536, k=10, 200 queries, width=64, PG18 local socket `/home/peter/.pgrx`, warm latency cache state `post_recall_warm`.

## Main comparison at nprobe 64 and 200

| cell | r@10 n64 | mean n64 ms | p95 n64 ms | r@10 n200 | mean n200 ms | ec_ivf index MiB | total MiB | build index s |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| source f32 | 0.9820 | 5.54 | 6.10 | 0.9985 | 10.50 | 13.80 | 808.70 | 5.21 |
| index f16 | 0.9820 | 5.57 | 6.86 | 0.9985 | 9.34 | 166.70 | 961.50 | 7.67 |
| index rq4 est c2 | 0.9345 | 4.32 | 4.98 | 0.9455 | 9.18 | 56.60 | 851.50 | 6.73 |
| index rq4 exact c2 | 0.9330 | 4.34 | 5.04 | 0.9435 | 8.75 | 56.60 | 851.50 | 6.73 |
| index rq4 est c3 | 0.9475 | 4.00 | 4.43 | 0.9605 | 7.58 | 56.60 | 851.50 | 7.49 |
| index rq4 exact c3 | 0.9465 | 4.00 | 4.45 | 0.9605 | 7.91 | 56.60 | 851.50 | 6.54 |
| index rq4 est c4 | 0.9405 | 3.96 | 4.43 | 0.9485 | 7.96 | 56.60 | 851.50 | 7.33 |
| index rq4 exact c4 | 0.9435 | 4.54 | 5.38 | 0.9535 | 9.03 | 56.60 | 851.50 | 6.99 |
| index rq8 est c2 | 0.9425 | 4.59 | 5.47 | 0.9530 | 8.30 | 93.40 | 888.30 | 6.97 |
| index rq8 exact c2 | 0.9430 | 5.47 | 6.92 | 0.9535 | 8.10 | 93.40 | 888.30 | 6.39 |
| index rq8 est c3 | 0.9710 | 4.81 | 6.14 | 0.9865 | 8.54 | 93.40 | 888.30 | 7.28 |
| index rq8 exact c3 | 0.9710 | 4.38 | 5.16 | 0.9865 | 8.07 | 93.40 | 888.30 | 6.97 |
| index rq8 est c4 | 0.9770 | 4.24 | 5.08 | 0.9930 | 8.55 | 93.40 | 888.30 | 6.69 |
| index rq8 exact c4 | 0.9770 | 4.52 | 5.26 | 0.9930 | 9.53 | 93.40 | 888.30 | 7.38 |
| index tq default | 0.9475 | 4.41 | 5.83 | 0.9590 | 10.50 | 56.60 | 851.50 | 6.63 |
| index tq exact | 0.9475 | 6.82 | 7.92 | 0.9590 | 8.68 | 56.60 | 851.50 | 6.43 |

## Recall thresholds

| cell | first nprobe r@10 >= 0.9700 | first nprobe r@10 >= 0.9900 | best r@10 | best nprobe |
|---|---:|---:|---:|---:|
| source f32 | 64 | 128 | 0.9985 | 200 |
| index f16 | 64 | 128 | 0.9985 | 200 |
| index rq4 est c2 | not hit | not hit | 0.9455 | 200 |
| index rq4 exact c2 | not hit | not hit | 0.9435 | 200 |
| index rq4 est c3 | not hit | not hit | 0.9605 | 200 |
| index rq4 exact c3 | not hit | not hit | 0.9605 | 200 |
| index rq4 est c4 | not hit | not hit | 0.9485 | 200 |
| index rq4 exact c4 | not hit | not hit | 0.9535 | 200 |
| index rq8 est c2 | not hit | not hit | 0.9530 | 200 |
| index rq8 exact c2 | not hit | not hit | 0.9535 | 200 |
| index rq8 est c3 | 64 | not hit | 0.9865 | 200 |
| index rq8 exact c3 | 64 | not hit | 0.9865 | 200 |
| index rq8 est c4 | 64 | 128 | 0.9930 | 200 |
| index rq8 exact c4 | 64 | 128 | 0.9930 | 200 |
| index tq default | not hit | not hit | 0.9590 | 200 |
| index tq exact | not hit | not hit | 0.9590 | 200 |

## Full recall sweep

| cell | n8 | n16 | n32 | n64 | n128 | n200 |
|---|---:|---:|---:|---:|---:|---:|
| source f32 | 0.8595 | 0.9200 | 0.9590 | 0.9820 | 0.9965 | 0.9985 |
| index f16 | 0.8595 | 0.9200 | 0.9590 | 0.9820 | 0.9965 | 0.9985 |
| index rq4 est c2 | 0.8375 | 0.8895 | 0.9200 | 0.9345 | 0.9445 | 0.9455 |
| index rq4 exact c2 | 0.8365 | 0.8900 | 0.9200 | 0.9330 | 0.9425 | 0.9435 |
| index rq4 est c3 | 0.8450 | 0.8980 | 0.9305 | 0.9475 | 0.9590 | 0.9605 |
| index rq4 exact c3 | 0.8450 | 0.8980 | 0.9300 | 0.9465 | 0.9585 | 0.9605 |
| index rq4 est c4 | 0.8385 | 0.8890 | 0.9205 | 0.9405 | 0.9475 | 0.9485 |
| index rq4 exact c4 | 0.8390 | 0.8890 | 0.9225 | 0.9435 | 0.9520 | 0.9535 |
| index rq8 est c2 | 0.8420 | 0.8950 | 0.9260 | 0.9425 | 0.9520 | 0.9530 |
| index rq8 exact c2 | 0.8405 | 0.8950 | 0.9265 | 0.9430 | 0.9520 | 0.9535 |
| index rq8 est c3 | 0.8550 | 0.9120 | 0.9505 | 0.9710 | 0.9850 | 0.9865 |
| index rq8 exact c3 | 0.8550 | 0.9120 | 0.9505 | 0.9710 | 0.9850 | 0.9865 |
| index rq8 est c4 | 0.8585 | 0.9165 | 0.9550 | 0.9770 | 0.9915 | 0.9930 |
| index rq8 exact c4 | 0.8585 | 0.9165 | 0.9550 | 0.9770 | 0.9915 | 0.9930 |
| index tq default | 0.8445 | 0.9005 | 0.9325 | 0.9475 | 0.9580 | 0.9590 |
| index tq exact | 0.8445 | 0.9005 | 0.9325 | 0.9475 | 0.9580 | 0.9590 |

## Full warm latency mean sweep, ms

| cell | n8 | n16 | n32 | n64 | n128 | n200 |
|---|---:|---:|---:|---:|---:|---:|
| source f32 | 3.78 | 4.09 | 4.52 | 5.54 | 7.09 | 10.50 |
| index f16 | 4.83 | 4.47 | 4.56 | 5.57 | 7.59 | 9.34 |
| index rq4 est c2 | 2.33 | 2.68 | 3.10 | 4.32 | 6.09 | 9.18 |
| index rq4 exact c2 | 2.28 | 2.62 | 3.33 | 4.34 | 6.32 | 8.75 |
| index rq4 est c3 | 2.11 | 2.49 | 2.93 | 4.00 | 6.03 | 7.58 |
| index rq4 exact c3 | 2.14 | 2.52 | 3.14 | 4.00 | 5.91 | 7.91 |
| index rq4 est c4 | 2.15 | 2.31 | 2.86 | 3.96 | 5.87 | 7.96 |
| index rq4 exact c4 | 2.30 | 2.79 | 3.28 | 4.54 | 6.25 | 9.03 |
| index rq8 est c2 | 2.90 | 3.51 | 4.15 | 4.59 | 6.36 | 8.30 |
| index rq8 exact c2 | 2.54 | 2.90 | 3.65 | 5.47 | 6.47 | 8.10 |
| index rq8 est c3 | 2.74 | 3.17 | 3.67 | 4.81 | 6.73 | 8.54 |
| index rq8 exact c3 | 2.63 | 2.93 | 3.45 | 4.38 | 6.20 | 8.07 |
| index rq8 est c4 | 2.31 | 2.75 | 3.39 | 4.24 | 5.99 | 8.55 |
| index rq8 exact c4 | 2.74 | 3.06 | 3.57 | 4.52 | 6.61 | 9.53 |
| index tq default | 2.35 | 2.69 | 3.64 | 4.41 | 8.62 | 10.50 |
| index tq exact | 2.78 | 3.18 | 5.00 | 6.82 | 8.34 | 8.68 |

## Storage

| cell | ec_ivf index | ec_ivf per row | all indexes | table | total |
|---|---:|---:|---:|---:|---:|
| source f32 | 13.8 MiB | 290.3 B | 14.9 MiB | 793.8 MiB | 808.7 MiB |
| index f16 | 166.7 MiB | 3495.5 B | 167.8 MiB | 793.8 MiB | 961.5 MiB |
| index rq4 est c2 | 56.6 MiB | 1187.8 B | 57.7 MiB | 793.8 MiB | 851.5 MiB |
| index rq4 exact c2 | 56.6 MiB | 1187.8 B | 57.7 MiB | 793.8 MiB | 851.5 MiB |
| index rq4 est c3 | 56.6 MiB | 1187.8 B | 57.7 MiB | 793.8 MiB | 851.5 MiB |
| index rq4 exact c3 | 56.6 MiB | 1187.8 B | 57.7 MiB | 793.8 MiB | 851.5 MiB |
| index rq4 est c4 | 56.6 MiB | 1187.8 B | 57.7 MiB | 793.8 MiB | 851.5 MiB |
| index rq4 exact c4 | 56.6 MiB | 1187.8 B | 57.7 MiB | 793.8 MiB | 851.5 MiB |
| index rq8 est c2 | 93.4 MiB | 1959.5 B | 94.5 MiB | 793.8 MiB | 888.3 MiB |
| index rq8 exact c2 | 93.4 MiB | 1959.5 B | 94.5 MiB | 793.8 MiB | 888.3 MiB |
| index rq8 est c3 | 93.4 MiB | 1959.5 B | 94.5 MiB | 793.8 MiB | 888.3 MiB |
| index rq8 exact c3 | 93.4 MiB | 1959.5 B | 94.5 MiB | 793.8 MiB | 888.3 MiB |
| index rq8 est c4 | 93.4 MiB | 1959.5 B | 94.5 MiB | 793.8 MiB | 888.3 MiB |
| index rq8 exact c4 | 93.4 MiB | 1959.5 B | 94.5 MiB | 793.8 MiB | 888.3 MiB |
| index tq default | 56.6 MiB | 1187.0 B | 57.7 MiB | 793.8 MiB | 851.5 MiB |
| index tq exact | 56.6 MiB | 1187.0 B | 57.7 MiB | 793.8 MiB | 851.5 MiB |

## Immediate readouts

- Source f32 and index f16 have identical recall in this 50k/w64 slice: r@10 0.9820 at nprobe 64 and 0.9985 at nprobe 200. Index f16 is slightly slower at nprobe 64 here (5.57 ms versus source f32 5.54 ms) and carries a much larger ec_ivf index (166.70 MiB versus source f32 13.80 MiB).
- Best RaBitQ-4 recall in this slice is `index rq4 est c3`: best r@10 0.9605 at nprobe 200, nprobe 64 mean 4.00 ms, ec_ivf index 56.6 MiB. RaBitQ-4 does not hit r@10 >= 0.97 or 0.99 on 50k/w64.
- Best RaBitQ-8 recall in this slice is `index rq8 est c4`: best r@10 0.9930 at nprobe 200, nprobe 64 mean 4.24 ms, ec_ivf index 93.4 MiB. It first hits r@10 >= 0.99 at nprobe 128.
- RaBitQ-8 clip 4, estimator and exact-dequant, have identical recall at every nprobe in this run. Exact-dequant is slower at nprobe 64 (4.52 ms versus estimator 4.24 ms) and at nprobe 200 (9.53 ms versus 8.55 ms).
- TurboQuant exact-dequant did not improve recall in this slice: default and exact both best r@10 0.9590. Exact was slower at nprobe 64 (6.82 ms versus default 4.41 ms).
- At the 0.99 recall target, RaBitQ-8 clip 4 reaches r@10 0.9915 at nprobe 128 with 5.99 ms mean latency and 93.4 MiB ec_ivf index storage. Index f16/source exceed 0.99 at nprobe 128 with r@10 0.9965, but index f16 uses 166.7 MiB ec_ivf index storage and mean latency 7.59 ms.
- This is a corrected 50k sweep, not a 111h closeout. It does not replace the required corrected 100k/final scale sweeps or final matched-recall decision across 0.97/0.99 targets.
