# Task 120 Packet 012 Artifact Manifest

- head SHA: `27f93162487436e4f29e8170fdf3b625a1bdb48e`
- task bucket: `reviews/task-120/`
- packet path: `reviews/task-120/012-aws-representative-packet-path-guard/`
- timestamp: `2026-06-21T20:41:40Z`
- lane / fixture / storage / rerank:
  - lane: SPIRE AWS representative preflight harness
  - fixture: no benchmark data loaded; this is a path-guard code checkpoint
  - storage format: not applicable
  - rerank mode: not applicable
  - isolated one-index-per-table or shared table: not applicable
- remote/distributed: no EC2 resources were provisioned; the preflight checked
  operator config, clean Terraform state, AWS permissions, and representative
  pass readiness for a Task 120 packet-local artifact path.

## Artifacts

### `bash-n.log`

- command:
  `bash -n scripts/spire-aws/run-representative-performance-pass.sh && bash -n scripts/spire-aws/preflight-representative-performance.sh`
- key result lines:
  - `bash -n scripts/spire-aws/run-representative-performance-pass.sh: OK`
  - `bash -n scripts/spire-aws/preflight-representative-performance.sh: OK`

### `preflight-task120-artifact-dir.log`

- command:
  `env SPIRE_AWS_ALLOW_NONDEFAULT_GRAVITON_LANE=1 make -C infra/spire-aws ARTIFACT_DIR=/home/peter/dev/ecaz/reviews/task-120/012-aws-representative-packet-path-guard/artifacts preflight-operator preflight-state preflight-permissions preflight-representative-performance`
- key result lines:
  - `SPIRE AWS nondefault Graviton lane override accepted: coordinator=r7g.4xlarge remote=r7g.2xlarge`
  - `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-04e0d7d889f694536 coordinator=r7g.4xlarge remote=r7g.2xlarge remote_count=2`
  - `SPIRE AWS state preflight passed: local Terraform state has no managed resources`
  - `SPIRE AWS permission preflight passed`
  - `SPIRE representative performance preflight passed: priority=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-priority.json pooling=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-pooling.json`
- note: the `make[1]: *** [Makefile:8: pass-correctness-body] Error 42`
  line is an intentional negative-control check inside
  `preflight-representative-performance.sh`; the outer preflight command exited
  successfully.
