# phase1-rabitq-100k-l1-control Node-Local Retry Failure

Status: failed before corpus load/build benchmark evidence.

## Summary

The node-local retry did not complete. No recall, latency, storage, or routing
result from this retry should be cited.

The retry started the existing Task 107 AWS topology, refreshed the AutoStop tag,
and sent an SSM command to the coordinator to download the prepared 100k corpus
from S3 and run a coordinator-local `ecaz corpus load`. The SSM command failed
with response code 3 after downloading the corpus, queries, and manifest.

## Command Evidence

- SSM command id: `67e1b2e7-5ac9-4bf8-a66d-75c7e35cddda`
- Coordinator instance: `i-0b4386fa5017f1363`
- Final invocation: `load/ssm-command-invocation.final.json`
- Captured stdout: `load/ssm-stdout.log`
- Captured stderr: `load/ssm-stderr.log`

Final SSM status:

```text
Status: Failed
ResponseCode: 3
ExecutionStartDateTime: 2026-06-15T04:06:07.591Z
ExecutionEndDateTime: 2026-06-15T04:06:15.591Z
```

Captured stderr:

```text
failed to run commands: exit status 3
```

Captured stdout shows successful S3 downloads for:

- `ec_real_100k_corpus.tsv`
- `ec_real_100k_queries.tsv`
- `ec_real_100k_manifest.json`

No `ecaz corpus load` log, copy timing, index build timing, storage result, or
`ecaz bench suite` output was produced.

## Failure Point

The durable evidence shows the failure happened after the S3 downloads and
before any usable load/build artifact was uploaded. The SSM stderr does not
include a more specific shell line. The likely local cause is the generated
drop-SQL shell command in the SSM payload; a retry should avoid that fragile
shell quoting and use a simple `psql -c` cleanup command or a checked, narrow
coordinator-local load helper.

## AWS State

AWS was stopped after the failed retry:

- `aws-stop/stop-instances-after-ssm-failure.json`
- `aws-stop/describe-stopped-after-ssm-failure.json`

Final recorded state: coordinator and both remotes stopped.

## Cleanup State

Because the command failed before a load log or load completion artifact was
produced, the next attempt must still begin by checking for and dropping any
`task107_phase1_rabitq_100k_l1%` objects on the coordinator.

## Corrective Action

Do not start another benchmark until the next action is chosen. The immediate
options are:

- retry this same cell with a robust coordinator-local cleanup/load command;
- make a narrow script/CLI fix only if the coordinator-local load path cannot be
  expressed safely through SSM;
- move to an independent checklist cell after explicitly recording that this
  cell remains incomplete.
