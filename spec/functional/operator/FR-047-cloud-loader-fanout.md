---
id: FR-047
title: In-VPC Corpus Load Fan-Out
type: FR
status: PROPOSED
object: process
relationships:
  - target: "ix://agent-ix/ecaz/US-021"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-044"
    type: "supports"
    cardinality: "1:1"
---
# FR-047: In-VPC Corpus Load Fan-Out

## Description

`ecaz cloud corpus load` SHALL execute parquet → COPY ingestion
inside the database VPC by fanning out parallel workers on the
loader EC2, never streaming corpus bytes from the operator
workstation.

## Behavior

1. The loader EC2 SHALL be running and reachable via SSM during a
   `corpus load` (provisioned by `ecaz cloud up` and resumed by
   `ecaz cloud resume`); `corpus load` dispatches the load over SSM
   and does not itself start or stop the instance.
2. Parquet shards staged in S3 SHALL be loaded by N parallel workers
   on the loader EC2 (the `--workers` flag, default `8`, fanned out
   via `xargs -P<workers>`).
3. Each worker SHALL invoke the existing
   `ecaz corpus prepare` + `ecaz corpus load` code paths against
   the DB's private IP, reusing the streaming COPY implementation
   in `crates/ecaz-cli/src/corpus/load.rs` unchanged.
4. Index builds SHALL run after load, not during. Build time SHALL
   be measured and recorded as a separate artifact.
5. Worker progress (shard id, rows loaded, bytes streamed) SHALL be
   reported back to the operator's terminal in real time and
   persisted to S3 so a re-run with `--resume` skips completed
   shards.
6. When a single worker fails, other workers SHALL continue. The
   overall command SHALL exit non-zero with a summary of failed
   shards.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-047-AC-1 | A `dev`-profile load with 4 parquet shards spawns 4 concurrent workers on the loader EC2 (SSM exec history) | Demonstration |
| FR-047-AC-2 | After a `corpus load`, the DB row counts match the registry's declared `row_count` for the dataset | Demonstration |
| FR-047-AC-3 | Killing a worker mid-load and re-running with `--resume` completes the load without duplicating rows in already-loaded shards | Demonstration |
| FR-047-AC-4 | Load throughput meets or exceeds NFR-011 targets for the profile | Analysis |

### FR-047-AC-1

A `dev`-profile load with 4 parquet shards spawns 4 concurrent
workers on the loader EC2 (verified via SSM exec history).

### FR-047-AC-2

After a `corpus load`, the DB row counts match the registry's
declared `row_count` for the dataset.

### FR-047-AC-3

Killing a worker mid-load and re-running with `--resume` completes
the load without duplicating rows in already-loaded shards.

### FR-047-AC-4

Load throughput meets or exceeds NFR-011 targets for the profile.

## Workflow

```mermaid
sequenceDiagram
    participant Op as "Operator (ecaz cloud corpus load)"
    participant TF as Terraform state
    participant SSM as SSM
    participant Loader as "Loader EC2"
    participant S3 as "S3 bucket"
    participant DB as "DB host (private IP)"

    Op->>Op: lookup dataset in registry (FR-046)
    Op->>Op: ensure AWS credentials
    Op->>TF: read outputs (s3_bucket, db_private_ip, loader_instance_id, region)
    Op->>Op: resolve table (default = dataset name with dashes to underscores)
    Op->>SSM: run_shell fan-out script on loader_instance_id
    SSM->>Loader: execute load script as user loader
    Loader->>S3: list parquet shards under parquet/<dataset>/
    Loader->>Loader: xargs -P<workers> fans shards to N parallel workers
    loop per shard worker
        Loader->>S3: if --resume head-object state/load/<dataset>/<shard>.done then skip
        Loader->>S3: copy shard parquet to local workdir
        Loader->>Loader: ecaz corpus prepare (parquet to tsv)
        Loader->>DB: ecaz corpus load via PGHOST=db_private_ip (streaming COPY)
        Loader->>S3: write state/load/<dataset>/<shard>.done receipt
    end
    Loader-->>SSM: aggregate exit (non-zero if any worker failed, others still ran)
    SSM-->>Op: report load result (dataset, table, workers)
```

## Dependencies

- **Upstream**: US-021 (implements), FR-044 (supports), FR-045 (loader EC2 provisioning), FR-046 (dataset registry), NFR-011 (throughput targets)
- **Downstream**: none identified
