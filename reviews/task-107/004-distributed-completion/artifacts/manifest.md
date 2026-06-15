# Task 107 Packet 004 Manifest

- Head SHA: `31ec11b202b8ba2bb3a85c40f158c1e2962a7c0e`
- Task bucket: `reviews/task-107/004-distributed-completion/`
- Created: 2026-06-15T03:07:58Z
- Purpose: run-control packet for completing the remaining Task 107 AWS
  benchmark evidence without repeating unrelated or already-covered work.

## Scope

This packet begins after packet 003, which completed only the RaBitQ 100k
distributed lane and recorded several partial/non-decision attempts. Packet 004
must follow `../run-checklist.md` before any additional AWS benchmark work.

No packet-004 benchmark cell has completed yet. The
`phase1-rabitq-100k-l1-control` cell has two failed attempts, both before any
decision-grade load/build, storage, recall/latency, or routing result.

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
  - Expected maximum runtime before stopping and asking for review: 45 minutes.
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
