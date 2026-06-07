# Task 85 Review Request: Product-Scale Closeout

## Request

Review the Task 85 closeout decision.

Task 85 should close under the third exit criterion: no product-scale SPIRE
Pareto point is justified yet. The accepted standard is the user's repeated
bar: improve latency while retaining the current recall level. A lower-recall
row, a lower rerank row, or a row that only renames/raises the candidate budget
does not count as a latency win.

## Decision

Keep SPIRE as research/opt-in for 1M product claims. Do not change defaults,
do not add a product profile, and do not promote a new current benchmark lane
from Task 85.

The retained Task 79/81 SPIRE point remains the baseline to beat:

| Row | recall@10 | candidate_sum | heap_rerank_sum | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| retained block16/global1152 | 0.9832 | 9,213,846 | 12,500 | 246.397 ms | 304.476 ms | 321.342 ms |

Task 85 found no material same-recall latency win:

| Option | Verdict | recall@10 | candidate_sum | p50 | p95 | p99 | Notes |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| block8/global1152 | rejected | 0.9832 | 4,607,442 | 283.839 ms | 357.274 ms | 2729.302 ms | Same recall and fewer candidates, but slower. |
| per-leaf cap 12 | rejected | 0.7714 | 9,213,924 | 477.209 ms | 494.663 ms | 500.243 ms | Matched candidates but large recall loss and slower. |
| block32/global576 | rejected | 0.9730 | 9,206,722 | 178.624 ms | 235.037 ms | 250.139 ms | Matched row budget/candidates but lower recall. |
| block32/global1152 | rejected as product profile | 0.9876 | 18,413,851 | 235.691 ms | 295.157 ms | 308.841 ms | Same-suite recall match, but roughly doubles candidates for only a tiny latency gain. |
| block16/global1152 same-suite repeat | retained control | 0.9876 | 9,213,846 | 237.482 ms | 297.192 ms | 310.792 ms | Warm control from packet 006. |

The strongest measured Task 85 latency row is block32/global1152, but it is not
a defensible product point: it buys about 1.8 ms p50 and 2.0 ms p95 versus the
same-suite warm block16 repeat while doubling `candidate_sum`. The matched-row
budget block32 row is faster, but misses the recall bar.

## Comparator Context

Existing AWS comparator evidence makes the product bar even higher:

| Comparator | Evidence | Recall | p50 | p95 | p99 | Storage |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| ec IVF/RaBitQ nprobe128 | `benchmarks/task51-aws-ivf-rabitq-final-gate/` | 0.9864 | 34.6 ms | 41.5 ms | 48.0 ms | 298.0 MiB index |
| ec DiskANN L800 | `benchmarks/task59-aws-diskann-final-graviton-suite/` | 0.9825 | 19.7 ms | 30.9 ms | 35.6 ms | 455.1 MiB index |
| pgvectorscale DiskANN sl400 | `benchmarks/comparators-50k-100k-1m/` | 0.984 | 19.5 ms | 32.0 ms | n/a | external comparator |
| vchord RaBitQ default | `benchmarks/comparators-50k-100k-1m/` | 0.9995 | 90.3 ms | 100.0 ms | n/a | external comparator |

SPIRE still has useful research behavior, but none of the Task 85 options is
competitive enough to call product-ready at 1M.

## Strongest Accepted And Rejected Options

Accepted current SPIRE baseline:

- Retained block16/global1152 remains the best SPIRE baseline for the task.
- No Task 85 profile/default change is accepted.

Rejected options:

- Task 83/84 blanket caps and k-summary variants: recall recovery or latency
  movement came from candidate growth or missed the recall target.
- Task 85 block8 geometry: fewer candidates at same recall, but slower due
  worse read/summary behavior.
- Task 85 per-leaf caps: changed the selected candidate surface, collapsed
  recall, and increased latency.
- Task 85 block32 geometry: promising layout signal, but the only same-recall
  row is not material once candidate inflation is counted.

## Next Research Direction

The next SPIRE latency project should attack object-read and summary-scoring
cost directly while preserving the selected candidate set. Packet 005 and 006
funnel evidence points to read + summary CPU dominating warm latency:

- retained block16 repeat in packet 006: object read p50 183.712 ms, score p50
  56.872 ms;
- block32/global1152 in packet 006: object read p50 167.198 ms, score p50
  44.204 ms, but with 18.4M candidates.

Concrete follow-up: design a new SPIRE physical layout or read path that can
reduce object reads for the existing retained block16 candidate surface without
changing recall semantics. Examples include a hot/cold payload split, a V5
object layout with true partial segment reads, or a row/block locator that
avoids reading unneeded payload for selected blocks.

Operator warmup/caching can be measured separately, but it should not be used
as the product Pareto answer unless it preserves the cold/warm reporting split.

## Final AWS State

Final packet-local status:

- `target/debug/ecaz cloud status --profile 1m --database postgres` returned
  `state: unknown`, captured in
  `artifacts/cloud-status-final-paused-closeout.log`.
- Direct EC2 verification of the known 1M DB instance returned
  `State: stopped`, captured in
  `artifacts/aws-ec2-status-final-closeout.log`.

So the AWS `1m` database instance is not running at closeout.

## Validation

No new benchmark suite was run for this packet. This is a closeout packet over
the completed Task 85 AWS 1M/q500 suite evidence in packets 001, 004, 005, and
006 plus the cited comparator benchmark packets.
