# Task 107 Packet 004 Manifest

- Head SHA: `82c59a02efc592aa82936d3b13ac1d65e213f97f`
- Task bucket: `reviews/task-107/004-distributed-completion/`
- Created: 2026-06-15T03:07:58Z
- Purpose: run-control packet for completing the remaining Task 107 AWS
  benchmark evidence without repeating unrelated or already-covered work.

## Scope

This packet begins after packet 003, which completed only the RaBitQ 100k
distributed lane and recorded several partial/non-decision attempts. Packet 004
must follow `../run-checklist.md` before any additional AWS benchmark work.

Packet 004 has completed `phase1-rabitq-100k-l1-control`. The earlier failed
attempts remain recorded as non-decision-grade history; the decision-grade
evidence for this cell is under
`phase1-rabitq-100k-l1-control/retry-direct-ssm/`.

## Current Packet Artifacts

### AWS Start / Preflight / Stop

- `aws-start/start-topology-instances.log`
  - Command: `scripts/spire-aws/start-topology-instances.sh reviews/task-107/002-aws-provisioning/artifacts/aws-topology.json reviews/task-107/004-distributed-completion/artifacts/aws-start`
  - Result: started the existing coordinator and two remote instances.
- `aws-start/start-topology-instance-state.log.before`
  - Instance state before the packet-004 start command.
- `aws-start/start-topology-instance-state.log.after`
  - Instance state after EC2 reported running/status-ok.
- `aws-start/refresh-autostop-tags.json`
  - Command: refreshed `AutoStop=2026-06-15T10:55:40Z` on all three Task 107
    instances.
- `aws-start/describe-after-autostop-refresh.json`
  - Verified running instance metadata and AutoStop tag after refresh.
- `preflight/coordinator-task107-objects.log`
  - Command: coordinator SQL preflight through SSM tunnels.
  - Result: coordinator was reachable as `ecaz_coord`; no stale `task107_%`
    relations were printed.
- `preflight/tunnel-*.log`
  - SSM tunnel logs for the preflight command.
- `aws-stop/stop-instances.json`
  - Command: `aws ec2 stop-instances` for the coordinator and two remotes.
  - Result: stop request accepted.
- `aws-stop/describe-stopped-instances.json`
  - Command: `aws ec2 describe-instances` for the coordinator and two remotes
    after the interrupted stop wait.
  - Result: all three instances reported `stopped`.

### Current AWS State

After the interrupted stop wait, AWS was checked directly in
`aws-stop/describe-stopped-instances.json`:

- `i-0b4386fa5017f1363` (`ecaz-spire-aws-coord`): stopped.
- `i-07bcc98c3d5d027ee` (`ecaz-spire-aws-remote-1`): stopped.
- `i-00c2f2aca9dbdd6bd` (`ecaz-spire-aws-remote-2`): stopped.

## Run List Source Of Truth

The run checklist is `../run-checklist.md`. It enumerates:

- remaining Phase 1 single-node multi-disk/multi-store cells;
- remaining Phase 2 distributed multi-node cells;
- completed packet-003 evidence that must be cited instead of rerun;
- stop/go checkpoints for avoiding infrastructure drift and accidental reruns.

## Prepared Cell Checkpoints

- `phase1-rabitq-100k-l1-control/checkpoint.md`
  - Status: checkpoint prepared; attempt 1 failed before benchmark.
  - Next cell: `phase1-rabitq-100k-l1-control`.
  - Runtime policy: no arbitrary wall-clock cap; run the cell to completion or
    command failure.
  - Scope: 100k representative corpus, RaBitQ, `bits=4`,
    `local_store_count=1`, one coordinator index only.
  - Benchmark matrix: rendered single-node `ecaz bench suite` config derived
    from packet-003 `suite-single-node.json`.
- `phase1-rabitq-100k-l1-control/failure.md`
  - Status: failed before benchmark.
  - Root cause: attempted to stream the 2GB 100k corpus through an SSM
    port-forward instead of using a node-local AWS load path.
  - AWS state after failure: stopped, recorded in
    `phase1-rabitq-100k-l1-control/aws-stop/describe-stopped-after-timeout.json`.
  - No load/build, recall, latency, storage, or routing result from this
    attempt is decision-grade.
- `phase1-rabitq-100k-l1-control/retry-node-local/failure.md`
  - Status: failed before corpus load/build benchmark evidence.
  - SSM command id: `67e1b2e7-5ac9-4bf8-a66d-75c7e35cddda`.
  - Final SSM result: `Status=Failed`, `ResponseCode=3`, execution window
    `2026-06-15T04:06:07.591Z` to `2026-06-15T04:06:15.591Z`.
  - Evidence: `retry-node-local/load/ssm-stdout.log` shows the coordinator
    downloaded the 100k corpus, queries, and manifest from S3; stderr only
    reports `failed to run commands: exit status 3`.
  - Failure point: after S3 downloads and before any usable `ecaz corpus load`
    log or load/build output.
  - AWS state after retry: stopped, recorded in
    `phase1-rabitq-100k-l1-control/retry-node-local/aws-stop/describe-stopped-after-ssm-failure.json`.
  - No load/build, recall, latency, storage, or routing result from this retry
    is decision-grade.
- `phase1-rabitq-100k-l1-control/retry-direct-ssm/checkpoint.md`
  - Status: completed.
  - Next cell: `phase1-rabitq-100k-l1-control`.
  - Runtime policy: no arbitrary wall-clock cap; ran to completion.
  - Scope: one coordinator-only RaBitQ 100k index with `bits=4` and
    `local_store_count=1`; no remote shard loading and no comparator or
    Task 106 reruns.
  - Command payload: `phase1-rabitq-100k-l1-control/retry-direct-ssm/ssm-parameters.json`.
  - Load/build artifacts:
    - `retry-direct-ssm/load/ssm-command-invocation.final.json`
    - `retry-direct-ssm/load/load.log`
    - `retry-direct-ssm/load/inspect.log`
  - Load/build result: 100000 corpus rows, 1000 queries, `bits=4`,
    `local_store_count=1`, `storage_format=rabitq`; copy 32.20s, encode
    24.65s, index build 89.52s, total 162.16s.
  - Recall/latency artifacts:
    - `phase1-rabitq-100k-l1-control/13a3a-recall-k10.log`
    - `phase1-rabitq-100k-l1-control/13a3a-recall-k100.log`
    - `phase1-rabitq-100k-l1-control/13a3a-latency-k10-c1.log`
    - `phase1-rabitq-100k-l1-control/13a3a-latency-k10-c4.log`
    - `phase1-rabitq-100k-l1-control/13a3a-latency-k10-c8.log`
    - `phase1-rabitq-100k-l1-control/13a3f-pk-c32.log`
    - `retry-direct-ssm/bench/suite-results-single-node.jsonl`
  - Key recall results:
    - k10 nprobe 8/16/24/32/64: 0.7939 / 0.8703 / 0.9041 / 0.9268 / 0.9661.
    - k100 nprobe 8/16/24/32/64: 0.6862 / 0.7899 / 0.8362 / 0.8687 / 0.9336.
  - Key latency results:
    - k10 c1 mean nprobe 8/16/24/32: 99.7 / 136.4 / 162.2 / 190.0 ms.
    - k10 c4 mean nprobe 8/16/24/32: 94.1 / 124.6 / 158.3 / 196.6 ms.
    - k10 c8 mean nprobe 8/16/24/32: 124.8 / 131.3 / 163.0 / 198.1 ms.
    - k1 c32 nprobe 32 mean: 499.3 ms.
  - Storage artifact: `retry-direct-ssm/storage/storage.log`.
  - Storage result: total 1.6 GiB, index 81.7 MiB.
  - Cleanup artifact: `retry-direct-ssm/cleanup/cleanup-drop.log`.
  - Cleanup result: dropped index, queries table, and corpus table; no remaining
    `task107_phase1_rabitq_100k_l1%` relations were printed.
