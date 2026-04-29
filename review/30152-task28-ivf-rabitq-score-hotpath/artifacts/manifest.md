# Artifact Manifest: 30152 Task 28 IVF RaBitQ Score Hot Path

## `latency_rabitq_10k_n64_w750_p48_i10_hwm.log`

- head SHA: `91964193`
- packet/topic: `30152-task28-ivf-rabitq-score-hotpath`
- lane / fixture / storage format / rerank mode: A10 10k RaBitQ latency/HWM, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 10 --iterations 10 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/latency_rabitq_10k_n64_w750_p48_i10_hwm.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated existing 10k RaBitQ surface from packet 30144
- key result lines: `p50=344.2 ms`, `p95=401.3 ms`, `p99=413.1 ms`, `hwm_peak_kb=68212`

## `latency_rabitq_25k_n64_w750_p48_i10_hwm.log`

- head SHA: `91964193`
- packet/topic: `30152-task28-ivf-rabitq-score-hotpath`
- lane / fixture / storage format / rerank mode: A10 25k RaBitQ latency/HWM, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 10 --iterations 10 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/latency_rabitq_25k_n64_w750_p48_i10_hwm.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated existing 25k RaBitQ surface from packet 30144
- key result lines: `p50=775.7 ms`, `p95=835.6 ms`, `p99=858.8 ms`, `hwm_peak_kb=92996`

## `recall10_rabitq_10k_n64_w750_p48_q20.log`

- head SHA: `91964193`
- packet/topic: `30152-task28-ivf-rabitq-score-hotpath`
- lane / fixture / storage format / rerank mode: A10 10k RaBitQ recall@10, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 10 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/recall10_rabitq_10k_n64_w750_p48_q20.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated existing 10k RaBitQ surface from packet 30144
- key result lines: `recall@k=1.0000`, `ndcg@k=1.0000`, `mean q-time=316.08 ms`

## `recall100_rabitq_10k_n64_w750_p48_q20.log`

- head SHA: `91964193`
- packet/topic: `30152-task28-ivf-rabitq-score-hotpath`
- lane / fixture / storage format / rerank mode: A10 10k RaBitQ recall@100, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 100 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/recall100_rabitq_10k_n64_w750_p48_q20.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated existing 10k RaBitQ surface from packet 30144
- key result lines: `recall@k=0.9930`, `ndcg@k=0.9995`, `mean q-time=346.64 ms`

## `recall10_rabitq_25k_n64_w750_p48_q20.log`

- head SHA: `91964193`
- packet/topic: `30152-task28-ivf-rabitq-score-hotpath`
- lane / fixture / storage format / rerank mode: A10 25k RaBitQ recall@10, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 10 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/recall10_rabitq_25k_n64_w750_p48_q20.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated existing 25k RaBitQ surface from packet 30144
- key result lines: `recall@k=1.0000`, `ndcg@k=1.0000`, `mean q-time=737.47 ms`

## `recall100_rabitq_25k_n64_w750_p48_q20.log`

- head SHA: `91964193`
- packet/topic: `30152-task28-ivf-rabitq-score-hotpath`
- lane / fixture / storage format / rerank mode: A10 25k RaBitQ recall@100, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 100 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30152-task28-ivf-rabitq-score-hotpath/artifacts/recall100_rabitq_25k_n64_w750_p48_q20.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated existing 25k RaBitQ surface from packet 30144
- key result lines: `recall@k=0.9915`, `ndcg@k=0.9997`, `mean q-time=765.43 ms`
