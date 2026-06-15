# Cell Retry Checkpoint: phase1-rabitq-100k-l1-control

Status: prepared; not started.

## Intent

- Checklist cell: `phase1-rabitq-100k-l1-control`.
- Phase: 1, single-node multi-disk / multi-store control.
- Scale: 100k representative corpus.
- Storage format: RaBitQ.
- Bits: 4.
- Store count: 1.
- Prefix: `task107_phase1_rabitq_100k_l1`.
- Index: `task107_phase1_rabitq_100k_l1_idx`.
- Artifact directory:
  `reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/retry-direct-ssm/`.

## Execution Policy

The retry should run until the cell completes or a command fails. On failure,
package the exact failure and proceed according to the checklist.

## Why This Retry Is Different

The previous node-local retry failed after S3 downloads and before load/build
evidence. This retry uses a simpler coordinator-local SSM payload:

- explicit `step=...` markers before each phase;
- no generated `printf` SQL file;
- `psql -c` for cleanup and residue checks;
- log upload from an `EXIT` trap, including failure paths;
- no remote shard loading;
- no comparator, HNSW, IVF, DiskANN, or Task 106 single-store reruns.

## Planned Commands

Start the existing Task 107 topology and refresh AutoStop:

```bash
scripts/spire-aws/start-topology-instances.sh \
  reviews/task-107/002-aws-provisioning/artifacts/aws-topology.json \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/retry-direct-ssm/aws-start

aws ec2 create-tags --region us-west-2 \
  --resources i-0b4386fa5017f1363 i-07bcc98c3d5d027ee i-00c2f2aca9dbdd6bd \
  --tags Key=AutoStop,Value=<refreshed-8-hour-deadline>
```

Send the coordinator-only SSM command using:

```bash
aws ssm send-command \
  --region us-west-2 \
  --instance-ids i-0b4386fa5017f1363 \
  --document-name AWS-RunShellScript \
  --comment "ecaz Task 107 phase1 rabitq 100k l1 coordinator-only load" \
  --parameters file://reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/retry-direct-ssm/ssm-parameters.json \
  --output-s3-bucket-name ecaz-spire-aws-20260614203301860100000009 \
  --output-s3-key-prefix task107/004/phase1-rabitq-100k-l1-control/retry-direct-ssm/ssm
```

If load/build succeeds, run the existing packet-local `ecaz bench suite` config,
capture storage evidence, clean up only the `task107_phase1_rabitq_100k_l1%`
objects, stop AWS, and record final EC2 state.
