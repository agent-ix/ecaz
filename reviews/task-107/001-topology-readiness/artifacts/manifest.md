# Task 107 Packet 001 Artifact Manifest

- Head SHA before this prerequisite slice: `c06975d8ec284071a13a11342a1ff4f436943b19`
- Task bucket: `reviews/task-107/001-topology-readiness/`
- Purpose: prerequisite readiness for AWS SPIRE multi-disk / multi-node benchmarks.
- Scope: operator support only. No Task 107 benchmark measurements are claimed in
  this packet.

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `bash-n-setup-coordinator-store-volumes.log` | `bash -n scripts/spire-aws/setup-coordinator-store-volumes.sh` | Passed. |
| `bash-n-load.log` | `bash -n scripts/spire-aws/load.sh` | Passed. |
| `bash-n-bench.log` | `bash -n scripts/spire-aws/bench.sh` | Passed. |
| `terraform-validate.log` | `terraform -chdir=infra/spire-aws validate` | Passed; configuration is valid. |

## Topology Metadata Required For Later Packets

Every Task 107 AWS result packet must record:

- software SHA and package artifact used on the nodes;
- node count and roles;
- coordinator and worker instance types;
- AZ, subnet, and same-AZ/cross-AZ placement;
- root volume and extra local-store volume IDs, type, size, IOPS, and throughput;
- mounted paths and PostgreSQL tablespace names;
- corpus profile, row count, query count, and storage format;
- local store count and `local_store_tablespaces`;
- distributed placement plan path, node-to-shard map, and remote descriptor snapshot;
- fanout GUCs, query concurrency, and nprobe/rerank settings;
- cleanup/teardown status for AWS resources.

## Disk-Space Rule

Task 107 benchmark execution must run one index/storage-format cell at a time:

1. Drop any previous cell tables/indexes for the target prefix.
2. Load/build exactly one SPIRE storage-format and store-count cell.
3. Capture build, storage, recall/NDCG, latency, pipeline/fanout, disk, CPU, and
   memory evidence.
4. Drop that cell before loading the next SPIRE cell.

This applies to both 100k and 1m scales and to both TurboQuant and RaBitQ.
