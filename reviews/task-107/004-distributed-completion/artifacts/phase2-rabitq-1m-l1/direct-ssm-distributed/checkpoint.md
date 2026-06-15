# Cell Checkpoint: phase2-rabitq-1m-l1

Status: prepared; setup not started.

## Intent

Run the Task 107 Phase 2 distributed RaBitQ 1m cell on the AWS topology with one coordinator and two remotes. This is not a single-node baseline and not a Task 106 rerun.

## Scope

- Prefix: `task107_phase2_rabitq_1m_l1`.
- Storage format: `rabitq`.
- Bits: `4`.
- Store count: `local_store_count=1` on coordinator and remotes.
- Topology: coordinator `i-0b4386fa5017f1363`; remotes `i-07bcc98c3d5d027ee` and `i-00c2f2aca9dbdd6bd`.
- Artifact directory: `reviews/task-107/004-distributed-completion/artifacts/phase2-rabitq-1m-l1/direct-ssm-distributed/`.

## Execution Policy

Run this cell by AWS SSM, one cell at a time. Benchmark sweeps run through `ecaz bench suite` using `suite-node.json`.

## Setup Command

- SSM setup payload: `setup-ssm-parameters.json`.
- Intended setup output prefix: `s3://ecaz-spire-aws-20260614203301860100000009/task107/004/phase2-rabitq-1m-l1/direct-ssm-distributed/`.
