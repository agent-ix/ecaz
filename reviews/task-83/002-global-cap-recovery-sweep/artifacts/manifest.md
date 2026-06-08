# Task 83 Global-Cap Recovery Sweep Manifest

- Task: `plan/tasks/83-spire-selected-block-containment-recovery.md`
- Packet: `reviews/task-83/002-global-cap-recovery-sweep/`
- Head SHA: `77cafdacd4361e7fb97f3d2e902f7aaa18c3d809`
- Suite config: `reviews/task-83/002-global-cap-recovery-sweep/suite-aws-1m-global-cap-recovery-q500.json`
- Lane: AWS `1m`, PostgreSQL 18, database `postgres`, local socket `/var/run/postgresql`
- Fixture/index: `task67_1m_hnsw_m7g2xlarge`, `aws_spire_1m_rabitq_t80_block16_tg256_idx`
- Storage/rerank: `rabitq`, `rerank_width=25`
- Queries: q500, truth cache `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`
- Surface: isolated SPIRE 1M index, local scan path; no remote fanout in this suite.

## Suite Audit

- Artifact: `suite-audit.log`
- Command: `target/debug/ecaz bench suite audit --config reviews/task-83/002-global-cap-recovery-sweep/suite-aws-1m-global-cap-recovery-q500.json --log-file reviews/task-83/002-global-cap-recovery-sweep/artifacts/suite-audit.log`
- Result: `[suite:task83-aws-1m-global-cap-recovery-q500] audit passed: 3 steps`
- Note: the checked-in suite includes harmless top-level padding fields to force
  the cloud S3 config-upload path.

## AWS Commands

- Resume log: `cloud-resume-task83-global-cap-sweep.log`
  - Command: `target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-83/002-global-cap-recovery-sweep/artifacts/cloud-resume-task83-global-cap-sweep.log`
  - Result: `resume: profile=1m db=10.42.1.131 ready`
- Bench log: `cloud-bench-task83-global-cap-sweep.log`
  - Command: `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-83/002-global-cap-recovery-sweep/suite-aws-1m-global-cap-recovery-q500.json --suite task83-aws-1m-global-cap-recovery-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-83/002-global-cap-recovery-sweep/artifacts/cloud-bench-task83-global-cap-sweep.log`
  - Result: synced artifacts from `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task83-aws-1m-global-cap-recovery-q500/20260606T001806Z/`
- Pause log: `cloud-pause-after-task83-global-cap-sweep.log`
  - Command: `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-83/002-global-cap-recovery-sweep/artifacts/cloud-pause-after-task83-global-cap-sweep.log`
  - Result: `pause: profile=1m stopped (db + loader)`
- Status logs:
  - `cloud-status-final-paused.log`: captured while AWS was still `stopping`.
  - `cloud-status-final-stopped.log`: final status after propagation, `state: paused`.

## Synced Suite Artifacts

- `aws-1m-global-cap-recovery-q500/suite-config.json`
- `aws-1m-global-cap-recovery-q500/suite-manifest.json`
- `aws-1m-global-cap-recovery-q500/suite-run.log`
- `aws-1m-global-cap-recovery-q500/results.jsonl`
- `aws-1m-global-cap-recovery-q500/pipeline-spire-1m-rabitq-global1280-q500.log`
- `aws-1m-global-cap-recovery-q500/pipeline-spire-1m-rabitq-global1536-q500.log`
- `aws-1m-global-cap-recovery-q500/pipeline-spire-1m-rabitq-global1664-q500.log`

## Key Rows

Baseline from packet 001:

- `global1152`: `recall@10=0.9832`, `candidate_sum=9,213,846`,
  p50 `288.769 ms`, p95 `363.138 ms`, p99 `375.732 ms`.

Recovery sweep:

- `global1280`: `recall@10=0.9846`, `candidate_sum=10,237,554`,
  p50 `292.896 ms`, p95 `363.363 ms`, p99 `380.597 ms`.
- `global1536`: `recall@10=0.9876`, `candidate_sum=12,284,852`,
  p50 `287.312 ms`, p95 `344.188 ms`, p99 `354.646 ms`.
- `global1664`: `recall@10=0.9892`, `candidate_sum=13,308,518`,
  p50 `295.989 ms`, p95 `352.377 ms`, p99 `364.170 ms`.

## Closeout Decision

The sweep proves recall can be recovered by expanding the selected-block global
cap, but it does not justify landing that as the Task 83 recovery policy. The
candidate surface increases by `1.02M`, `3.07M`, and `4.09M` q500 candidates
for caps `1280`, `1536`, and `1664` respectively. That moves SPIRE back toward
the high candidate surfaces that Task 79/80 explicitly moved away from.

Task 83 therefore closes with measured attribution and the next recommendation:
improve selected-block scoring or implement selective near-cap rescue while
preserving the retained Task 79/81 candidate baseline.
