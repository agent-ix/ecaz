# Task 107 Packet 001: AWS Topology Readiness Prerequisite

## Summary

This packet is a prerequisite slice for Task 107. It does not claim benchmark
completion. It closes two setup gaps found during Phase 0:

- The SPIRE AWS Terraform topology could not provision separate local-store
  volumes, so multi-disk evidence would have been indistinguishable from
  multiple stores on one root filesystem.
- The Phase 13 representative load/bench wrappers hard-coded the prepared
  100k profile, despite using a 1m-looking prefix. Task 107 needs both 100k
  and the real 1m profile.

## Code Changes

- Added opt-in coordinator gp3 local-store EBS volumes:
  `coordinator_extra_store_volume_count` and
  `coordinator_extra_store_volume_gb`.
- Exposed those volumes in the topology JSON with volume ID, device name,
  mount path, tablespace name, size, IOPS, and throughput.
- Added `scripts/spire-aws/setup-coordinator-store-volumes.sh`, which formats
  and mounts the coordinator store volumes through SSM and creates matching
  PostgreSQL tablespaces.
- Added `make -C infra/spire-aws setup-coordinator-store-volumes`.
- Made `scripts/spire-aws/load.sh` accept
  `SPIRE_AWS_REPRESENTATIVE_PREPARED_PREFIX`, `SPIRE_AWS_REPRESENTATIVE_DATASET`,
  and caller-provided `PREFIX`.
- Made `scripts/spire-aws/bench.sh` render the truth corpus path from
  `SPIRE_AWS_REPRESENTATIVE_PREPARED_PREFIX` and optionally rewrite benchmark
  step prefixes from `SPIRE_AWS_BENCH_PREFIX` / `PREFIX`.

## Validation

Packet-local evidence is in `artifacts/manifest.md`.

- `bash -n scripts/spire-aws/setup-coordinator-store-volumes.sh`
- `bash -n scripts/spire-aws/load.sh`
- `bash -n scripts/spire-aws/bench.sh`
- `terraform -chdir=infra/spire-aws validate`

All passed.

## Benchmark Execution Constraints

Per operator note, Task 107 runs must build one index at a time to avoid AWS
disk pressure:

- one storage format at a time (`turboquant` or `rabitq`);
- one local-store count at a time for single-node multi-store cells;
- explicit drop/cleanup before loading the next cell;
- comparator baselines cited only from existing packets, not rerun.

## Remaining Work

- Provision a Task 107 AWS topology with `remote_count = 2` and
  `coordinator_extra_store_volume_count >= 4`.
- Run 100k and real 1m SPIRE cells for TurboQuant and RaBitQ.
- Capture final build/storage/recall/NDCG/latency/fanout/disk/CPU/memory
  artifacts and publish the keep/drop/narrow recommendation in later Task 107
  packets.
