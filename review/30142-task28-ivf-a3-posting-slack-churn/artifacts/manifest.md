# Artifact Manifest: 30142 Task 28 IVF A3 Posting-Slack Churn

## `ivf_a3_100k_rotating_slack50.log`

- head SHA: `419a0713`
- packet/topic: `30142-task28-ivf-a3-posting-slack-churn`
- lane / fixture / storage format / rerank mode: IVF A3 rotating-window churn, synthetic 100k 4D, `quantizer = 'turboquant'`, `rerank = 'heap_f32'`, `posting_slack_percent = 50`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 stress ivf-vacuum-scale --table-prefix task28_ivf_a3_100k_slack50 --rows 100000 --nlists 32,64 --nprobe 8 --training-sample-rows 10000 --dimensions 4 --vector-period 100000 --quantizer turboquant --posting-slack-percent 50 --cycles 10 --churn-rows 25000 --refill-after-vacuum --sample-interval-ms 25 --log-output review/30142-task28-ivf-a3-posting-slack-churn/artifacts/ivf_a3_100k_rotating_slack50.log`
- timestamp: 2026-04-28 local
- isolated/shared surface: isolated one-index-per-table surfaces
- key result lines:
  - n32 cycle 1 `idx_before=13623296`, cycle 10 `idx_after_refill=13623296`
  - n64 cycle 1 `idx_before=13901824`, cycle 10 `idx_after_refill=13901824`
  - max n32 HWM `82920 kB`; max n64 HWM `106432 kB`

## `page_ownership_slack50.sql`

- head SHA: `419a0713`
- packet/topic: `30142-task28-ivf-a3-posting-slack-churn`
- lane / fixture / storage format / rerank mode: SQL diagnostic for the slack-50 n32/n64 final indexes
- command: packet-local SQL input for `page_ownership_slack50.log`
- timestamp: 2026-04-28 local
- isolated/shared surface: isolated one-index-per-table surfaces
- key result lines: source SQL only

## `page_ownership_slack50.log`

- head SHA: `419a0713`
- packet/topic: `30142-task28-ivf-a3-posting-slack-churn`
- lane / fixture / storage format / rerank mode: final page ownership after slack-50 rotating-window churn
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30142-task28-ivf-a3-posting-slack-churn/artifacts/page_ownership_slack50.sql --raw --log-output review/30142-task28-ivf-a3-posting-slack-churn/artifacts/page_ownership_slack50.log`
- timestamp: 2026-04-28 local
- isolated/shared surface: isolated one-index-per-table surfaces
- key result lines:
  - n32: `1057 posting_blocks`, `100000 posting_tuples`, `100000 heap_tid_refs`, `0 deleted_postings`, `0 cross_list_blocks`, `0 mixed_blocks`
  - n64: `1096 posting_blocks`, `100000 posting_tuples`, `100000 heap_tid_refs`, `0 deleted_postings`, `0 cross_list_blocks`, `0 mixed_blocks`
