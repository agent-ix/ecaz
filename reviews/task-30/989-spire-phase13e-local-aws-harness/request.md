# Review Request: SPIRE Phase 13e Local AWS Harness Gate

## Summary

This slice adds a repeatable local PG18 gate that runs the AWS Phase 13e
correctness workflow without AWS: four local PostgreSQL instances, the
10k/1536 correctness corpus, and the checked-in `scripts/spire-aws/load.sh`,
`register.sh`, and `smoke.sh` path.

It reproduced the AWS `remote_candidate_receive_failed` locally, then fixed the
root cause: remote `conninfo_secret_name` values were not visible to the
coordinator PostgreSQL backend. The local harness now exports conninfo before
coordinator startup, and the AWS install path now writes a systemd
`EnvironmentFile` for all remote conninfo entries onto the coordinator and
restarts PostgreSQL before registration/read.

## Code Under Review

- `scripts/run_spire_phase13e_aws_harness_local_pg18.sh`
- `scripts/spire-aws/install.sh`
- `scripts/spire-aws/smoke-customscan-read.sql`
- `scripts/spire-aws/smoke.sh`
- `scripts/spire-aws/suite-correctness.json`
- `scripts/spire-aws/suite-representative.json`
- `scripts/spire-aws/suite-stress.json`

## Evidence

Passing local AWS-shape run:

- Command:
  `bash scripts/run_spire_phase13e_aws_harness_local_pg18.sh --skip-install --artifact-dir reviews/task-30/989-spire-phase13e-local-aws-harness/artifacts/conninfo-before-start-run`
- `smoke-customscan-read.log` shows `Custom Scan (EcSpireDistributedScan)` and `remote_fanout: 3`.
- `production-read-profile-smoke.log` shows `status=ready`, `result_source=remote_heap_candidates`, `dispatch_count=3`, `socket_open_count=3`, `candidate_receive_query_count=3`, and `heap_receive_query_count=3`.
- `phase13e-aws-harness-local.log` ends with `SPIRE Phase 13e AWS harness local PG18 fixture passed`.

Reproduction before the fix:

- `artifacts/escalated-skip-install-run/phase13e-aws-harness-local.log` shows
  `conninfo_secret_missing` during descriptor registration and then
  `remote_candidate_receive_failed` on node 2 during CustomScan.

Syntax validation:

- `bash -n scripts/run_spire_phase13e_aws_harness_local_pg18.sh scripts/spire-aws/install.sh scripts/spire-aws/smoke.sh`
- `jq empty scripts/spire-aws/suite-correctness.json scripts/spire-aws/suite-representative.json scripts/spire-aws/suite-stress.json`

See `artifacts/manifest.md` for the artifact inventory and cited lines.
