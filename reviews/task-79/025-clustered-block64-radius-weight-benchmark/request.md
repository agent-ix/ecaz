# Review Request: Task 79 Clustered Block64 Radius-Weight Benchmark

## Scope

This packet benchmarks the packet 024 clustered leaf block implementation on the local PG18 Task 79 RaBitQ 100k fixture. It is a measurement packet, not a new code-change packet.

No AWS was used.

## Question

Does clustering rows inside each leaf make the existing V3 block summaries selective enough to reduce scored candidates while preserving RaBitQ recall?

Short answer: no. It improves candidate count and p50 at low global caps, but recall remains below the Task 79 floor. Recall only clears the floor when the scan returns to 9.5M candidates, which is far over budget.

## Evidence

Packet path: `reviews/task-79/025-clustered-block64-radius-weight-benchmark/`

Important artifacts:

- `suite-rabitq-clustered-block64-radius-weight.json`: checked-in `ecaz bench suite` config.
- `artifacts/manifest.md`: artifact metadata, commands, head SHA, backend SHA, gate readout.
- `artifacts/suite-run.log`: canonical suite run log.
- `artifacts/suite-status.log`: completed=15 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: suite report and parsed result stream.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block64-clustered.log`: clustered rebuild evidence.

## Setup

- Head SHA: `d5c15c6c314e17646c38aa9e9eac99617fc0fbfb`
- Local PG18 backend sha256: `5e9aec3491f7c07cde2be38f1476f65225b94207270afbfc98cbb78947e99f8d`
- Fixture: `task79_spire_candidate_surface`, `task79_surface_100k_corpus`, `task79_surface_100k_queries`
- Index: `task79_surface_100k_idx`
- Storage: RaBitQ
- Index shape: `nlists=128`, `recursive_fanout=8`, `nprobe=24`, `rerank_width=25`, `boundary_replica_count=0`, top graph search list size 96
- Scan shape: `nprobe=96`, `rerank_width=25`, 200 queries, recall@10
- Block layout: `ec_spire.leaf_block_rows=64`, clustered before V3 summary generation

Build timing from the clustered rebuild:

- `total_ms=14152`
- `draft_leaf_inputs_ms=4438`

This packet did not run a same-packet unclustered rebuild control; earlier block64 unclustered packets vary from 9.616s to 14.983s depending on run conditions, so this packet only claims the local clustered rebuild total.

## Gate

Task 79 requires:

- Recall@10 >= 0.9925.
- Candidate sum <= 5.2M over 200 queries.
- p50 <=45ms or at least 25% lower than the 63.515ms baseline, but latency alone is invalid.

Baseline in this packet:

| global_cap | radius_weight | candidates | p50 | recall@10 |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 0 | 15,506,227 | 63.515 ms | 0.9975 |

## Results

Best row under the candidate ceiling:

| global_cap | radius_weight | candidates | p50 | p95 | recall@10 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 416 | 0.25 | 5,197,973 | 45.796 ms | 57.740 ms | 0.9835 |

This fails recall by 0.0090 absolute.

Best recall row:

| global_cap | radius_weight | candidates | p50 | p95 | recall@10 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 768 | 0 | 9,525,502 | 56.486 ms | 68.121 ms | 0.9930 |

This clears recall but fails the candidate gate by 4.33M candidates and is not a latency win.

Representative rows:

| global_cap | radius_weight | candidates | p50 | recall@10 | decision |
| ---: | ---: | ---: | ---: | ---: | --- |
| 384 | 0 | 4,764,181 | 44.606 ms | 0.9690 | fails recall |
| 384 | 0.25 | 4,798,986 | 44.745 ms | 0.9760 | fails recall |
| 400 | 0.25 | 4,998,286 | 44.943 ms | 0.9810 | fails recall |
| 416 | 0.25 | 5,197,973 | 45.796 ms | 0.9835 | fails recall |
| 512 | 0.25 | 6,392,555 | 48.363 ms | 0.9870 | fails candidates and recall |
| 768 | 0 | 9,525,502 | 56.486 ms | 0.9930 | fails candidates and latency |

## Conclusion

Clustered block64 summaries are not a valid Task 79 fix. They directly reduce scored candidates at the target caps, but the block ranking still prunes true neighbors too aggressively. The result points away from further cap/weight sweeps and toward diagnosing missed winners: for each query, record the summary-score rank of blocks containing true top-10 neighbors, then either calibrate the block score or add a safer two-stage selector.
