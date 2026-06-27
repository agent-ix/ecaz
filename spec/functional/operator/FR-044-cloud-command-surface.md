---
id: FR-044
title: Ecaz Cloud Command Surface
type: FR
status: PROPOSED
object: interface
relationships:
  - target: "ix://agent-ix/ecaz/US-021"
    type: "implements"
    cardinality: "N:1"
---
# FR-044: Ecaz Cloud Command Surface

## Description

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

| ID | Criteria | Verification |
|---|---|---|
| FR-044-AC-1 | `ecaz cloud --help` lists every verb and each verb dispatches to the `ecaz-cloud` crate | Test |
| FR-044-AC-2 | Re-running any verb on an already-converged state exits zero with no side effects (`up`, `down`, `pause`, `resume`) | Demonstration |
| FR-044-AC-3 | `status` output is parsable as JSON with `--json` and matches the true AWS state | Test |
| FR-044-AC-4 | A `corpus load` interrupted between shards resumes from the next incomplete shard when re-run with `--resume` | Demonstration |

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

## Contract

```yaml
interface: ecaz cloud
description: >-
  Owns the full provision -> install -> load -> bench -> teardown lifecycle for
  AWS-hosted ecaz benchmark stacks. Verbs dispatch to the `ecaz-cloud` crate;
  each verb takes `--profile <name>` and aims to be idempotent. AWS credentials
  are sourced from the standard AWS SDK chain (no interactive prompting).
operations:
  - name: up
    inputs:
      - { flag: --profile, type: Profile, required: true }
      - { flag: --git-ref, type: string, default: main }
      - { flag: --from-snapshot, type: string, optional: true, semantics: restore the DB EBS volume from a snapshot id }
      - { flag: --confirm-cost, type: u64, optional: true }
      - { flag: --dry-run, type: bool }
    output: provisioned stack + installed extension; returns DSN
    semantics: terraform apply, wait for SSM readiness, install ecaz on DB host, CREATE EXTENSION ecaz; no-op on an existing stack
  - name: install
    inputs:
      - { flag: --profile, type: Profile, required: true }
      - { flag: --git-ref, type: string, default: main }
      - { flag: --git-url, type: string, default: "https://github.com/agent-ix/ecaz.git" }
      - { flag: --timeout, type: u64, default: 1800 }
      - { flag: --extension-feature, type: string[] }
      - { flag: --skip-extension-recreate, type: bool }
      - { flag: --skip-cli-build, type: bool }
      - { flag: --clean-cargo-target, type: bool }
    output: re-installed ecaz extension/CLI on the DB host
    semantics: idempotent re-run of extension install on the DB host
  - name: corpus
    inputs:
      subcommands:
        - { name: list-datasets, flags: [--json], semantics: print the dataset registry (FR-046) as a table or JSON }
        - { name: stage, flags: [--profile, --dataset, --dry-run, --local, "--timeout (default 10800)"], semantics: upload parquet/converted-BIGANN shards for a dataset to the profile S3 bucket via the loader EC2 }
        - { name: load, flags: [--profile, --dataset, --resume, "--workers (default 8)", --table, "--timeout (default 21600)"], semantics: fan out parquet -> COPY workers on the loader EC2 (FR-047) }
    output: datasets staged in / loaded from S3 into the DB
    semantics: stage and load corpora from the named-dataset registry
  - name: bench
    inputs:
      - { flag: --profile, type: Profile, required: true }
      - { flag: --config, type: path, optional: true }
      - { flag: --suite, type: string, default: smoke }
      - { flag: --database, type: string, default: postgres }
      - { flag: --ecaz-bin, type: string, default: ecaz }
      - { flag: --skip-upload, type: bool }
      - { flag: --simd-mode, type: string, optional: true }
    output: bench artifacts; uploaded to the profile S3 bucket unless --skip-upload
    semantics: run the bench suite against the remote DSN and upload --log-file artifacts to S3
  - name: cleanup-scratch
    inputs:
      - { flag: --profile, type: Profile, required: true }
      - { flag: --path, type: string, required: true }
      - { flag: --restart-postgres, type: bool }
    output: removed task-scoped scratch staging on a cloud host
    semantics: remove scratch staging on a cloud host
  - name: pause
    inputs: [{ flag: --profile, type: Profile, required: true }]
    output: EC2 StopInstances on DB and loader; EBS retained
    semantics: stop instances, retain data; restore via resume
  - name: resume
    inputs:
      - { flag: --profile, type: Profile, required: true }
      - { flag: --wait-secs, type: u64, default: 300 }
    output: started instances; re-emitted DSN once Postgres accepts connections
    semantics: StartInstances and wait for Postgres readiness
  - name: snapshot
    inputs:
      - { flag: --profile, type: Profile, required: true }
      - { flag: --description, type: string, default: "ecaz cloud snapshot" }
    output: EBS snapshot id recorded in local per-profile state
    semantics: create an EBS snapshot of the DB volume
  - name: repair-state
    inputs:
      - { flag: --profile, type: Profile, required: true }
      - { flag: --forget-stale-db-volume, type: bool }
      - { flag: --dry-run, type: bool }
    output: repaired local Terraform state
    semantics: repair local Terraform state after an interrupted cloud lifecycle
  - name: down
    inputs:
      - { flag: --profile, type: Profile, required: true }
      - { flag: --yes, type: bool, semantics: skip interactive confirmation }
      - { flag: --no-snapshot-required, type: bool }
    output: terraform destroy; snapshots/bucket retained unless asked otherwise
    semantics: tear down the stack; no-op when already down
  - name: status
    inputs:
      - { flag: --profile, type: Profile, required: true }
      - { flag: --json, type: bool }
    output: stack state, instance ids, attached EBS volumes, snapshot ids, estimated $/hr and $/mo, recommended next verb
    semantics: report stack state; JSON output matches true AWS state
```

## Dependencies

- **Upstream**: US-021 (implements), FR-046 (dataset registry consumed by `corpus stage`/`corpus load`), FR-047 (loader fan-out used by `corpus load`)
- **Downstream**: FR-045 (Terraform infrastructure supports this surface)
