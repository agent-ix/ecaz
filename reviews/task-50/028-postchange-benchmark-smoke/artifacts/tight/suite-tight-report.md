# Suite Report: task-50-postchange-smoke

- config: `reviews/task-50/028-postchange-benchmark-smoke/suite-tight.json`
- config_sha256: `7dd21296db4653b6d24ef9cd0835b2f41db9491a7bebdcb928dd0da32db614b0`
- dry_run: `false`
- steps: completed 16, failed 0, skipped 57, dry-run 0, missing artifacts 0, stale 0

| Step | Kind | Status | Duration ms | Artifacts |
| --- | --- | --- | ---: | --- |
| prepare-ec_real_25k | corpus-prepare | Skipped | - | `target/real-corpus/staged-task50/ec_real_25k_corpus.tsv`<br>`target/real-corpus/staged-task50/ec_real_25k_queries.tsv`<br>`target/real-corpus/staged-task50/ec_real_25k_manifest.json` |
| prepare-ec_real_100k | corpus-prepare | Skipped | - | `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`<br>`target/real-corpus/staged-task50/ec_real_100k_queries.tsv`<br>`target/real-corpus/staged-task50/ec_real_100k_manifest.json` |
| load-10k-ivfrabitq | load | Succeeded | 1171 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_10k-ivfrabitq.log` |
| recall-10k-ivfrabitq | recall | Succeeded | 28501 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_10k-ivfrabitq.log` |
| latency-10k-ivfrabitq | latency | Succeeded | 4302 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_10k-ivfrabitq.log` |
| storage-10k-ivfrabitq | storage | Succeeded | 24 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_10k-ivfrabitq.log` |
| load-10k-spirerabitq | load | Succeeded | 1098 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_10k-spirerabitq.log` |
| recall-10k-spirerabitq | recall | Succeeded | 160747 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_10k-spirerabitq.log` |
| latency-10k-spirerabitq | latency | Succeeded | 36692 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_10k-spirerabitq.log` |
| storage-10k-spirerabitq | storage | Succeeded | 23 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_10k-spirerabitq.log` |
| load-10k-hnsw | load | Succeeded | 1061 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_10k-hnsw.log` |
| recall-10k-hnsw | recall | Succeeded | 15697 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_10k-hnsw.log` |
| latency-10k-hnsw | latency | Succeeded | 955 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_10k-hnsw.log` |
| storage-10k-hnsw | storage | Succeeded | 21 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_10k-hnsw.log` |
| load-10k-diskann | load | Succeeded | 1091 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_10k-diskann.log` |
| recall-10k-diskann | recall | Succeeded | 21848 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_10k-diskann.log` |
| latency-10k-diskann | latency | Succeeded | 2493 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_10k-diskann.log` |
| storage-10k-diskann | storage | Succeeded | 22 | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_10k-diskann.log` |
| load-25k-ivfrabitq | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_25k-ivfrabitq.log` |
| recall-25k-ivfrabitq | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_25k-ivfrabitq.log` |
| latency-25k-ivfrabitq | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_25k-ivfrabitq.log` |
| storage-25k-ivfrabitq | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_25k-ivfrabitq.log` |
| load-25k-spirerabitq | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_25k-spirerabitq.log` |
| recall-25k-spirerabitq | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_25k-spirerabitq.log` |
| latency-25k-spirerabitq | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_25k-spirerabitq.log` |
| storage-25k-spirerabitq | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_25k-spirerabitq.log` |
| load-25k-hnsw | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_25k-hnsw.log` |
| recall-25k-hnsw | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_25k-hnsw.log` |
| latency-25k-hnsw | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_25k-hnsw.log` |
| storage-25k-hnsw | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_25k-hnsw.log` |
| load-25k-diskann | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_25k-diskann.log` |
| recall-25k-diskann | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_25k-diskann.log` |
| latency-25k-diskann | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_25k-diskann.log` |
| storage-25k-diskann | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_25k-diskann.log` |
| load-50k-ivfrabitq | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_50k-ivfrabitq.log` |
| recall-50k-ivfrabitq | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_50k-ivfrabitq.log` |
| latency-50k-ivfrabitq | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_50k-ivfrabitq.log` |
| storage-50k-ivfrabitq | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_50k-ivfrabitq.log` |
| load-50k-spirerabitq | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_50k-spirerabitq.log` |
| load-50k-hnsw | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_50k-hnsw.log` |
| recall-50k-hnsw | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_50k-hnsw.log` |
| latency-50k-hnsw | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_50k-hnsw.log` |
| storage-50k-hnsw | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_50k-hnsw.log` |
| load-50k-diskann | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_50k-diskann.log` |
| recall-50k-diskann | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_50k-diskann.log` |
| latency-50k-diskann | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_50k-diskann.log` |
| storage-50k-diskann | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_50k-diskann.log` |
| load-100k-ivfrabitq | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_100k-ivfrabitq.log` |
| recall-100k-ivfrabitq | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_100k-ivfrabitq.log` |
| latency-100k-ivfrabitq | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_100k-ivfrabitq.log` |
| storage-100k-ivfrabitq | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_100k-ivfrabitq.log` |
| load-100k-spirerabitq | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_100k-spirerabitq.log` |
| load-100k-hnsw | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_100k-hnsw.log` |
| recall-100k-hnsw | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_100k-hnsw.log` |
| latency-100k-hnsw | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_100k-hnsw.log` |
| storage-100k-hnsw | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_100k-hnsw.log` |
| load-100k-diskann | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_100k-diskann.log` |
| recall-100k-diskann | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_100k-diskann.log` |
| latency-100k-diskann | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_100k-diskann.log` |
| storage-100k-diskann | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_100k-diskann.log` |
| load-990k-ivfrabitq | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_ann_benchmarks_anchor-ivfrabitq.log` |
| recall-990k-ivfrabitq | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_ann_benchmarks_anchor-ivfrabitq.log` |
| latency-990k-ivfrabitq | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_ann_benchmarks_anchor-ivfrabitq.log` |
| storage-990k-ivfrabitq | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_ann_benchmarks_anchor-ivfrabitq.log` |
| load-990k-spirerabitq | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_ann_benchmarks_anchor-spirerabitq.log` |
| load-990k-diskann | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_ann_benchmarks_anchor-diskann.log` |
| recall-990k-diskann | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_ann_benchmarks_anchor-diskann.log` |
| latency-990k-diskann | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_ann_benchmarks_anchor-diskann.log` |
| storage-990k-diskann | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_ann_benchmarks_anchor-diskann.log` |
| load-990k-hnsw | load | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/corpus-load-ec_real_ann_benchmarks_anchor-hnsw.log` |
| recall-990k-hnsw | recall | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/recall-ec_real_ann_benchmarks_anchor-hnsw.log` |
| latency-990k-hnsw | latency | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/latency-ec_real_ann_benchmarks_anchor-hnsw.log` |
| storage-990k-hnsw | storage | Skipped | - | `reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/storage-ec_real_ann_benchmarks_anchor-hnsw.log` |

## Parsed Results

| Step | Kind | Metric | Values |
| --- | --- | --- | --- |
| load-10k-ivfrabitq | load | load_timing | `phase=total`, `seconds=1.170000`, `subject=ec_real_10k_ivfrabitq` |
| recall-10k-ivfrabitq | recall | recall | `mean q-time=4.55 ms`, `ndcg@k=0.9995`, `nprobe=8`, `queries=200`, `recall@k=0.9735`, `recall_ci95_high=0.9797`, `recall_ci95_low=0.9655`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-ivfrabitq | recall | recall | `mean q-time=8.24 ms`, `ndcg@k=0.9998`, `nprobe=16`, `queries=200`, `recall@k=0.9780`, `recall_ci95_high=0.9836`, `recall_ci95_low=0.9706`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-ivfrabitq | recall | recall | `mean q-time=10.68 ms`, `ndcg@k=0.9999`, `nprobe=24`, `queries=200`, `recall@k=0.9790`, `recall_ci95_high=0.9844`, `recall_ci95_low=0.9717`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-ivfrabitq | recall | recall | `mean q-time=13.48 ms`, `ndcg@k=0.9999`, `nprobe=32`, `queries=200`, `recall@k=0.9790`, `recall_ci95_high=0.9844`, `recall_ci95_low=0.9717`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-ivfrabitq | recall | recall | `mean q-time=19.63 ms`, `ndcg@k=0.9999`, `nprobe=48`, `queries=200`, `recall@k=0.9790`, `recall_ci95_high=0.9844`, `recall_ci95_low=0.9717`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-ivfrabitq | recall | recall | `mean q-time=25.54 ms`, `ndcg@k=0.9999`, `nprobe=64`, `queries=200`, `recall@k=0.9790`, `recall_ci95_high=0.9844`, `recall_ci95_low=0.9717`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| latency-10k-ivfrabitq | latency | latency | `count=50`, `max=6.37 ms`, `mean=4.09 ms`, `min=2.35 ms`, `nprobe=8`, `p50=3.98 ms`, `p95=5.22 ms`, `p99=5.87 ms`, `stddev=0.83 ms` |
| latency-10k-ivfrabitq | latency | latency | `count=50`, `max=11.1 ms`, `mean=7.32 ms`, `min=4.67 ms`, `nprobe=16`, `p50=7.39 ms`, `p95=9.13 ms`, `p99=10.4 ms`, `stddev=1.32 ms` |
| latency-10k-ivfrabitq | latency | latency | `count=50`, `max=18.5 ms`, `mean=10.5 ms`, `min=8.08 ms`, `nprobe=24`, `p50=10.6 ms`, `p95=12.3 ms`, `p99=17.0 ms`, `stddev=1.74 ms` |
| latency-10k-ivfrabitq | latency | latency | `count=50`, `max=21.7 ms`, `mean=13.3 ms`, `min=11.2 ms`, `nprobe=32`, `p50=13.2 ms`, `p95=16.2 ms`, `p99=20.9 ms`, `stddev=1.86 ms` |
| latency-10k-ivfrabitq | latency | latency | `count=50`, `max=26.0 ms`, `mean=19.2 ms`, `min=17.6 ms`, `nprobe=48`, `p50=19.0 ms`, `p95=21.3 ms`, `p99=24.0 ms`, `stddev=1.29 ms` |
| latency-10k-ivfrabitq | latency | latency | `count=50`, `max=39.6 ms`, `mean=26.0 ms`, `min=23.2 ms`, `nprobe=64`, `p50=24.9 ms`, `p95=35.1 ms`, `p99=39.4 ms`, `stddev=3.58 ms` |
| storage-10k-ivfrabitq | storage | storage_field | `field=prefix`, `value=ec_real_10k_ivfrabitq` |
| storage-10k-ivfrabitq | storage | storage_field | `field=corpus`, `value=ec_real_10k_ivfrabitq_corpus` |
| storage-10k-ivfrabitq | storage | storage_field | `field=rows`, `value=10000` |
| storage-10k-ivfrabitq | storage | storage_field | `field=heap`, `value=1.3 MiB` |
| storage-10k-ivfrabitq | storage | storage_field | `field=table (heap + toast + fsm/vm)`, `value=159.4 MiB` |
| storage-10k-ivfrabitq | storage | storage_field | `field=indexes`, `value=10.3 MiB` |
| storage-10k-ivfrabitq | storage | storage_field | `field=total`, `value=169.7 MiB` |
| storage-10k-ivfrabitq | storage | storage_field | `field=per row (total)`, `value=17789.7 B` |
| storage-10k-ivfrabitq | storage | storage_field | `field=per row (heap only)`, `value=136.8 B` |
| storage-10k-ivfrabitq | storage | storage_index | `access method=ec_ivf`, `index=ec_real_10k_ivfrabitq_rabitq_idx`, `per row=1028.1 B`, `profile=ec_ivf`, `reloptions={storage_format=rabitq}`, `size=9.8 MiB` |
| storage-10k-ivfrabitq | storage | storage_index | `access method=btree`, `index=ec_real_10k_ivfrabitq_corpus_pkey`, `per row=46.7 B`, `profile=<unknown>`, `reloptions={}`, `size=456.0 KiB` |
| load-10k-spirerabitq | load | load_timing | `phase=total`, `seconds=1.090000`, `subject=ec_real_10k_spirerabitq` |
| recall-10k-spirerabitq | recall | recall | `mean q-time=36.02 ms`, `ndcg@k=0.9996`, `nprobe=8`, `queries=200`, `recall@k=0.9920`, `recall_ci95_high=0.9951`, `recall_ci95_low=0.9870`, `recall_p10=1.0000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-spirerabitq | recall | recall | `mean q-time=66.44 ms`, `ndcg@k=0.9999`, `nprobe=16`, `queries=200`, `recall@k=0.9985`, `recall_ci95_high=0.9995`, `recall_ci95_low=0.9956`, `recall_p10=1.0000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-spirerabitq | recall | recall | `mean q-time=95.66 ms`, `ndcg@k=1.0000`, `nprobe=24`, `queries=200`, `recall@k=1.0000`, `recall_ci95_high=1.0000`, `recall_ci95_low=0.9981`, `recall_p10=1.0000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-spirerabitq | recall | recall | `mean q-time=122.90 ms`, `ndcg@k=1.0000`, `nprobe=32`, `queries=200`, `recall@k=1.0000`, `recall_ci95_high=1.0000`, `recall_ci95_low=0.9981`, `recall_p10=1.0000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-spirerabitq | recall | recall | `mean q-time=182.38 ms`, `ndcg@k=1.0000`, `nprobe=48`, `queries=200`, `recall@k=1.0000`, `recall_ci95_high=1.0000`, `recall_ci95_low=0.9981`, `recall_p10=1.0000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-spirerabitq | recall | recall | `mean q-time=240.15 ms`, `ndcg@k=1.0000`, `nprobe=64`, `queries=200`, `recall@k=1.0000`, `recall_ci95_high=1.0000`, `recall_ci95_low=0.9981`, `recall_p10=1.0000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| latency-10k-spirerabitq | latency | latency | `count=50`, `max=54.2 ms`, `mean=32.9 ms`, `min=16.3 ms`, `nprobe=8`, `p50=31.7 ms`, `p95=44.2 ms`, `p99=49.8 ms`, `stddev=7.80 ms` |
| latency-10k-spirerabitq | latency | latency | `count=50`, `max=81.6 ms`, `mean=61.6 ms`, `min=37.3 ms`, `nprobe=16`, `p50=63.1 ms`, `p95=78.2 ms`, `p99=80.8 ms`, `stddev=11.8 ms` |
| latency-10k-spirerabitq | latency | latency | `count=50`, `max=110.2 ms`, `mean=91.9 ms`, `min=70.2 ms`, `nprobe=24`, `p50=95.3 ms`, `p95=102.6 ms`, `p99=107.8 ms`, `stddev=9.90 ms` |
| latency-10k-spirerabitq | latency | latency | `count=50`, `max=142.6 ms`, `mean=120.7 ms`, `min=101.9 ms`, `nprobe=32`, `p50=120.7 ms`, `p95=136.6 ms`, `p99=142.2 ms`, `stddev=9.50 ms` |
| latency-10k-spirerabitq | latency | latency | `count=50`, `max=206.1 ms`, `mean=179.5 ms`, `min=166.0 ms`, `nprobe=48`, `p50=179.1 ms`, `p95=190.4 ms`, `p99=200.2 ms`, `stddev=7.40 ms` |
| latency-10k-spirerabitq | latency | latency | `count=50`, `max=279.6 ms`, `mean=241.7 ms`, `min=227.7 ms`, `nprobe=64`, `p50=240.1 ms`, `p95=258.4 ms`, `p99=276.7 ms`, `stddev=9.77 ms` |
| storage-10k-spirerabitq | storage | storage_field | `field=prefix`, `value=ec_real_10k_spirerabitq` |
| storage-10k-spirerabitq | storage | storage_field | `field=corpus`, `value=ec_real_10k_spirerabitq_corpus` |
| storage-10k-spirerabitq | storage | storage_field | `field=rows`, `value=10000` |
| storage-10k-spirerabitq | storage | storage_field | `field=heap`, `value=1.3 MiB` |
| storage-10k-spirerabitq | storage | storage_field | `field=table (heap + toast + fsm/vm)`, `value=159.4 MiB` |
| storage-10k-spirerabitq | storage | storage_field | `field=indexes`, `value=9.4 MiB` |
| storage-10k-spirerabitq | storage | storage_field | `field=total`, `value=168.8 MiB` |
| storage-10k-spirerabitq | storage | storage_field | `field=per row (total)`, `value=17701.3 B` |
| storage-10k-spirerabitq | storage | storage_field | `field=per row (heap only)`, `value=136.8 B` |
| storage-10k-spirerabitq | storage | storage_index | `access method=ec_spire`, `index=ec_real_10k_spirerabitq_rabitq_idx`, `per row=939.6 B`, `profile=ec_spire`, `reloptions={storage_format=rabitq}`, `size=9.0 MiB` |
| storage-10k-spirerabitq | storage | storage_index | `access method=btree`, `index=ec_real_10k_spirerabitq_corpus_pkey`, `per row=46.7 B`, `profile=<unknown>`, `reloptions={}`, `size=456.0 KiB` |
| load-10k-hnsw | load | load_timing | `phase=total`, `seconds=1.060000`, `subject=ec_real_10k_hnsw` |
| recall-10k-hnsw | recall | recall | `ef_search=40`, `mean q-time=2.80 ms`, `ndcg@k=0.9616`, `queries=200`, `recall@k=0.8845`, `recall_ci95_high=0.8978`, `recall_ci95_low=0.8697`, `recall_p10=0.7000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-hnsw | recall | recall | `ef_search=80`, `mean q-time=2.47 ms`, `ndcg@k=0.9819`, `queries=200`, `recall@k=0.9300`, `recall_ci95_high=0.9404`, `recall_ci95_low=0.9180`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-hnsw | recall | recall | `ef_search=120`, `mean q-time=2.82 ms`, `ndcg@k=0.9864`, `queries=200`, `recall@k=0.9385`, `recall_ci95_high=0.9482`, `recall_ci95_low=0.9271`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-hnsw | recall | recall | `ef_search=200`, `mean q-time=3.00 ms`, `ndcg@k=0.9931`, `queries=200`, `recall@k=0.9545`, `recall_ci95_high=0.9628`, `recall_ci95_low=0.9445`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-hnsw | recall | recall | `ef_search=400`, `mean q-time=4.54 ms`, `ndcg@k=0.9994`, `queries=200`, `recall@k=0.9720`, `recall_ci95_high=0.9784`, `recall_ci95_low=0.9638`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| latency-10k-hnsw | latency | latency | `count=50`, `ef_search=40`, `max=6.57 ms`, `mean=1.74 ms`, `min=1.33 ms`, `p50=1.60 ms`, `p95=2.13 ms`, `p99=4.45 ms`, `stddev=0.72 ms` |
| latency-10k-hnsw | latency | latency | `count=50`, `ef_search=80`, `max=6.91 ms`, `mean=2.46 ms`, `min=1.98 ms`, `p50=2.33 ms`, `p95=2.90 ms`, `p99=5.22 ms`, `stddev=0.69 ms` |
| latency-10k-hnsw | latency | latency | `count=50`, `ef_search=120`, `max=6.58 ms`, `mean=2.26 ms`, `min=1.79 ms`, `p50=2.02 ms`, `p95=3.18 ms`, `p99=4.99 ms`, `stddev=0.72 ms` |
| latency-10k-hnsw | latency | latency | `count=50`, `ef_search=200`, `max=7.71 ms`, `mean=2.85 ms`, `min=2.31 ms`, `p50=2.60 ms`, `p95=3.74 ms`, `p99=6.11 ms`, `stddev=0.80 ms` |
| latency-10k-hnsw | latency | latency | `count=50`, `ef_search=400`, `max=10.1 ms`, `mean=4.32 ms`, `min=3.71 ms`, `p50=4.12 ms`, `p95=4.87 ms`, `p99=8.15 ms`, `stddev=0.91 ms` |
| storage-10k-hnsw | storage | storage_field | `field=prefix`, `value=ec_real_10k_hnsw` |
| storage-10k-hnsw | storage | storage_field | `field=corpus`, `value=ec_real_10k_hnsw_corpus` |
| storage-10k-hnsw | storage | storage_field | `field=rows`, `value=10000` |
| storage-10k-hnsw | storage | storage_field | `field=heap`, `value=1.3 MiB` |
| storage-10k-hnsw | storage | storage_field | `field=table (heap + toast + fsm/vm)`, `value=159.4 MiB` |
| storage-10k-hnsw | storage | storage_field | `field=indexes`, `value=25.3 MiB` |
| storage-10k-hnsw | storage | storage_field | `field=total`, `value=184.6 MiB` |
| storage-10k-hnsw | storage | storage_field | `field=per row (total)`, `value=19361.0 B` |
| storage-10k-hnsw | storage | storage_field | `field=per row (heap only)`, `value=136.8 B` |
| storage-10k-hnsw | storage | storage_index | `access method=ec_hnsw`, `index=ec_real_10k_hnsw_m16_idx`, `per row=1366.4 B`, `profile=ec_hnsw`, `reloptions={m=16,ef_construction=128,build_source_column=source}`, `size=13.0 MiB` |
| storage-10k-hnsw | storage | storage_index | `access method=ec_hnsw`, `index=ec_real_10k_hnsw_m8_idx`, `per row=1235.4 B`, `profile=ec_hnsw`, `reloptions={m=8,ef_construction=128,build_source_column=source}`, `size=11.8 MiB` |
| storage-10k-hnsw | storage | storage_index | `access method=btree`, `index=ec_real_10k_hnsw_corpus_pkey`, `per row=46.7 B`, `profile=<unknown>`, `reloptions={}`, `size=456.0 KiB` |
| load-10k-diskann | load | load_timing | `phase=total`, `seconds=1.090000`, `subject=ec_real_10k_diskann` |
| recall-10k-diskann | recall | recall | `list_size=64`, `mean q-time=9.61 ms`, `ndcg@k=0.9999`, `queries=200`, `recall@k=0.9965`, `recall_ci95_high=0.9983`, `recall_ci95_low=0.9928`, `recall_p10=1.0000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-diskann | recall | recall | `list_size=128`, `mean q-time=8.50 ms`, `ndcg@k=0.9999`, `queries=200`, `recall@k=0.9965`, `recall_ci95_high=0.9983`, `recall_ci95_low=0.9928`, `recall_p10=1.0000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-diskann | recall | recall | `list_size=200`, `mean q-time=9.07 ms`, `ndcg@k=0.9999`, `queries=200`, `recall@k=0.9970`, `recall_ci95_high=0.9986`, `recall_ci95_low=0.9935`, `recall_p10=1.0000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-diskann | recall | recall | `list_size=400`, `mean q-time=9.43 ms`, `ndcg@k=0.9999`, `queries=200`, `recall@k=0.9970`, `recall_ci95_high=0.9986`, `recall_ci95_low=0.9935`, `recall_p10=1.0000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| recall-10k-diskann | recall | recall | `list_size=800`, `mean q-time=9.91 ms`, `ndcg@k=0.9999`, `queries=200`, `recall@k=0.9975`, `recall_ci95_high=0.9989`, `recall_ci95_low=0.9942`, `recall_p10=1.0000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=2000` |
| latency-10k-diskann | latency | latency | `count=50`, `list_size=64`, `max=12.8 ms`, `mean=8.12 ms`, `min=7.51 ms`, `p50=7.97 ms`, `p95=9.12 ms`, `p99=11.3 ms`, `stddev=0.79 ms` |
| latency-10k-diskann | latency | latency | `count=50`, `list_size=128`, `max=12.5 ms`, `mean=8.53 ms`, `min=7.38 ms`, `p50=8.34 ms`, `p95=9.77 ms`, `p99=12.4 ms`, `stddev=0.92 ms` |
| latency-10k-diskann | latency | latency | `count=50`, `list_size=200`, `max=12.1 ms`, `mean=8.55 ms`, `min=7.76 ms`, `p50=8.37 ms`, `p95=9.55 ms`, `p99=11.1 ms`, `stddev=0.67 ms` |
| latency-10k-diskann | latency | latency | `count=50`, `list_size=400`, `max=13.4 ms`, `mean=9.14 ms`, `min=8.13 ms`, `p50=8.87 ms`, `p95=11.7 ms`, `p99=13.4 ms`, `stddev=1.14 ms` |
| latency-10k-diskann | latency | latency | `count=50`, `list_size=800`, `max=14.7 ms`, `mean=10.1 ms`, `min=8.91 ms`, `p50=9.82 ms`, `p95=12.4 ms`, `p99=14.2 ms`, `stddev=1.10 ms` |
| storage-10k-diskann | storage | storage_field | `field=prefix`, `value=ec_real_10k_diskann` |
| storage-10k-diskann | storage | storage_field | `field=corpus`, `value=ec_real_10k_diskann_corpus` |
| storage-10k-diskann | storage | storage_field | `field=rows`, `value=10000` |
| storage-10k-diskann | storage | storage_field | `field=heap`, `value=1.3 MiB` |
| storage-10k-diskann | storage | storage_field | `field=table (heap + toast + fsm/vm)`, `value=159.4 MiB` |
| storage-10k-diskann | storage | storage_field | `field=indexes`, `value=5.2 MiB` |
| storage-10k-diskann | storage | storage_field | `field=total`, `value=164.5 MiB` |
| storage-10k-diskann | storage | storage_field | `field=per row (total)`, `value=17253.2 B` |
| storage-10k-diskann | storage | storage_field | `field=per row (heap only)`, `value=136.8 B` |
| storage-10k-diskann | storage | storage_index | `access method=ec_diskann`, `index=ec_real_10k_diskann_idx`, `per row=494.0 B`, `profile=ec_diskann`, `reloptions={}`, `size=4.7 MiB` |
| storage-10k-diskann | storage | storage_index | `access method=btree`, `index=ec_real_10k_diskann_corpus_pkey`, `per row=46.7 B`, `profile=<unknown>`, `reloptions={}`, `size=456.0 KiB` |
wrote reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/results-report.jsonl
