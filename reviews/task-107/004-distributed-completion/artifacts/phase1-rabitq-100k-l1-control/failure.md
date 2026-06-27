# phase1-rabitq-100k-l1-control Attempt 1 Failure

Status: failed before benchmark.

## Summary

The cell did not complete. No recall, latency, storage, or routing result from
this attempt should be cited.

The corrected `ecaz corpus load` command started through an SSM port-forward
and attempted to send the 2GB 100k corpus from the local workstation through
the tunnel. It was stopped before copy completion was logged. AWS was stopped
to terminate the stalled cell.

## What Ran

- AWS start completed:
  `aws-start/start-topology-instances.log`.
- AutoStop was refreshed:
  `aws-start/refresh-autostop-tags.json`.
- Preflight completed:
  `preflight/coordinator-task107-objects.log`.
- Existing local 100k corpus was selected for reuse:
  `corpus-reuse.log`.
- Single-node suite config was rendered:
  `suite-single-node.json`.
- First load command failed immediately because it specified storage format
  twice:
  `--storage-format rabitq` and `--reloption storage_format=rabitq`.
- Retried load command without the duplicate reloption, then stopped during the
  earlier interrupted attempt.

## Failure Details

Initial invocation error:

```text
--reloption storage_format=... conflicts with --storage-format. Use the native CLI flag or drop the --reloption, not both
```

Retried invocation final error after AWS stop:

```text
COPY send failed for ecaz_corpus_stage
connection closed
```

The packet-local `load.log` contains only input inspection and manifest-warning
lines, with no completed copy/build timings.

## Root Cause

The planned command used local `ecaz corpus load` over an SSM port-forward for
a multi-GB corpus file. That is the wrong transport for this AWS benchmark
cell. The established SPIRE AWS load path uses node-local SSM/S3 transfer so
the corpus is copied to the coordinator and loaded locally.

## AWS State

AWS was stopped after the interrupted attempt:

- `aws-stop/stop-instances-after-interruption.json`
- `aws-stop/describe-stopped-after-interruption.json`

Final recorded state: coordinator and both remotes stopped.

## Cleanup State

Cleanup SQL was not completed in this attempt because the interruption was
handled by stopping the instances. Before any retry, the first command after
startup must check for and drop any `task107_phase1_rabitq_100k_l1%` objects.

## Corrective Action

Retry this same cell using the node-local AWS loading pattern:

- upload/reuse the prepared corpus via S3 or the existing
  `scripts/spire-aws/load.sh` node-local path;
- build only `task107_phase1_rabitq_100k_l1_idx`;
- run the already-rendered single-node `ecaz bench suite` config after load
  succeeds;
- capture storage, cleanup, and stopped AWS state.
