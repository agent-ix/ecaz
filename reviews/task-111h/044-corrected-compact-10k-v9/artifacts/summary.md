# Task 111h corrected compact 10k v9 summary

Source artifacts: `results.jsonl`, `suite-manifest.json`, `suite-report.md`, and per-step logs under `artifacts/suite/`.

Run status: suite completed 65 selected steps, 0 failures, 0 skipped, 0 missing artifacts. Corpus was `ec_real_10k`, dim=1536, k=10, 200 queries, width=64, PG18 local socket `/home/peter/.pgrx`, warm latency cache state `post_recall_warm`.

## Main comparison at nprobe 64 and 200

| cell | r@10 n64 | mean n64 ms | p95 n64 ms | r@10 n200 | mean n200 ms | ec_ivf index MiB | total MiB | build index s |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| source f32 | 1.0000 | 3.97 | 4.35 | 1.0000 | 4.93 | 5.10 | 164.20 | 2.42 |
| index f16 | 0.9990 | 2.57 | 3.33 | 0.9990 | 3.48 | 36.00 | 195.00 | 2.76 |
| index rq4 est c2 | 0.9790 | 2.34 | 3.34 | 0.9790 | 4.44 | 13.90 | 173.00 | 2.77 |
| index rq4 exact c2 | 0.9800 | 2.32 | 2.76 | 0.9800 | 3.37 | 13.90 | 173.00 | 2.57 |
| index rq4 est c3 | 0.9835 | 2.40 | 2.82 | 0.9835 | 3.32 | 13.90 | 173.00 | 2.74 |
| index rq4 exact c3 | 0.9830 | 2.49 | 2.82 | 0.9830 | 3.36 | 13.90 | 173.00 | 2.87 |
| index rq4 est c4 | 0.9780 | 2.43 | 2.74 | 0.9780 | 3.32 | 13.90 | 173.00 | 2.65 |
| index rq4 exact c4 | 0.9780 | 2.73 | 3.35 | 0.9780 | 2.85 | 13.90 | 173.00 | 2.86 |
| index rq8 est c2 | 0.9865 | 2.31 | 2.79 | 0.9865 | 3.13 | 21.30 | 180.40 | 2.61 |
| index rq8 exact c2 | 0.9870 | 2.17 | 2.63 | 0.9870 | 3.13 | 21.30 | 180.40 | 2.58 |
| index rq8 est c3 | 0.9950 | 2.35 | 2.99 | 0.9950 | 3.17 | 21.30 | 180.40 | 2.67 |
| index rq8 exact c3 | 0.9950 | 2.16 | 2.55 | 0.9950 | 3.32 | 21.30 | 180.40 | 2.61 |
| index rq8 est c4 | 0.9990 | 2.14 | 2.55 | 0.9990 | 2.94 | 21.30 | 180.40 | 2.91 |
| index rq8 exact c4 | 0.9990 | 2.21 | 2.61 | 0.9990 | 3.06 | 21.30 | 180.40 | 2.64 |
| index tq default | 0.9815 | 2.16 | 2.63 | 0.9815 | 3.07 | 13.90 | 172.90 | 2.47 |
| index tq exact | 0.9815 | 3.83 | 4.68 | 0.9815 | 3.54 | 13.90 | 172.90 | 2.67 |

## Recall thresholds

| cell | first nprobe r@10 >= 0.9700 | first nprobe r@10 >= 0.9900 | best r@10 | best nprobe |
|---|---:|---:|---:|---:|
| source f32 | 8 | 16 | 1.0000 | 200 |
| index f16 | 8 | 16 | 0.9990 | 200 |
| index rq4 est c2 | 8 | not hit | 0.9790 | 200 |
| index rq4 exact c2 | 8 | not hit | 0.9800 | 200 |
| index rq4 est c3 | 8 | not hit | 0.9835 | 200 |
| index rq4 exact c3 | 8 | not hit | 0.9830 | 200 |
| index rq4 est c4 | 16 | not hit | 0.9780 | 200 |
| index rq4 exact c4 | 16 | not hit | 0.9780 | 200 |
| index rq8 est c2 | 8 | not hit | 0.9865 | 200 |
| index rq8 exact c2 | 8 | not hit | 0.9870 | 200 |
| index rq8 est c3 | 8 | 16 | 0.9950 | 200 |
| index rq8 exact c3 | 8 | 16 | 0.9950 | 200 |
| index rq8 est c4 | 8 | 16 | 0.9990 | 200 |
| index rq8 exact c4 | 8 | 16 | 0.9990 | 200 |
| index tq default | 8 | not hit | 0.9815 | 200 |
| index tq exact | 8 | not hit | 0.9815 | 200 |

## Full recall sweep

| cell | n8 | n16 | n32 | n64 | n128 | n200 |
|---|---:|---:|---:|---:|---:|---:|
| source f32 | 0.9895 | 0.9970 | 0.9985 | 1.0000 | 1.0000 | 1.0000 |
| index f16 | 0.9890 | 0.9960 | 0.9975 | 0.9990 | 0.9990 | 0.9990 |
| index rq4 est c2 | 0.9705 | 0.9770 | 0.9780 | 0.9790 | 0.9790 | 0.9790 |
| index rq4 exact c2 | 0.9715 | 0.9775 | 0.9785 | 0.9800 | 0.9800 | 0.9800 |
| index rq4 est c3 | 0.9760 | 0.9810 | 0.9825 | 0.9835 | 0.9835 | 0.9835 |
| index rq4 exact c3 | 0.9755 | 0.9805 | 0.9820 | 0.9830 | 0.9830 | 0.9830 |
| index rq4 est c4 | 0.9695 | 0.9760 | 0.9765 | 0.9780 | 0.9780 | 0.9780 |
| index rq4 exact c4 | 0.9695 | 0.9760 | 0.9765 | 0.9780 | 0.9780 | 0.9780 |
| index rq8 est c2 | 0.9780 | 0.9840 | 0.9850 | 0.9865 | 0.9865 | 0.9865 |
| index rq8 exact c2 | 0.9785 | 0.9845 | 0.9855 | 0.9870 | 0.9870 | 0.9870 |
| index rq8 est c3 | 0.9850 | 0.9920 | 0.9935 | 0.9950 | 0.9950 | 0.9950 |
| index rq8 exact c3 | 0.9850 | 0.9920 | 0.9935 | 0.9950 | 0.9950 | 0.9950 |
| index rq8 est c4 | 0.9885 | 0.9960 | 0.9975 | 0.9990 | 0.9990 | 0.9990 |
| index rq8 exact c4 | 0.9885 | 0.9960 | 0.9975 | 0.9990 | 0.9990 | 0.9990 |
| index tq default | 0.9730 | 0.9790 | 0.9800 | 0.9815 | 0.9815 | 0.9815 |
| index tq exact | 0.9730 | 0.9790 | 0.9800 | 0.9815 | 0.9815 | 0.9815 |

## Full warm latency mean sweep, ms

| cell | n8 | n16 | n32 | n64 | n128 | n200 |
|---|---:|---:|---:|---:|---:|---:|
| source f32 | 3.69 | 3.75 | 3.95 | 3.97 | 4.34 | 4.93 |
| index f16 | 2.03 | 2.20 | 2.41 | 2.57 | 2.98 | 3.48 |
| index rq4 est c2 | 1.72 | 1.80 | 1.86 | 2.34 | 3.77 | 4.44 |
| index rq4 exact c2 | 2.02 | 2.06 | 2.24 | 2.32 | 2.90 | 3.37 |
| index rq4 est c3 | 1.96 | 1.99 | 2.10 | 2.40 | 2.74 | 3.32 |
| index rq4 exact c3 | 2.02 | 2.08 | 2.22 | 2.49 | 2.94 | 3.36 |
| index rq4 est c4 | 1.95 | 1.98 | 2.16 | 2.43 | 2.79 | 3.32 |
| index rq4 exact c4 | 2.04 | 2.13 | 2.22 | 2.73 | 2.67 | 2.85 |
| index rq8 est c2 | 1.83 | 1.98 | 2.03 | 2.31 | 2.79 | 3.13 |
| index rq8 exact c2 | 1.86 | 1.87 | 1.94 | 2.17 | 2.63 | 3.13 |
| index rq8 est c3 | 1.82 | 1.83 | 2.17 | 2.35 | 2.70 | 3.17 |
| index rq8 exact c3 | 2.25 | 1.87 | 2.02 | 2.16 | 2.84 | 3.32 |
| index rq8 est c4 | 1.71 | 1.81 | 1.95 | 2.14 | 2.56 | 2.94 |
| index rq8 exact c4 | 1.91 | 1.86 | 2.05 | 2.21 | 2.60 | 3.06 |
| index tq default | 1.61 | 1.80 | 2.03 | 2.16 | 2.58 | 3.07 |
| index tq exact | 3.20 | 3.42 | 3.57 | 3.83 | 4.75 | 3.54 |

## Storage

| cell | ec_ivf index | ec_ivf per row | all indexes | table | total |
|---|---:|---:|---:|---:|---:|
| source f32 | 5.10 MiB | 538.2 B | 5.40 MiB | 158.80 MiB | 164.20 MiB |
| index f16 | 36.00 MiB | 3771.6 B | 36.20 MiB | 158.80 MiB | 195.00 MiB |
| index rq4 est c2 | 13.90 MiB | 1462.3 B | 14.20 MiB | 158.80 MiB | 173.00 MiB |
| index rq4 exact c2 | 13.90 MiB | 1462.3 B | 14.20 MiB | 158.80 MiB | 173.00 MiB |
| index rq4 est c3 | 13.90 MiB | 1462.3 B | 14.20 MiB | 158.80 MiB | 173.00 MiB |
| index rq4 exact c3 | 13.90 MiB | 1462.3 B | 14.20 MiB | 158.80 MiB | 173.00 MiB |
| index rq4 est c4 | 13.90 MiB | 1462.3 B | 14.20 MiB | 158.80 MiB | 173.00 MiB |
| index rq4 exact c4 | 13.90 MiB | 1462.3 B | 14.20 MiB | 158.80 MiB | 173.00 MiB |
| index rq8 est c2 | 21.30 MiB | 2238.1 B | 21.60 MiB | 158.80 MiB | 180.40 MiB |
| index rq8 exact c2 | 21.30 MiB | 2238.1 B | 21.60 MiB | 158.80 MiB | 180.40 MiB |
| index rq8 est c3 | 21.30 MiB | 2238.1 B | 21.60 MiB | 158.80 MiB | 180.40 MiB |
| index rq8 exact c3 | 21.30 MiB | 2238.1 B | 21.60 MiB | 158.80 MiB | 180.40 MiB |
| index rq8 est c4 | 21.30 MiB | 2238.1 B | 21.60 MiB | 158.80 MiB | 180.40 MiB |
| index rq8 exact c4 | 21.30 MiB | 2238.1 B | 21.60 MiB | 158.80 MiB | 180.40 MiB |
| index tq default | 13.90 MiB | 1458.2 B | 14.10 MiB | 158.80 MiB | 172.90 MiB |
| index tq exact | 13.90 MiB | 1458.2 B | 14.10 MiB | 158.80 MiB | 172.90 MiB |

## Immediate readouts

- `index f16` is the fastest high-recall 10k/w64 cell here: r@10 0.9990 at nprobe 64 with 2.57 ms mean warm latency; source f32 is r@10 1.0000 at 3.97 ms.
- Best `rq4` recall in this slice is `index rq4 est c3`: best r@10 0.9835 at nprobe 200, nprobe 64 mean 2.40 ms, ec_ivf index 13.90 MiB.
- Best `rq8` recall in this slice is `index rq8 est c4`: best r@10 0.9990 at nprobe 200, nprobe 64 mean 2.14 ms, ec_ivf index 21.30 MiB.
- TurboQuant exact-dequant did not improve recall in this slice: default and exact both best r@10 0.9815; exact nprobe 64 latency was 3.83 ms versus default 2.16 ms.
- `rq8 est c4` matched `index f16` recall at nprobe 64 and 200 in this 10k slice (0.9990/0.9990), with smaller ec_ivf index storage (21.30 MiB vs f16 36.00 MiB) and lower nprobe 64 warm latency (2.14 ms vs 2.57 ms).
- This is a corrected 10k smoke/sweep, not a 111h closeout. It does not replace the required 50k/100k/final scale sweeps or matched-recall comparisons at 0.97/0.99 on larger corpora.
