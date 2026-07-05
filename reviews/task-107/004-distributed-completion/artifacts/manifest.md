# Task 107 Packet 004 Manifest

- Head SHA: `92254bca929c949f2de3715efefec6d4c53e4568`
- Task bucket: `reviews/task-107/004-distributed-completion/`
- Created: 2026-06-15T03:07:58Z
- Purpose: run-control packet for completing the remaining Task 107 AWS
  benchmark evidence without repeating unrelated or already-covered work.

## Scope

This packet begins after packet 003, which completed only the RaBitQ 100k
distributed lane and recorded several partial/non-decision attempts. Packet 004
must follow `../run-checklist.md` before any additional AWS benchmark work.

Packet 004's active checklist is limited to single node with 2 disks and
multinode with 1 controller plus 2 nodes. The in-scope completed packet-004
cells are `phase1-rabitq-100k-l2`, `phase1-rabitq-1m-l2`,
`phase1-turboquant-100k-l2`, and `phase1-turboquant-1m-l2`. Earlier packet-004
single-disk and 4-disk artifacts remain recorded only as historical,
out-of-scope evidence and do not count toward the current checklist. The
decision-grade evidence for the in-scope completed packet-004 cells is under
`phase1-rabitq-100k-l2/direct-ssm-tablespaces/`,
`phase1-rabitq-1m-l2/direct-ssm-tablespaces/`, and
`phase1-turboquant-100k-l2/direct-ssm-tablespaces/`, and
`phase1-turboquant-1m-l2/direct-ssm-tablespaces/`. The earlier
`phase1-rabitq-100k-l2/direct-ssm/` run is superseded because it omitted
explicit `local_store_tablespaces`.

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

After the final `phase2-turboquant-1m-l1` distributed benchmark, the three Task
107 instances were initially left running for operator review. The final cell's
coordinator and remote DB objects were cleaned up successfully. This state is
superseded by the `AWS Teardown` section below, where Terraform destroy and
direct AWS checks verify that the instances, volumes, bucket, VPC, IAM
resources, and secrets were removed.

Earlier after `phase1-turboquant-1m-l2`, AWS was checked directly in
`phase1-turboquant-1m-l2/direct-ssm-tablespaces/aws-state/describe-after-cell.json`:

- `i-0b4386fa5017f1363` (`ecaz-spire-aws-coord`): running,
  `AutoStop=2026-06-17T05:30:31Z`.
- `i-07bcc98c3d5d027ee` (`ecaz-spire-aws-remote-1`): running,
  `AutoStop=2026-06-17T05:30:31Z`.
- `i-00c2f2aca9dbdd6bd` (`ecaz-spire-aws-remote-2`): running,
  `AutoStop=2026-06-17T05:30:31Z`.

### Out-of-Scope / Drift Artifact Quarantine

The decision packet must draw only from the corrected checklist: single node
with 2 disks and multinode with 1 coordinator plus 2 remotes. The following
packet-004 artifacts remain committed only as historical drift/debug evidence
and are not decision-grade checklist cells:

- `phase1-rabitq-100k-l1-control/`
- `phase1-rabitq-1m-l1-control/`
- `phase1-rabitq-100k-l4/`
- `phase1-rabitq-1m-l4/direct-ssm-tablespaces/`
- `phase1-rabitq-100k-l2/direct-ssm/`

In particular, `phase1-rabitq-1m-l4/direct-ssm-tablespaces/` is abandoned
out-of-scope evidence from the discontinued 4-disk path. It should not be read
as in-flight benchmark work.

## Run List Source Of Truth

The run checklist is `../run-checklist.md`. It enumerates:

- remaining Phase 1 single-node 2-disk cells;
- remaining Phase 2 cells for one controller and two nodes;
- completed packet-003 evidence that must be cited instead of rerun;
- stop/go checkpoints for avoiding infrastructure drift and accidental reruns.

## Prepared Cell Checkpoints

- `phase1-rabitq-100k-l1-control/checkpoint.md`
  - Status: checkpoint prepared; attempt 1 failed before benchmark.
  - Next cell: `phase1-rabitq-100k-l1-control`.
  - Execution policy: run the cell to completion or command failure.
  - Scope: 100k representative corpus, RaBitQ, `bits=4`,
    `local_store_count=1`, one coordinator index only.
  - Benchmark matrix: rendered single-node `ecaz bench suite` config derived
    from packet-003 `suite-single-node.json`.
- `phase1-rabitq-100k-l1-control/failure.md`
  - Status: failed before benchmark.
  - Root cause: attempted to stream the 2GB 100k corpus through an SSM
    port-forward instead of using a node-local AWS load path.
  - AWS state after failure: stopped, recorded in
    `phase1-rabitq-100k-l1-control/aws-stop/describe-stopped-after-interruption.json`.
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
  - Execution policy: ran to completion.
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
- `phase1-rabitq-100k-l2/direct-ssm/checkpoint.md`
  - Status: superseded by
    `phase1-rabitq-100k-l2/direct-ssm-tablespaces/`.
  - Reason: this run used `local_store_count=2` but omitted explicit
    `local_store_tablespaces`, so it is retained only as history.
- `phase1-rabitq-100k-l2/direct-ssm-tablespaces/checkpoint.md`
  - Status: completed.
  - Cell: `phase1-rabitq-100k-l2`.
  - Execution policy: ran one isolated index cell to completion.
  - Scope: one coordinator-only RaBitQ 100k index with `bits=4`,
    `local_store_count=2`, and
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`; no remote
    shard loading and no comparator or Task 106 reruns.
  - Command payload:
    `phase1-rabitq-100k-l2/direct-ssm-tablespaces/ssm-parameters.json`.
  - Load/build artifacts:
    - `phase1-rabitq-100k-l2/direct-ssm-tablespaces/load/ssm-command-invocation.final.json`
    - `phase1-rabitq-100k-l2/direct-ssm-tablespaces/load/load.log`
    - `phase1-rabitq-100k-l2/direct-ssm-tablespaces/load/inspect.log`
  - Load/build result: 100000 corpus rows, 1000 queries, `bits=4`,
    `local_store_count=2`,
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`,
    `storage_format=rabitq`; copy 32.26s, encode 33.77s, index build 89.72s,
    total 168.94s.
  - Routing/fanout evidence: `load/inspect.log` and `storage/storage.log`
    both record the isolated `task107_phase1_rabitq_100k_l2_idx` index with
    reloptions `{local_store_count=2,
    local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2,
    storage_format=rabitq}`.
  - Recall/latency artifacts:
    - `phase1-rabitq-100k-l2/direct-ssm-tablespaces/13a3a-recall-k10.log`
    - `phase1-rabitq-100k-l2/direct-ssm-tablespaces/13a3a-recall-k100.log`
    - `phase1-rabitq-100k-l2/direct-ssm-tablespaces/13a3a-latency-k10-c1.log`
    - `phase1-rabitq-100k-l2/direct-ssm-tablespaces/13a3a-latency-k10-c4.log`
    - `phase1-rabitq-100k-l2/direct-ssm-tablespaces/13a3a-latency-k10-c8.log`
    - `phase1-rabitq-100k-l2/direct-ssm-tablespaces/13a3f-pk-c32.log`
    - `phase1-rabitq-100k-l2/direct-ssm-tablespaces/bench/suite-results-single-node.jsonl`
  - Key recall results:
    - k10 nprobe 8/16/24/32/64: 0.7939 / 0.8703 / 0.9041 / 0.9268 / 0.9661.
    - k100 nprobe 8/16/24/32/64: 0.6862 / 0.7899 / 0.8362 / 0.8687 / 0.9336.
  - Key latency results:
    - k10 c1 mean nprobe 8/16/24/32: 83.8 / 118.1 / 150.9 / 183.7 ms.
    - k10 c4 mean nprobe 8/16/24/32: 90.5 / 122.2 / 157.0 / 191.6 ms.
    - k10 c8 mean nprobe 8/16/24/32: 122.9 / 131.0 / 161.9 / 199.3 ms.
    - k1 c32 nprobe 32 mean: 488.7 ms.
  - Storage artifact:
    `phase1-rabitq-100k-l2/direct-ssm-tablespaces/storage/storage.log`.
  - Storage result: total 1.6 GiB; storage command reports `ec_spire` relation
    size 64.0 KiB with `local_store_count=2`,
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`, and
    `storage_format=rabitq`.
  - Cleanup artifact:
    `phase1-rabitq-100k-l2/direct-ssm-tablespaces/cleanup/cleanup-drop.log`.
  - Cleanup result: dropped index, queries table, and corpus table; no remaining
    `task107_phase1_rabitq_100k_l2%` relations were printed.
  - AWS state after cell:
    `phase1-rabitq-100k-l2/direct-ssm-tablespaces/aws-state/describe-after-cell.json`;
    all three Task 107 instances remained running for the next cell.
- `phase1-rabitq-100k-l4/direct-ssm-tablespaces/checkpoint.md`
  - Status: completed.
  - Cell: `phase1-rabitq-100k-l4`.
  - Execution policy: ran one isolated index cell to completion.
  - Scope: one coordinator-only RaBitQ 100k index with `bits=4`,
    `local_store_count=4`, and
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2,ecaz_spire_store_3,ecaz_spire_store_4`;
    no remote shard loading and no comparator or Task 106 reruns.
  - Command payload:
    `phase1-rabitq-100k-l4/direct-ssm-tablespaces/ssm-parameters.json`.
  - Load/build artifacts:
    - `phase1-rabitq-100k-l4/direct-ssm-tablespaces/load/ssm-command-invocation.final.json`
    - `phase1-rabitq-100k-l4/direct-ssm-tablespaces/load/load.log`
    - `phase1-rabitq-100k-l4/direct-ssm-tablespaces/load/inspect.log`
  - Load/build result: 100000 corpus rows, 1000 queries, `bits=4`,
    `local_store_count=4`,
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2,ecaz_spire_store_3,ecaz_spire_store_4`,
    `storage_format=rabitq`; copy 32.24s, encode 23.79s, index build 89.71s,
    total 159.15s.
  - Routing/fanout evidence: `load/inspect.log` and `storage/storage.log`
    both record the isolated `task107_phase1_rabitq_100k_l4_idx` index with
    reloptions `{local_store_count=4,
    local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2,ecaz_spire_store_3,ecaz_spire_store_4,
    storage_format=rabitq}`.
  - Recall/latency artifacts:
    - `phase1-rabitq-100k-l4/direct-ssm-tablespaces/13a3a-recall-k10.log`
    - `phase1-rabitq-100k-l4/direct-ssm-tablespaces/13a3a-recall-k100.log`
    - `phase1-rabitq-100k-l4/direct-ssm-tablespaces/13a3a-latency-k10-c1.log`
    - `phase1-rabitq-100k-l4/direct-ssm-tablespaces/13a3a-latency-k10-c4.log`
    - `phase1-rabitq-100k-l4/direct-ssm-tablespaces/13a3a-latency-k10-c8.log`
    - `phase1-rabitq-100k-l4/direct-ssm-tablespaces/13a3f-pk-c32.log`
    - `phase1-rabitq-100k-l4/direct-ssm-tablespaces/bench/suite-results-single-node.jsonl`
  - Key recall results:
    - k10 nprobe 8/16/24/32/64: 0.7939 / 0.8703 / 0.9041 / 0.9268 / 0.9661.
    - k100 nprobe 8/16/24/32/64: 0.6862 / 0.7899 / 0.8362 / 0.8687 / 0.9336.
  - Key latency results:
    - k10 c1 mean nprobe 8/16/24/32: 81.3 / 114.6 / 145.0 / 178.6 ms.
    - k10 c4 mean nprobe 8/16/24/32: 89.3 / 118.8 / 152.0 / 186.1 ms.
    - k10 c8 mean nprobe 8/16/24/32: 123.5 / 129.0 / 160.0 / 193.9 ms.
    - k1 c32 nprobe 32 mean: 492.8 ms.
  - Storage artifact:
    `phase1-rabitq-100k-l4/direct-ssm-tablespaces/storage/storage.log`.
  - Storage result: total 1.6 GiB; storage command reports `ec_spire` relation
    size 64.0 KiB with `local_store_count=4`,
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2,ecaz_spire_store_3,ecaz_spire_store_4`,
    and `storage_format=rabitq`.
  - Cleanup artifact:
    `phase1-rabitq-100k-l4/direct-ssm-tablespaces/cleanup/cleanup-drop.log`.
  - Cleanup result: dropped index, queries table, and corpus table; no remaining
    `task107_phase1_rabitq_100k_l4%` relations were printed.
  - AWS state after cell:
    `phase1-rabitq-100k-l4/direct-ssm-tablespaces/aws-state/describe-after-cell.json`;
    all three Task 107 instances remained running for the next cell.
- `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/checkpoint.md`
  - Status: completed.
  - Cell: `phase1-rabitq-1m-l1-control`.
  - Execution policy: ran one isolated index cell to completion using the
    existing 1m representative corpus and a resume from the valid index left by
    the timed-out first SSM attempt.
  - Scope: one coordinator-only RaBitQ 1m index with `bits=4`,
    `local_store_count=1`, and
    `local_store_tablespaces=ecaz_spire_store_1`; no remote shard loading and
    no comparator or Task 106 reruns.
  - Command payloads:
    - first attempt:
      `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/ssm-parameters.json`;
    - resume:
      `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/ssm-resume-parameters.json`.
  - SSM evidence:
    - first attempt:
      `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/load/ssm-command-invocation.latest.json`;
    - resume final status:
      `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume-command-invocation.latest.json`;
    - resume stdout:
      `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/ssm/5b1a96bd-3efd-4253-94f2-185475af4f55/i-0b4386fa5017f1363/awsrunShellScript/0.awsrunShellScript/stdout`.
  - Load/build artifacts:
    - `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/load/ssm-stderr.txt`
    - `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/load/index-validity.log`
    - `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/load/inspect.log`
  - Load/build result: 990000 corpus rows, 10000 queries, valid `ec_spire`
    index `task107_phase1_rabitq_1m_l1_idx`, `bits=4`,
    `local_store_count=1`,
    `local_store_tablespaces=ecaz_spire_store_1`,
    `storage_format=rabitq`; first-attempt load timings were corpus copy
    317.26s, encode 433.28s, query copy 3.38s, and the index was later found
    valid after the SSM document timeout.
  - Routing/fanout evidence: `resume/load/index-validity.log`,
    `resume/load/inspect.log`, and `resume/bench/storage.log` record the
    isolated `task107_phase1_rabitq_1m_l1_idx` index with reloptions
    `{local_store_count=1,local_store_tablespaces=ecaz_spire_store_1,storage_format=rabitq}`.
  - Recall/latency artifacts:
    - `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/bench/13a3a-recall-k10.log`
    - `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/bench/13a3a-recall-k100.log`
    - `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/bench/13a3a-latency-k10-c1.log`
    - `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/bench/13a3a-latency-k10-c4.log`
    - `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/bench/13a3a-latency-k10-c8.log`
    - `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/bench/13a3f-pk-c32.log`
    - `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/bench/suite-results-node.jsonl`
  - Key recall results:
    - k10 nprobe 8/16/24/32/64: 0.8110 / 0.8820 / 0.9060 / 0.9340 / 0.9690.
    - k100 nprobe 8/16/24/32/64: 0.7626 / 0.8425 / 0.8763 / 0.8988 / 0.9375.
  - Key latency results:
    - k10 c1 mean nprobe 8/16/24/32: 187.9 / 336.2 / 487.4 / 620.9 ms.
    - k10 c4 mean nprobe 8/16/24/32: 211.4 / 382.6 / 541.4 / 678.5 ms.
    - k10 c8 mean nprobe 8/16/24/32: 230.3 / 404.3 / 575.3 / 721.7 ms.
    - k1 c32 nprobe 32 mean: 1505.3 ms.
  - Storage artifact:
    `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/bench/storage.log`.
  - Storage result: total 16.1 GiB; `ec_spire` index 784.8 MiB,
    831.3 B/row with `local_store_count=1`,
    `local_store_tablespaces=ecaz_spire_store_1`, and
    `storage_format=rabitq`.
  - Cleanup artifact:
    `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/resume/load/cleanup-drop.log`.
  - Cleanup result: dropped index, queries table, and corpus table; no
    remaining `task107_phase1_rabitq_1m_l1%` relations were printed in the
    empty `resume/load/residue-after-cleanup.log`.
  - AWS state after cell:
    `phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/aws-state/describe-after-cell.json`;
    all three Task 107 instances remained running for the next cell.
- `phase1-rabitq-1m-l2/direct-ssm-tablespaces/checkpoint.md`
  - Status: completed.
  - Cell: `phase1-rabitq-1m-l2`.
  - Execution policy: ran one isolated index cell to completion using the
    existing 1m representative corpus.
  - Scope: one coordinator-only RaBitQ 1m index with `bits=4`,
    `local_store_count=2`, and
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`; no remote
    shard loading and no comparator or Task 106 reruns.
  - Command payload:
    `phase1-rabitq-1m-l2/direct-ssm-tablespaces/ssm-parameters.json`.
  - SSM evidence:
    - final invocation:
      `phase1-rabitq-1m-l2/direct-ssm-tablespaces/load/ssm-command-invocation.latest.json`;
    - stdout/stderr:
      `phase1-rabitq-1m-l2/direct-ssm-tablespaces/ssm/63bd3e6c-a375-4957-acde-a146a33dc1ca/i-0b4386fa5017f1363/awsrunShellScript/0.awsrunShellScript/`.
  - SSM result: `Status=Success`, `ResponseCode=0`, execution window
    `2026-06-15T10:07:44.318Z` to `2026-06-15T11:22:57.318Z`, elapsed
    `PT1H15M13.844S`.
  - Load/build artifacts:
    - `phase1-rabitq-1m-l2/direct-ssm-tablespaces/load/load.log`
    - `phase1-rabitq-1m-l2/direct-ssm-tablespaces/load/inspect.log`
  - Load/build result: 990000 corpus rows, 10000 queries, `bits=4`,
    `local_store_count=2`,
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`,
    `storage_format=rabitq`; corpus copy 320.23s, encode 421.44s, query copy
    3.37s, index build 2680.67s, total 3553.77s.
  - Routing/fanout evidence: `load/inspect.log` and `bench/storage.log` record
    the isolated `task107_phase1_rabitq_1m_l2_idx` index with reloptions
    `{local_store_count=2,"local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2",storage_format=rabitq}`.
  - Recall/latency artifacts:
    - `phase1-rabitq-1m-l2/direct-ssm-tablespaces/bench/13a3a-recall-k10.log`
    - `phase1-rabitq-1m-l2/direct-ssm-tablespaces/bench/13a3a-recall-k100.log`
    - `phase1-rabitq-1m-l2/direct-ssm-tablespaces/bench/13a3a-latency-k10-c1.log`
    - `phase1-rabitq-1m-l2/direct-ssm-tablespaces/bench/13a3a-latency-k10-c4.log`
    - `phase1-rabitq-1m-l2/direct-ssm-tablespaces/bench/13a3a-latency-k10-c8.log`
    - `phase1-rabitq-1m-l2/direct-ssm-tablespaces/bench/13a3f-pk-c32.log`
    - `phase1-rabitq-1m-l2/direct-ssm-tablespaces/bench/suite-results-node.jsonl`
  - Key recall results:
    - k10 nprobe 8/16/24/32/64: 0.8110 / 0.8820 / 0.9060 / 0.9340 / 0.9690.
    - k100 nprobe 8/16/24/32/64: 0.7626 / 0.8425 / 0.8763 / 0.8988 / 0.9375.
  - Key latency results:
    - k10 c1 mean nprobe 8/16/24/32: 203.3 / 361.9 / 521.1 / 658.0 ms.
    - k10 c4 mean nprobe 8/16/24/32: 217.8 / 387.4 / 556.3 / 703.4 ms.
    - k10 c8 mean nprobe 8/16/24/32: 232.6 / 411.6 / 584.0 / 743.1 ms.
    - k1 c32 nprobe 32 mean: 1527.0 ms.
  - Storage artifact:
    `phase1-rabitq-1m-l2/direct-ssm-tablespaces/bench/storage.log`.
  - Storage result: total 15.4 GiB; `ec_spire` index 168.0 KiB, 0.2 B/row
    with `local_store_count=2`,
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`, and
    `storage_format=rabitq`.
  - Cleanup artifact:
    `phase1-rabitq-1m-l2/direct-ssm-tablespaces/load/cleanup-drop.log`.
  - Cleanup result: dropped index, queries table, and corpus table; no
    remaining `task107_phase1_rabitq_1m_l2%` relations were printed in the
    empty `load/residue-after-cleanup.log`.
  - AWS state after cell:
    `phase1-rabitq-1m-l2/direct-ssm-tablespaces/aws-state/describe-after-cell.json`;
    all three Task 107 instances remained running for the next cell.
- `phase1-turboquant-100k-l2/direct-ssm-tablespaces/checkpoint.md`
  - Status: completed.
  - Cell: `phase1-turboquant-100k-l2`.
  - Execution policy: ran one isolated index cell to completion using the
    existing 100k representative corpus.
  - Scope: one coordinator-only TurboQuant 100k index with `bits=4`,
    `local_store_count=2`, and
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`; no remote
    shard loading and no comparator or Task 106 reruns.
  - Command payload:
    `phase1-turboquant-100k-l2/direct-ssm-tablespaces/ssm-parameters.json`.
  - SSM evidence:
    - final invocation:
      `phase1-turboquant-100k-l2/direct-ssm-tablespaces/load/ssm-command-invocation.final.json`;
    - stdout/stderr:
      `phase1-turboquant-100k-l2/direct-ssm-tablespaces/ssm/6f98340e-4524-41fb-8938-54c4c6c72fc7/i-0b4386fa5017f1363/awsrunShellScript/0.awsrunShellScript/`.
  - SSM result: `Status=Success`, `ResponseCode=0`, execution window
    `2026-06-15T12:43:04.535Z` to `2026-06-15T13:19:58.535Z`, elapsed
    `PT36M54.883S`.
  - Load/build artifacts:
    - `phase1-turboquant-100k-l2/direct-ssm-tablespaces/load/load.log`
    - `phase1-turboquant-100k-l2/direct-ssm-tablespaces/load/inspect.log`
  - Load/build result: 100000 corpus rows, 1000 queries, `bits=4`,
    `local_store_count=2`,
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`,
    `storage_format=turboquant`; corpus copy 32.46s, encode 23.94s, query
    copy 416.78ms, index build 90.02s, total 159.82s.
  - Routing/fanout evidence: `load/inspect.log` and `bench/storage.log` record
    the isolated `task107_phase1_turboquant_100k_l2_idx` index with reloptions
    `{local_store_count=2,"local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2",storage_format=turboquant}`.
  - Recall/latency artifacts:
    - `phase1-turboquant-100k-l2/direct-ssm-tablespaces/bench/13a3a-recall-k10.log`
    - `phase1-turboquant-100k-l2/direct-ssm-tablespaces/bench/13a3a-recall-k100.log`
    - `phase1-turboquant-100k-l2/direct-ssm-tablespaces/bench/13a3a-latency-k10-c1.log`
    - `phase1-turboquant-100k-l2/direct-ssm-tablespaces/bench/13a3a-latency-k10-c4.log`
    - `phase1-turboquant-100k-l2/direct-ssm-tablespaces/bench/13a3a-latency-k10-c8.log`
    - `phase1-turboquant-100k-l2/direct-ssm-tablespaces/bench/13a3f-pk-c32.log`
    - `phase1-turboquant-100k-l2/direct-ssm-tablespaces/bench/suite-results-node.jsonl`
  - Key recall results:
    - k10 nprobe 8/16/24/32/64: 0.7939 / 0.8703 / 0.9041 / 0.9268 / 0.9661.
    - k100 nprobe 8/16/24/32/64: 0.6862 / 0.7899 / 0.8362 / 0.8687 / 0.9336.
  - Key latency results:
    - k10 c1 mean nprobe 8/16/24/32: 47.6 / 82.8 / 117.4 / 159.4 ms.
    - k10 c4 mean nprobe 8/16/24/32: 49.9 / 83.4 / 122.5 / 156.2 ms.
    - k10 c8 mean nprobe 8/16/24/32: 49.0 / 84.6 / 121.6 / 159.8 ms.
    - k1 c32 nprobe 32 mean: 327.1 ms.
  - Storage artifact:
    `phase1-turboquant-100k-l2/direct-ssm-tablespaces/bench/storage.log`.
  - Storage result: total 1.6 GiB; `ec_spire` index 64.0 KiB, 0.7 B/row with
    `local_store_count=2`,
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`, and
    `storage_format=turboquant`.
  - Cleanup artifact:
    `phase1-turboquant-100k-l2/direct-ssm-tablespaces/load/cleanup-drop.log`.
  - Cleanup result: dropped index, queries table, and corpus table; no
    remaining `task107_phase1_turboquant_100k_l2%` relations were printed in
    the empty `load/residue-after-cleanup.log`.
  - AWS state after cell:
    `phase1-turboquant-100k-l2/direct-ssm-tablespaces/aws-state/describe-after-cell.json`;
    all three Task 107 instances remained running for the next cell.
- `phase1-turboquant-1m-l2/direct-ssm-tablespaces/checkpoint.md`
  - Status: completed.
  - Cell: `phase1-turboquant-1m-l2`.
  - Execution policy: ran one isolated index cell to completion using the
    existing 1m representative corpus.
  - Scope: one coordinator-only TurboQuant 1m index with `bits=4`,
    `local_store_count=2`, and
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`; no remote
    shard loading and no comparator or Task 106 reruns.
  - Command payload:
    `phase1-turboquant-1m-l2/direct-ssm-tablespaces/ssm-parameters.json`.
  - SSM evidence:
    - final invocation:
      `phase1-turboquant-1m-l2/direct-ssm-tablespaces/load/ssm-command-invocation.final.json`;
    - stdout/stderr:
      `phase1-turboquant-1m-l2/direct-ssm-tablespaces/ssm/74873503-5793-43b0-b65e-6ff92c8bc08d/i-0b4386fa5017f1363/awsrunShellScript/0.awsrunShellScript/`.
  - SSM result: `Status=Success`, `ResponseCode=0`, execution window
    `2026-06-15T13:29:07.038Z` to `2026-06-15T14:44:26.038Z`, elapsed
    `PT1H15M19.4S`.
  - Load/build artifacts:
    - `phase1-turboquant-1m-l2/direct-ssm-tablespaces/load/load.log`
    - `phase1-turboquant-1m-l2/direct-ssm-tablespaces/load/inspect.log`
  - Load/build result: 990000 corpus rows, 10000 queries, `bits=4`,
    `local_store_count=2`,
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`,
    `storage_format=turboquant`; corpus copy 320.20s, encode 449.26s, query
    copy 5.72s, index build 2678.40s, total 3583.10s.
  - Routing/fanout evidence: `load/inspect.log` and `bench/storage.log` record
    the isolated `task107_phase1_turboquant_1m_l2_idx` index with reloptions
    `{local_store_count=2,"local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2",storage_format=turboquant}`.
  - Recall/latency artifacts:
    - `phase1-turboquant-1m-l2/direct-ssm-tablespaces/bench/13a3a-recall-k10.log`
    - `phase1-turboquant-1m-l2/direct-ssm-tablespaces/bench/13a3a-recall-k100.log`
    - `phase1-turboquant-1m-l2/direct-ssm-tablespaces/bench/13a3a-latency-k10-c1.log`
    - `phase1-turboquant-1m-l2/direct-ssm-tablespaces/bench/13a3a-latency-k10-c4.log`
    - `phase1-turboquant-1m-l2/direct-ssm-tablespaces/bench/13a3a-latency-k10-c8.log`
    - `phase1-turboquant-1m-l2/direct-ssm-tablespaces/bench/13a3f-pk-c32.log`
    - `phase1-turboquant-1m-l2/direct-ssm-tablespaces/bench/suite-results-node.jsonl`
  - Key recall results:
    - k10 nprobe 8/16/24/32/64: 0.8110 / 0.8820 / 0.9060 / 0.9340 / 0.9690.
    - k100 nprobe 8/16/24/32/64: 0.7626 / 0.8425 / 0.8763 / 0.8988 / 0.9375.
  - Key latency results:
    - k10 c1 mean nprobe 8/16/24/32: 188.1 / 343.6 / 499.1 / 620.1 ms.
    - k10 c4 mean nprobe 8/16/24/32: 214.6 / 380.1 / 542.3 / 686.4 ms.
    - k10 c8 mean nprobe 8/16/24/32: 233.2 / 410.0 / 583.9 / 732.6 ms.
    - k1 c32 nprobe 32 mean: 1525.1 ms.
  - Storage artifact:
    `phase1-turboquant-1m-l2/direct-ssm-tablespaces/bench/storage.log`.
  - Storage result: total 15.4 GiB; `ec_spire` index 168.0 KiB, 0.2 B/row
    with `local_store_count=2`,
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`, and
    `storage_format=turboquant`.
  - Cleanup artifact:
    `phase1-turboquant-1m-l2/direct-ssm-tablespaces/load/cleanup-drop.log`.
  - Cleanup result: dropped index, queries table, and corpus table; no
    remaining `task107_phase1_turboquant_1m_l2%` relations were printed in the
    empty `load/residue-after-cleanup.log`.
  - AWS state after cell:
    `phase1-turboquant-1m-l2/direct-ssm-tablespaces/aws-state/describe-after-cell.json`;
    all three Task 107 instances remained running for the next cell.
- `phase2-rabitq-1m-l1/direct-ssm-distributed/`
  - Status: completed.
  - Cell: `phase2-rabitq-1m-l1`.
  - Execution policy: resumed one distributed RaBitQ 1m cell to completion
    after the packet-003 cancellation and packet-004 export retry; no
    single-node/single-disk or comparator rows were rerun.
  - Scope: one coordinator plus two remotes, `bits=4`,
    `local_store_count=1`, `storage_format=rabitq`.
  - Coordinator setup/export artifacts:
    - `setup/coordinator-load.log`
    - `resume-export/distributed-placement-plan.path`
    - `distributed-representative/distributed-placement-plan.json`
  - Remote load/materialization artifacts:
    - `remote-load-node-2/load.log`
    - `remote-load-node-3/load.log`
    - `remote-materialize-node-2/remote-materialize.log`
    - `remote-materialize-node-3/remote-materialize.log`
    - `remote-materialize-node-2/identity.json`
    - `remote-materialize-node-3/identity.json`
  - Registration and suite artifacts:
    - `coordinator-register-run/ssm-command-invocation.final.json`
    - `coordinator-register-run/register-remotes.log`
    - `coordinator-register-run/remote-node-snapshot.jsonl`
    - `bench/suite-manifest-node.json`
    - `bench/suite-results-node.jsonl`
  - SSM result: coordinator registration and suite command
    `ed487c10-f162-4a30-ba71-21994dc99ea9` completed with `Status=Success`,
    `ResponseCode=0`, elapsed `PT8M41.939S`.
  - Load/build result: coordinator load/build reused the completed
    `task107_phase2_rabitq_1m_l1_idx`; remote node 2 loaded 504734 rows and
    remote node 3 loaded 485266 rows. Both remotes materialized their exported
    coordinator leaf-base assignments and emitted `endpoint_status=ready` and
    `tuple_transport_status=ready`.
  - Registration/routing evidence: `publish-remote-placements.log` reports
    995 rewritten placements across 2 remote nodes; `register-remotes.log`
    reports `registered_node_2=t` and `registered_node_3=t`;
    `remote-node-snapshot.jsonl` reports node 2 with 498 available placements
    and node 3 with 497 available placements, both `status=ready`,
    `descriptor_state=active`, `descriptor_generation=1`,
    `local_store_count=1`, and `last_error=none`.
  - Key recall results:
    - k10 nprobe 8/16/24/32/64: 0.7940 / 0.8650 / 0.8880 / 0.9160 / 0.9510.
    - k100 nprobe 8/16/24/32/64: 0.7583 / 0.8376 / 0.8710 / 0.8935 / 0.9321.
  - Key latency results:
    - k10 c1 mean nprobe 8/16/24/32: 89.3 / 103.6 / 111.8 / 121.3 ms.
    - k10 c4 mean nprobe 8/16/24/32: 95.7 / 107.7 / 117.4 / 126.0 ms.
    - k10 c8 mean nprobe 8/16/24/32: 101.8 / 113.7 / 123.6 / 133.0 ms.
    - k1 c32 nprobe 32 mean: 228.4 ms.
  - Production remote-read evidence:
    - k10 nprobe 64 recall 0.9510; `result_source=remote_heap_candidates`,
      `status=ready`, `remote_pid_sum=6400`, `dispatch_sum=200`,
      `total_p50=117.000 ms`, `total_p95=135.000 ms`.
    - k100 nprobe 64 recall 0.9321; `result_source=remote_heap_candidates`,
      `status=ready`, `remote_pid_sum=6400`, `dispatch_sum=200`,
      `total_p50=118.000 ms`, `total_p95=137.000 ms`.
  - Storage result: 990000 rows, total 16.1 GiB, indexes 806.4 MiB;
    `task107_phase2_rabitq_1m_l1_idx` reports 784.9 MiB with reloptions
    `{local_store_count=1,storage_format=rabitq}`.
  - Cleanup artifacts:
    - `cleanup/coordinator/ssm-command-invocation.final.json`
    - `cleanup/remote-node-2/ssm-command-invocation.final.json`
    - `cleanup/remote-node-3/ssm-command-invocation.final.json`
    - `cleanup/coordinator/residue-after.log`
    - `cleanup/remote-node-2/residue-after.log`
    - `cleanup/remote-node-3/residue-after.log`
  - Cleanup result: coordinator and both remote cleanup commands completed
    with `Status=Success`, `ResponseCode=0`; all three `residue-after.log`
    files are empty.
- `phase2-turboquant-100k-l1/direct-ssm-distributed/`
  - Status: completed after commit
    `92254bca929c949f2de3715efefec6d4c53e4568` enabled TurboQuant SPIRE
    remote endpoints. PQ-fastscan remains unsupported as expected.
  - Cell: `phase2-turboquant-100k-l1`.
  - Scope: one coordinator plus two remotes, `bits=4`,
    `local_store_count=1`, `storage_format=turboquant`.
  - Key readiness evidence: both remote identities report
    `endpoint_status=ready` and `tuple_transport_status=ready`; node 2
    endpoint identity hex `94648bf2daaa390a`, node 3 endpoint identity hex
    `f020eba543c7568a`.
  - Key recall results:
    - k10 nprobe64: 0.9600, mean q-time 64.51 ms.
    - k100 nprobe64: 0.9369, mean q-time 65.82 ms.
  - Production remote-read evidence:
    - k10/k100 nprobe64 both `status=ready`,
      `result_source=remote_heap_candidates`, `selected_pid_sum=6400`,
      `remote_pid_sum=6400`, `dispatch_sum=200`, and
      timeout/cancel/degraded-skip sums all zero.
  - Storage result: 100000 rows, total 1.6 GiB, index 81.5 MiB.
  - Cleanup result: coordinator and both remote cleanup commands completed
    with `Status=Success`, `ResponseCode=0`.
- `phase2-turboquant-1m-l1/direct-ssm-distributed/`
  - Status: completed with suite threshold miss.
  - Cell: `phase2-turboquant-1m-l1`.
  - Scope: one coordinator plus two remotes, `bits=4`,
    `local_store_count=1`, `storage_format=turboquant`.
  - Coordinator setup/export result: SSM command
    `7256baf7-5bbe-4571-a176-6a91e11d1254` completed successfully. Load/build
    timings: corpus copy 318.09s, encode 489.47s, index build 2691.81s, total
    3634.45s.
  - Remote load result: node 2 SSM command
    `077d6515-9995-471e-99ff-5a5be7cad15a` loaded 504734 rows and completed
    successfully; node 3 SSM command `74a0fbdb-d8d9-4d0b-a6dc-1a29ab6c0d27`
    loaded 485266 rows and completed successfully.
  - Remote materialization result: node 2 SSM command
    `c2ef0085-1fe4-4010-adde-fdbbeb96ad78` completed successfully; node 3 SSM
    command `e5f0e481-0b13-4f24-b2b2-90e378d8755b` completed successfully.
  - Registration and suite artifacts:
    - `coordinator-register-run-send-command.json`
    - `coordinator-register-run/run-suite.log`
    - `coordinator-register-run/run-suite.stderr.log`
    - `coordinator-register-run/remote-node-snapshot.jsonl`
    - `bench/suite-manifest-node.json`
    - `bench/suite-results-node.jsonl`
    - `bench/13a3a-recall-k10.log`
    - `bench/13a3a-recall-k100.log`
    - `bench/13e3-production-read-profile-k10.log`
    - `bench/13e3-production-read-profile-k100.log`
    - `bench/storage.log`
  - Suite result: coordinator command
    `70dc864d-df90-407b-9931-419d0e61a68c` exited `Status=Failed`,
    `ResponseCode=1`, elapsed `PT9M5.705S`, because the suite threshold
    summary reported `suite thresholds failed: 2`. All 9 suite steps in
    `suite-manifest-node.json` have `status=succeeded`.
  - Threshold miss:
    - `phase2-turboquant-1m-l1-recall-k10-nprobe64-floor`: actual
      `recall@k=0.9490`, expected `>=0.9500`.
    - `phase2-turboquant-1m-l1-remote-read-k10-nprobe64-floor`: actual
      `recall@k=0.9490`, expected `>=0.9500`.
  - Key recall results:
    - k10 nprobe 8/16/24/32/64: 0.7940 / 0.8640 / 0.8860 / 0.9140 / 0.9490.
    - k100 nprobe64: 0.9331.
  - Production remote-read evidence:
    - k10 nprobe64: `status=ready`,
      `result_source=remote_heap_candidates`, `selected_pid_sum=6400`,
      `remote_pid_sum=6400`, `dispatch_sum=200`, `total_p50=140.000 ms`,
      `total_p95=164.000 ms`, timeout/cancel/degraded-skip sums all zero.
    - k100 nprobe64: `status=ready`,
      `result_source=remote_heap_candidates`, `selected_pid_sum=6400`,
      `remote_pid_sum=6400`, `dispatch_sum=200`, `total_p50=141.000 ms`,
      `total_p95=166.000 ms`, timeout/cancel/degraded-skip sums all zero.
  - Storage result: 990000 rows, total 16.1 GiB, indexes 805.6 MiB;
    `task107_phase2_turboquant_1m_l1_idx` reports 784.1 MiB with reloptions
    `{local_store_count=1,storage_format=turboquant}`.
  - Cleanup result: coordinator command
    `13795924-9f0d-4a81-a21d-66f5e75d8380`, remote node 2 command
    `1e1f2668-763f-494b-86a2-6e26b08ad2da`, and remote node 3 command
    `860aa065-47b4-469a-92fb-985614749369` all completed with
    `Status=Success`, `ResponseCode=0`.

### AWS Teardown

- `aws-teardown/teardown-summary.md`
  - Status: completed.
  - Command: `make -C infra/spire-aws ARTIFACT_DIR=/home/peter/dev/ecaz/reviews/task-107/004-distributed-completion/artifacts/aws-teardown teardown`.
  - Result: Terraform destroy completed successfully with `37 destroyed`.
  - Direct verification:
    - `aws-teardown/terraform-state-list-after-destroy.log`: no
      Terraform-managed resources remained in state.
    - `aws-teardown/describe-instances-after-destroy.log`: all three Task 107
      EC2 instances reported `terminated`.
    - `aws-teardown/describe-volumes-after-destroy.log`: the Task 107 EBS
      volume ids returned `InvalidVolume.NotFound`.
    - `aws-teardown/head-bucket-after-destroy.log`: the Task 107 artifact
      bucket returned `404 Not Found`.
    - `aws-teardown/residue-final-after-secret-notfound.log`: no matching S3
      buckets, Secrets Manager secrets, VPCs, IAM role, or IAM instance
      profile remained.
