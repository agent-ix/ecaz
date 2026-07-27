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
traversal reconciliations passed.

| arm | mean ms | p50 ms | p95 ms | p99 ms |
| --- | ---: | ---: | ---: | ---: |
| BW4 control | 239.20 | 28.10 | 1213.00 | 1230.10 |
| BW8 candidate | 234.70 | 25.80 | 1191.40 | 1208.10 |

Candidate minus control: mean `-4.50 ms`, p50 `-2.30 ms`, p95 `-21.60 ms`,
p99 `-22.00 ms`.

Direct attribution also moved in the candidate arm: `custom_scan_total`
`233.338562 -> 228.812245 ms`, `remote_expand` `11.691947 -> 9.218777 ms`,
and `traversal_total` `13.439234 -> 11.034471 ms`. The merged work counters
reported traversal hop rounds `9.72 -> 5.58` per scan, while remote candidates
were effectively unchanged at `6.64 -> 6.62` per scan. This directly confirms
the stage-level mechanism for this fresh batch-10 run; recall remains sourced
from packet 005 because this diagnostic intentionally omitted recall.

Durable evidence is in `suite-manifest.json`, `results.jsonl`,
`distann-multinode-summary.log`, and the two arm latency logs in this directory.
