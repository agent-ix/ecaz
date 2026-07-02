# Task 131 Phase 3 Increment A A/B Result Summary

- Head SHA: `b277cd9f6b90c6446b01b8e303d6948d0e28a451`
- Run command: `target/debug/ecaz bench suite run --config reviews/task-131/027-phase3-increment-a-ab/artifacts/task131-phase3-increment-a-ab-suite.json`
- Runner: `ecaz bench suite`
- Matrix: local multi-instance PG18 SPIRE, `rabitq`, `n128/b4`, `nprobe=96`, `k=10`
- Summaries: `ec_spire.leaf_block_rows=64`
- Variants:
  - `threshold-off`: `ec_spire.remote_search_initial_threshold_early_stop=off`
  - `threshold-on`: `ec_spire.remote_search_initial_threshold_early_stop=on`

## Identity Checks

Commands:

- `cmp -s artifacts/10k-n128-b4/bench-suite/production-read-k10-threshold-off-default-identity.jsonl artifacts/10k-n128-b4/bench-suite/production-read-k10-threshold-on-default-identity.jsonl`
- `cmp -s artifacts/50k-n128-b4/bench-suite/production-read-k10-threshold-off-default-identity.jsonl artifacts/50k-n128-b4/bench-suite/production-read-k10-threshold-on-default-identity.jsonl`

Results:

- 10k identity files: 200 rows each, byte-identical.
- 50k identity files: 1000 rows each, byte-identical.
- Caveat: byte-identical means the threshold gate did not change returned IDs.
  It does not mean each returned top-10 is k-distinct. The identity artifacts
  show duplicate corpus IDs in both arms; see
  `plan/tasks/132-spire-distributed-result-deduplication.md`.

## Latency and Recall

| Scale | Variant | Queries | Recall | Latency p50 | Latency p95 | Profile total p50 | Profile total p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | threshold-off | 200 | 0.9985 | 609.243 ms | 686.941 ms | 570.000 ms | 655.000 ms |
| 10k | threshold-on | 200 | 0.9985 | 613.294 ms | 728.343 ms | 576.000 ms | 653.000 ms |
| 50k | threshold-off | 1000 | 1.0000 | 2645.864 ms | 3287.777 ms | 2605.000 ms | 3090.000 ms |
| 50k | threshold-on | 1000 | 1.0000 | 2659.226 ms | 3191.039 ms | 2620.000 ms | 3214.000 ms |

Recall values above are matched under the current duplicate-tolerant metric.
They should not be read as proof that each query returned 10 distinct corpus
rows.

## Duplicate-ID Caveat

The identity files surfaced a distributed result-quality defect that is shared
by `threshold-off` and `threshold-on`:

- 10k threshold-off: 183/200 queries contain duplicate IDs in top-10; worst
  case has 4 distinct IDs.
- 50k threshold-off: 1000/1000 queries contain duplicate IDs in top-10; worst
  case has 4 distinct IDs.

This does not change the A/B conclusion because the duplicate behavior is
identical between arms, but it qualifies all recall language in this packet.

## Actual Scan Work Avoided

Actual scan profile rows report `leaf_block_skipped_sum=0` for every remote node
at both scales and both variants.

| Scale | Variant | Node | Leaf blocks selected | Leaf blocks skipped |
| --- | --- | ---: | ---: | ---: |
| 10k | threshold-off | 2 | 39024 | 0 |
| 10k | threshold-off | 3 | 46491 | 0 |
| 10k | threshold-off | 4 | 40743 | 0 |
| 10k | threshold-on | 2 | 39024 | 0 |
| 10k | threshold-on | 3 | 46491 | 0 |
| 10k | threshold-on | 4 | 40743 | 0 |
| 50k | threshold-off | 2 | 846828 | 0 |
| 50k | threshold-off | 3 | 1110570 | 0 |
| 50k | threshold-off | 4 | 1048928 | 0 |
| 50k | threshold-on | 2 | 846828 | 0 |
| 50k | threshold-on | 3 | 1110570 | 0 |
| 50k | threshold-on | 4 | 1048928 | 0 |

## Diagnostic Threshold Profile

The threshold-profile diagnostic rows still show potential skipped blocks/rows,
but those values are identical for `threshold-off` and `threshold-on`; they did
not translate into production scan avoidance.

| Scale | Variant | Node | Threshold blocks skipped | Threshold rows skipped |
| --- | --- | ---: | ---: | ---: |
| 10k | threshold-off | 2 | 2153 | 118205 |
| 10k | threshold-off | 3 | 2161 | 107400 |
| 10k | threshold-off | 4 | 2617 | 135894 |
| 10k | threshold-on | 2 | 2153 | 118205 |
| 10k | threshold-on | 3 | 2161 | 107400 |
| 10k | threshold-on | 4 | 2617 | 135894 |
| 50k | threshold-off | 2 | 1128 | 4021 |
| 50k | threshold-off | 3 | 2871 | 8980 |
| 50k | threshold-off | 4 | 3019 | 5930 |
| 50k | threshold-on | 2 | 1128 | 4021 |
| 50k | threshold-on | 3 | 2871 | 8980 |
| 50k | threshold-on | 4 | 3019 | 5930 |

## Decision

Shelve/reject this initial-threshold worker early-stop path. It preserves
returned IDs and recall, but it does not skip production scan work and does not
deliver a matched-recall latency win at both 10k and 50k.
