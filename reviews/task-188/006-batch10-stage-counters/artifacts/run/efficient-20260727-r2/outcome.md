# Task 188 efficient batch-10 stage-counter rerun

- Status: succeeded
- Date: 2026-07-27
- Code checkpoint: `193cff682`
- Extension provenance: `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7`, release
- Suite config: `../../task188-batch10-stage-counters-efficient-suite.json`
- Suite config SHA-256: `d4c18956f06f139707cff700bbb4afdff7027050614dee3df78efe29445bb8af`
- Run: `efficient-20260727-r2`

The suite used PG18, three physical nodes, the staged `ec_real_100k` corpus,
`head_index_cap=16384`, `head_policy=training_landmarks`, 10 warmups, and 50
timed queries per arm. It ran `stage_counter_only=true`, so it skipped the
duplicate single-index build, recall matrix, and seed-coverage diagnostic. The
latency workers used `worker_batch_size=5`, reconnecting after every five timed
queries while merging the counters and latency samples.

The physical serving gate passed for 100,000 source rows and zero orphans. Both
physical arms emitted 37 stage rows and 28 materialization-work rows, and both
traversal reconciliations passed. The shared latency worker now re-warms each
fresh backend before timing; this run predates that fix and used five-query
reconnect batches.

| arm | p50 ms | mean/p95/p99 |
| --- | ---: | ---: | ---: | ---: |
| BW4 control | 28.10 | 239.20 / 1213.00 / 1230.10 (harness-affected) |
| BW8 candidate | 25.80 | 234.70 / 1191.40 / 1208.10 (harness-affected) |

Only p50 is cited from this run: candidate minus control is `-2.30 ms`.
The mean/p95/p99 values include cold first queries after each reconnect, so
their candidate-minus-control deltas are not latency evidence. The p50 values
track packet 005's warm rows (28.80 / 26.50 ms), which is consistent with the
warm path being intact.

The stage counters are also not a latency distribution: `custom_scan_total`
`233.338562 -> 228.812245 ms` follows the reconnect-contaminated mean and is
not a warm per-scan cost. The useful direct attribution is
`remote_expand` `11.691947 -> 9.218777 ms` and `traversal_total`
`13.439234 -> 11.034471 ms`. The merged work counters reported traversal hop
rounds `9.72 -> 5.58` per scan. More importantly, remote candidates moved from
`25.86 / 29.56` per scan in the packet-002 eager-0 control/candidate rows to
`6.64 / 6.62` in this explicit batch-10 rerun. The approximately four-fold
change is a batching/deduplication effect: ranked remote candidates are
deduplicated and materialized as one batch rather than counted once per
eager request; the counter's units did not change. This removes BW8's earlier
remote-work penalty and brings it to parity with BW4. Recall remains sourced
from packet 005 because this diagnostic intentionally omitted recall.

Durable evidence is in `suite-manifest.json`, `results.jsonl`,
`distann-multinode-summary.log`, and the two arm latency logs in this directory.
