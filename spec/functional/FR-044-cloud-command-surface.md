---
id: FR-044
title: Ecaz Cloud Command Surface
type: functional-requirement
artifact_type: FR
status: PROPOSED
object_type: interface
relationships:
  - target: "ix://agent-ix/ecaz/US-021"
    type: "implements"
    cardinality: "N:1"
---
# FR-044: Ecaz Cloud Command Surface

## Requirement

Ecaz SHALL expose an `ecaz cloud` subcommand group that owns the full
provision → install → load → bench → teardown lifecycle for cloud-hosted
benchmark environments. Every verb SHALL be idempotent and runnable
without AWS console access, SSH, or manual SQL.

## Behavior

1. The CLI SHALL expose `ecaz cloud` with verbs:
   `up`, `install`, `corpus`, `bench`, `pause`, `resume`, `down`,
   `status`, `snapshot`.
2. `ecaz cloud up --profile <name>` SHALL apply the Terraform module
   for the named profile, wait for SSM agent readiness on both DB and
   loader instances, install ecaz on the DB host, and run
   `CREATE EXTENSION ecaz`. Re-running `up` on a stack that already
   exists SHALL be a no-op that returns the same DSN.
3. `ecaz cloud corpus stage --dataset <name>` and `ecaz cloud corpus
   load --dataset <name>` SHALL operate from a registry of named
   datasets (FR-046). `corpus load` SHALL fan out parallel workers
   on the loader EC2 (FR-047).
4. `ecaz cloud bench --suite <name>` SHALL invoke the existing
   `ecaz bench` entry points against the remote DSN and upload all
   `--log-file` artifacts to the profile's S3 bucket.
5. `ecaz cloud pause` SHALL call EC2 `StopInstances` on both DB and
   loader hosts. `ecaz cloud resume` SHALL `StartInstances`, wait for
   Postgres to accept connections, and re-emit the DSN.
6. `ecaz cloud snapshot` SHALL create an EBS snapshot of the DB
   volume and record the snapshot id in the profile's local state.
7. `ecaz cloud down` SHALL run `terraform destroy` and require an
   interactive confirmation unless `--yes` is passed. Re-running
   `down` on a torn-down stack SHALL be a no-op.
8. `ecaz cloud status` SHALL report, per profile: stack state
   (running/paused/down), instance ids, attached EBS volumes,
   recorded snapshot ids, and an estimated $/hr while running plus
   $/mo of retained storage. When a stack has been paused for >7
   days, status SHALL recommend `snapshot` + `down`.
9. `corpus load` and `bench` SHALL accept a `--resume` flag that
   skips already-completed shards or suite entries (mirroring the
   existing suite-runner pattern in `crates/ecaz-cli/src/bench/`).
10. AWS credentials SHALL be sourced from the standard AWS SDK chain
    (`AWS_PROFILE`, env vars, instance profile). Missing credentials
    SHALL produce a remediation message and a non-zero exit; no
    interactive prompting.

## Acceptance Criteria

### FR-044-AC-1

`ecaz cloud --help` lists every verb above and each verb dispatches
to the `ecaz-cloud` crate.

### FR-044-AC-2

Re-running any verb on an already-converged state exits zero with no
side effects (verified for `up`, `down`, `pause`, `resume`).

### FR-044-AC-3

`status` output is parsable as JSON with `--json` and matches the
true AWS state (verified by querying EC2 directly in tests).

### FR-044-AC-4

A `corpus load` interrupted between shards resumes from the next
incomplete shard when re-run with `--resume`.
