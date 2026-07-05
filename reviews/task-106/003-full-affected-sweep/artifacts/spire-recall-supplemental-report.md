# Suite Report: task106-spire-recall-supplemental

- config: `reviews/task-106/003-full-affected-sweep/task106-spire-recall-supplemental.json`
- config_sha256: `641866bcd417b142ea16606d098015e62676e45ec735d9ef1ba41a0a19eeaace`
- dry_run: `false`
- steps: completed 2, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0

| Step | Kind | Status | Duration ms | Artifacts |
| --- | --- | --- | ---: | --- |
| recall-1m-spire-rabitq-batch-on | recall | Succeeded | 74889 | `reviews/task-106/003-full-affected-sweep/artifacts/suite-spire-recall-supplemental/recall-1m-spire-rabitq-batch-on.log` |
| recall-1m-spire-rabitq-batch-off | recall | Succeeded | 58827 | `reviews/task-106/003-full-affected-sweep/artifacts/suite-spire-recall-supplemental/recall-1m-spire-rabitq-batch-off.log` |

## Parsed Results

| Step | Kind | Metric | Values |
| --- | --- | --- | --- |
| recall-1m-spire-rabitq-batch-on | recall | recall | `mean q-time=58.49 ms`, `ndcg@k=0.9959`, `nprobe=16`, `prefix=t106_1m_spire_rabitq`, `profile=ec_spire`, `queries=100`, `recall@k=0.9540`, `recall_ci95_high=0.9653`, `recall_ci95_low=0.9392`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=1000`, `recall_worst=0.2000`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| recall-1m-spire-rabitq-batch-on | recall | recall | `mean q-time=107.13 ms`, `ndcg@k=0.9977`, `nprobe=24`, `prefix=t106_1m_spire_rabitq`, `profile=ec_spire`, `queries=100`, `recall@k=0.9700`, `recall_ci95_high=0.9789`, `recall_ci95_low=0.9575`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=1000`, `recall_worst=0.4000`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| recall-1m-spire-rabitq-batch-on | recall | recall | `mean q-time=107.68 ms`, `ndcg@k=0.9982`, `nprobe=32`, `prefix=t106_1m_spire_rabitq`, `profile=ec_spire`, `queries=100`, `recall@k=0.9760`, `recall_ci95_high=0.9838`, `recall_ci95_low=0.9645`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=1000`, `recall_worst=0.5000`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| recall-1m-spire-rabitq-batch-on | recall | recall | `mean q-time=154.70 ms`, `ndcg@k=0.9986`, `nprobe=48`, `prefix=t106_1m_spire_rabitq`, `profile=ec_spire`, `queries=100`, `recall@k=0.9800`, `recall_ci95_high=0.9870`, `recall_ci95_low=0.9693`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=1000`, `recall_worst=0.5000`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| recall-1m-spire-rabitq-batch-off | recall | recall | `mean q-time=57.77 ms`, `ndcg@k=0.9959`, `nprobe=16`, `prefix=t106_1m_spire_rabitq`, `profile=ec_spire`, `queries=100`, `recall@k=0.9540`, `recall_ci95_high=0.9653`, `recall_ci95_low=0.9392`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=1000`, `recall_worst=0.2000`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| recall-1m-spire-rabitq-batch-off | recall | recall | `mean q-time=65.88 ms`, `ndcg@k=0.9977`, `nprobe=24`, `prefix=t106_1m_spire_rabitq`, `profile=ec_spire`, `queries=100`, `recall@k=0.9700`, `recall_ci95_high=0.9789`, `recall_ci95_low=0.9575`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=1000`, `recall_worst=0.4000`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| recall-1m-spire-rabitq-batch-off | recall | recall | `mean q-time=75.03 ms`, `ndcg@k=0.9982`, `nprobe=32`, `prefix=t106_1m_spire_rabitq`, `profile=ec_spire`, `queries=100`, `recall@k=0.9760`, `recall_ci95_high=0.9838`, `recall_ci95_low=0.9645`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=1000`, `recall_worst=0.5000`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| recall-1m-spire-rabitq-batch-off | recall | recall | `mean q-time=111.29 ms`, `ndcg@k=0.9986`, `nprobe=48`, `prefix=t106_1m_spire_rabitq`, `profile=ec_spire`, `queries=100`, `recall@k=0.9800`, `recall_ci95_high=0.9870`, `recall_ci95_low=0.9693`, `recall_p10=0.9000`, `recall_p50=1.0000`, `recall_p90=1.0000`, `recall_trials=1000`, `recall_worst=0.5000`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
wrote reviews/task-106/003-full-affected-sweep/artifacts/spire-recall-supplemental-report-results.jsonl
