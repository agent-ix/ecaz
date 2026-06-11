---
id: FR-047
title: In-VPC Corpus Load Fan-Out
type: functional-requirement
artifact_type: FR
status: PROPOSED
object_type: process
relationships:
  - target: "ix://agent-ix/ecaz/US-021"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-044"
    type: "supports"
    cardinality: "1:1"
---
# FR-047: In-VPC Corpus Load Fan-Out

## Requirement

`ecaz cloud corpus load` SHALL execute parquet → COPY ingestion
inside the database VPC by fanning out parallel workers on the
loader EC2, never streaming corpus bytes from the operator
workstation.

## Behavior

1. The loader EC2 SHALL be running and reachable via SSM during a
   `corpus load`. If stopped, `corpus load` SHALL start it and stop
   it on exit (configurable via `--keep-loader`).
2. Parquet shards staged in S3 SHALL be sharded across N workers
   (default `min(8, num_shards)`); workers run in parallel on the
   loader EC2.
3. Each worker SHALL invoke the existing
   `ecaz corpus prepare` + `ecaz corpus load` code paths against
   the DB's private IP, reusing the streaming COPY implementation
   in `crates/ecaz-cli/src/corpus/load.rs` unchanged.
4. Index builds SHALL run after load, not during; build time SHALL
   be measured and recorded as a separate artifact.
5. Worker progress (shard id, rows loaded, bytes streamed) SHALL be
   reported back to the operator's terminal in real time and
   persisted to S3 so a re-run with `--resume` skips completed
   shards.
6. On failure of a single worker, other workers SHALL continue;
   the overall command SHALL exit non-zero with a summary of failed
   shards.

## Acceptance Criteria

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
