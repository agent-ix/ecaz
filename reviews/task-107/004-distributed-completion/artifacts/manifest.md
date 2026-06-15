# Task 107 Packet 004 Manifest

- Head SHA: `8ab418dffe02ff58f671dae0a24e1785eb56e86a`
- Task bucket: `reviews/task-107/004-distributed-completion/`
- Created: 2026-06-15T03:07:58Z
- Purpose: run-control packet for completing the remaining Task 107 AWS
  benchmark evidence without repeating unrelated or already-covered work.

## Scope

This packet begins after packet 003, which completed only the RaBitQ 100k
distributed lane and recorded several partial/non-decision attempts. Packet 004
must follow `../run-checklist.md` before any additional AWS benchmark work.

No benchmark/load has been run in this packet yet.

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
